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
            if let Some(redis::Token::String(ref s)) = command.get(0) {
                if s == "PING" {
                    stream.write_all(b"+PONG\r\n").unwrap();
                }
            }
        }
        Err(e) => eprintln!("Failed to parse command: {}", e),
    }
}
