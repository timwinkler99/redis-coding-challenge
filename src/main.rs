use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};

use redis::{parse, RespValue};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Connection established!");

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    loop {
        let command = {
            let mut buf_reader = BufReader::new(&stream);
            match parse(&mut buf_reader) {
                Ok(command) => command,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        // Client closed the connection
                        println!("Client disconnected");
                        break;
                    } else {
                        eprintln!("Failed to parse command: {}", e);
                        if let Err(e) = stream.write_all(b"-ERR Failed to parse command\r\n") {
                            eprintln!("Failed to write error message: {}", e);
                            break;
                        }
                        continue;
                    }
                }
            }
        };

        let result = handle_command(&command);
        let response = result.serialize();
        println!("Writing to stream: {:?}", response);
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Failed to write to stream: {}", e);
            break;
        }
    }
}

fn handle_command(command: &RespValue) -> RespValue {
    match command {
        RespValue::Array(Some(elements)) => {
            if elements.is_empty() {
                return RespValue::Error("ERR Invalid command".to_string());
            }

            match &elements[0] {
                RespValue::BulkString(Some(cmd)) => {
                    match cmd.to_uppercase().as_str() {
                        "PING" => {
                            if elements.len() != 1 {
                                return RespValue::Error(
                                    "ERR wrong number of arguments for 'PING' command".to_string(),
                                );
                            }
                            RespValue::SimpleString("PONG".to_string())
                        }
                        "ECHO" => {
                            // ECHO should have exactly 2 elements (ECHO and the argument)
                            if elements.len() != 2 {
                                return RespValue::Error(
                                    "ERR wrong number of arguments for 'ECHO' command".to_string(),
                                );
                            }
                            // Get the second element (the argument)
                            match &elements[1] {
                                RespValue::BulkString(Some(s)) => {
                                    RespValue::BulkString(Some(s.clone()))
                                }
                                RespValue::BulkString(None) => RespValue::BulkString(None),
                                _ => {
                                    RespValue::Error("ERR Invalid ECHO argument format".to_string())
                                }
                            }
                        }
                        _ => RespValue::Error(format!("ERR Unknown command '{}'", cmd)),
                    }
                }
                _ => RespValue::Error("ERR Invalid command format".to_string()),
            }
        }
        _ => RespValue::Error("ERR Invalid command format".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_handle_command_ping() {
        let command = RespValue::Array(Some(vec![RespValue::BulkString(Some("PING".to_string()))]));
        assert_eq!(
            handle_command(&command),
            RespValue::SimpleString("PONG".to_string())
        );

        let command = RespValue::Array(Some(vec![RespValue::BulkString(Some("ping".to_string()))]));
        assert_eq!(
            handle_command(&command),
            RespValue::SimpleString("PONG".to_string())
        );
    }

    #[test]
    fn test_handle_command_echo() {
        let command = RespValue::Array(Some(vec![
            RespValue::BulkString(Some("ECHO".to_string())),
            RespValue::BulkString(Some("hello world".to_string())),
        ]));
        assert_eq!(
            handle_command(&command),
            RespValue::BulkString(Some("hello world".to_string()))
        );

        let command = RespValue::Array(Some(vec![
            RespValue::BulkString(Some("echo".to_string())),
            RespValue::BulkString(Some("hello world".to_string())),
        ]));
        assert_eq!(
            handle_command(&command),
            RespValue::BulkString(Some("hello world".to_string()))
        );
    }

    fn start_server() {
        thread::spawn(|| {
            main();
        });
        // Give the server a moment to start
        thread::sleep(Duration::from_millis(100));
    }

    #[test]
    fn test_tcp_ping() {
        start_server();

        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:6379") {
            stream.write_all(b"*1\r\n$4\r\nPING\r\n").unwrap();

            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            reader.read_line(&mut response).unwrap();

            assert_eq!(response, "+PONG\r\n");
        } else {
            panic!("Failed to connect to server");
        }
    }

    #[test]
    fn test_tcp_echo() {
        start_server();

        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:6379") {
            stream
                .write_all(b"*2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n")
                .unwrap();

            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            let mut buffer = String::new();

            // Read the response in a loop until we have the complete response
            loop {
                buffer.clear();
                let bytes_read = reader.read_line(&mut buffer).unwrap();
                if bytes_read == 0 {
                    break; // End of stream
                }
                response.push_str(&buffer);
                // Check if we have read the complete response
                if response.ends_with("\r\n") && response.contains("\r\nhello\r\n") {
                    break;
                }
            }

            assert_eq!(response, "$5\r\nhello\r\n");
        } else {
            panic!("Failed to connect to server");
        }
    }
}
