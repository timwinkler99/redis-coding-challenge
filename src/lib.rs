use core::panic;
use std::result::Result;

const CLRF: &str = "\r\n";

pub fn parse_input(input: &str) -> Result<String, String> {
    match input.chars().next() {
        Some('$') => Ok(parse_bulk_string(input)),
        Some('*') => Ok(parse_array(input).join(",")),
        _ => Err("Invalid data type".to_string()),
    }
}

fn parse_bulk_string(input: &str) -> String {
    if input == "$-1\r\n" {
        return "null".to_string();
    }
    let parts: Vec<&str> = input.split(CLRF).collect();
    parts[1].to_string()
}

fn parse_array(input: &str) -> Vec<String> {
    if input == "*-1\r\n" {
        return vec!["null".to_string()];
    }

    let mut elements = Vec::new();
    let len_end = input.find(CLRF).unwrap();
    let len: usize = input[1..len_end].parse().unwrap();
    let mut rest = &input[len_end + 2..]; // array elements

    for _ in 0..len {
        match rest.chars().next() {
            Some('$') => {
                // bulk string starts with data type, len, CLRF, string, CLRF: $3\r\nfoo\r\n
                // find index of first \r\n to find index of second
                let start_first = rest.find(CLRF).unwrap();
                let start_second =
                    rest[start_first + 2..].find(CLRF).unwrap() + start_first + CLRF.len();
                println!("input: {:?}", &rest[..start_second + CLRF.len()]);
                let element = parse_bulk_string(&rest[..start_second + CLRF.len()]);
                elements.push(element);
                rest = &rest[start_second + CLRF.len()..];
            }
            _ => panic!(""),
        }
    }
    elements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bulk_string() {
        assert_eq!(parse_bulk_string("$6\r\nfoobar\r\n"), "foobar");
        assert_eq!(parse_bulk_string("$-1\r\n"), "null");
    }

    #[test]
    fn test_parse_array() {
        assert_eq!(
            parse_array("*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"),
            vec!["foo", "bar"]
        );
        assert_eq!(parse_array("*-1\r\n"), vec!["null"]);
    }
}
