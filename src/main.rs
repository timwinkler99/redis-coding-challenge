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
    let mut buf_reader = BufReader::new(&stream);

    match parse(&mut buf_reader) {
        Ok(command) => {
            println!("{:?}", command);
            let response = handle_command(&command);
            stream.write_all(response.as_bytes()).unwrap();
        }
        Err(e) => {
            eprintln!("Failed to parse command: {}", e);
            stream
                .write_all(b"-ERR Failed to parse command\r\n")
                .unwrap();
        }
    }
}

fn handle_command(command: &Vec<redis::Token>) -> String {
    if command.len() < 1 {
        return String::from("-ERR Invalid command\r\n");
    }

    match command.first() {
        Some(redis::Token::String(cmd)) => {
            match cmd.as_str() {
                "PING" => String::from("+PONG\r\n"),
                "ECHO" => {
                    if command.len() < 2 {
                        return String::from("-ERR ECHO requires an argument\r\n");
                    }
                    // Join all remaining tokens after ECHO
                    let echo_text: String = command[1..]
                        .iter()
                        .filter_map(|token| {
                            if let redis::Token::String(s) = token {
                                Some(s.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<&str>>()
                        .join(" ");
                    format!("+{}\r\n", echo_text)
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
        let command = vec![redis::Token::String("PING".to_string())];
        assert_eq!(handle_command(&command), "+PONG\r\n");
    }

    #[test]
    fn test_handle_command_echo() {
        let command = vec![
            redis::Token::String("ECHO".to_string()),
            redis::Token::String("hello".to_string()),
            redis::Token::String("world".to_string()),
        ];
        assert_eq!(handle_command(&command), "+hello world\r\n");
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
