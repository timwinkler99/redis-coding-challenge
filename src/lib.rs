use std::io::{self, BufRead};

#[derive(Debug, PartialEq)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),    // None for NULL bulk string
    Array(Option<Vec<RespValue>>), // None for NULL array
}

impl RespValue {
    pub fn serialize(&self) -> String {
        match &self {
            RespValue::SimpleString(s) => format!("+{}\r\n", s),
            RespValue::Error(s) => format!("-{}\r\n", s),
            RespValue::Integer(i) => format!(":{}\r\n", i),
            RespValue::BulkString(Some(s)) => format!("${}\r\n{}\r\n", s.len(), s),
            RespValue::BulkString(None) => "$-1\r\n".to_string(),
            RespValue::Array(Some(values)) => {
                let serialized_elements: String = values.iter().map(|v| v.serialize()).collect();
                format!("*{}\r\n{}", values.len(), serialized_elements)
            }
            RespValue::Array(None) => "*-1\r\n".to_string(),
        }
    }
}

const CR: u8 = b'\r';
const LF: u8 = b'\n';

pub fn parse<R: BufRead>(input: &mut R) -> io::Result<RespValue> {
    let mut first_byte = [0; 1];
    input.read_exact(&mut first_byte)?;

    match first_byte[0] {
        b'+' => {
            let element = read_element(input)?;
            Ok(RespValue::SimpleString(element))
        }
        b'-' => {
            let element = read_element(input)?;
            Ok(RespValue::Error(element))
        }
        b':' => {
            let element = read_element(input)?;
            let number = element
                .parse::<i64>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid integer"))?;
            Ok(RespValue::Integer(number))
        }
        b'$' => {
            let length = read_element(input)?
                .parse::<i32>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid integer"))?;
            if length == -1 {
                Ok(RespValue::BulkString(None))
            } else {
                let element = read_element(input)?;
                Ok(RespValue::BulkString(Some(element)))
            }
        }
        b'*' => {
            let number_of_elements = read_element(input)?.parse::<i32>().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid number of elements")
            })?;
            if number_of_elements == -1 {
                Ok(RespValue::Array(None))
            } else {
                let mut elements = Vec::new();
                for _ in 0..number_of_elements {
                    elements.push(parse(input)?);
                }
                Ok(RespValue::Array(Some(elements)))
            }
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid data type",
        )),
    }
}

fn read_element<R: BufRead>(input: &mut R) -> io::Result<String> {
    let mut buffer = Vec::with_capacity(2048);
    let mut element = [0; 1];

    while input.read(&mut element)? > 0 {
        if element[0] == CR {
            break;
        }
        buffer.push(element[0]);
    }

    input.read_exact(&mut element)?;
    if element[0] != LF {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unexpected end of token",
        ));
    }

    Ok(String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_simple_string() {
        let data = b"+OK\r\n";
        let mut reader = BufReader::new(&data[..]);

        let result = parse(&mut reader).unwrap();

        assert_eq!(result, RespValue::SimpleString("OK".to_string()));
    }

    #[test]
    fn test_error() {
        let data = b"-Error message\r\n";
        let mut reader = BufReader::new(&data[..]);

        let result = parse(&mut reader).unwrap();

        assert_eq!(result, RespValue::Error("Error message".to_string()));
    }

    #[test]
    fn test_integer() {
        let data = b":100\r\n";
        let mut reader = BufReader::new(&data[..]);

        let result = parse(&mut reader).unwrap();

        assert_eq!(result, RespValue::Integer(100));
    }

    #[test]
    fn test_bulk_string() {
        let data = b"$6\r\nfoobar\r\n";
        let mut reader = BufReader::new(&data[..]);

        let result = parse(&mut reader).unwrap();

        assert_eq!(result, RespValue::BulkString(Some("foobar".to_string())));
    }

    #[test]
    fn test_bulk_string_null() {
        let data = b"$-1\r\n";
        let mut reader = BufReader::new(&data[..]);

        let result = parse(&mut reader).unwrap();

        assert_eq!(result, RespValue::BulkString(None));
    }

    #[test]
    fn test_parse_array() {
        let data = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let mut reader = BufReader::new(&data[..]);

        let result = parse(&mut reader).unwrap();

        assert_eq!(
            result,
            RespValue::Array(Some(vec![
                RespValue::BulkString(Some("foo".to_string())),
                RespValue::BulkString(Some("bar".to_string()))
            ]))
        );
    }

    #[test]
    fn test_parse_array_null() {
        let data = b"*-1\r\n";
        let mut reader = BufReader::new(&data[..]);

        let result = parse(&mut reader).unwrap();

        assert_eq!(result, RespValue::Array(None));
    }
}
