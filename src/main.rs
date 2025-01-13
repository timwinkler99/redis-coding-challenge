use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};

use redis::parse;

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

        let response = handle_command(&command);
        println!("Writing to stream: {:?}", response);
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Failed to write to stream: {}", e);
            break;
        }
    }
}

fn handle_command(command: &Vec<redis::Token>) -> String {
    if command.len() < 1 {
        return String::from("-ERR Invalid command\r\n");
    }

    match command.first() {
        Some(redis::Token::String(cmd)) => {
            match cmd.to_uppercase().as_str() {
                "PING" => {
                    if command.len() != 1 {
                        return String::from(
                            "-ERR wrong number of arguments for 'PING' command\r\n",
                        );
                    }
                    String::from("+PONG\r\n")
                }
                "ECHO" => {
                    // ECHO should have exactly 2 tokens (ECHO and the argument)
                    if command.len() != 2 {
                        return String::from("-ERR ECHO requires exactly one argument\r\n");
                    }
                    // Get the second token (the argument)
                    if let Some(redis::Token::String(s)) = command.get(1) {
                        format!("${}\r\n{}\r\n", s.len(), s)
                    } else {
                        String::from("-ERR Invalid ECHO argument format\r\n")
                    }
                }
                _ => format!("-ERR Unknown command '{}'\r\n", cmd),
            }
        }
        _ => String::from("-ERR Invalid command format\r\n"),
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
        let command_uc = vec![redis::Token::String("PING".to_string())];
        assert_eq!(handle_command(&command_uc), "+PONG\r\n");

        let command_lc = vec![redis::Token::String("ping".to_string())];
        assert_eq!(handle_command(&command_lc), "+PONG\r\n");
    }

    #[test]
    fn test_handle_command_echo() {
        let command_uc = vec![
            redis::Token::String("ECHO".to_string()),
            redis::Token::String("hello world".to_string()),
        ];
        assert_eq!(handle_command(&command_uc), "+hello world\r\n");

        let command_lc = vec![
            redis::Token::String("echo".to_string()),
            redis::Token::String("hello world".to_string()),
        ];
        assert_eq!(handle_command(&command_lc), "+hello world\r\n");
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
            reader.read_line(&mut response).unwrap();

            assert_eq!(response, "+hello\r\n");
        } else {
            panic!("Failed to connect to server");
        }
    }
}
