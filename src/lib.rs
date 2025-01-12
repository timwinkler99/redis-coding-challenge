use std::io::{self, BufRead};

#[derive(Debug, PartialEq)]
pub enum Token {
    String(String),
    Integer(i64),
    Error(String),
    Null,
}

const CR: u8 = b'\r';
const LF: u8 = b'\n';

pub fn parse<R: BufRead>(input: &mut R) -> io::Result<Vec<Token>> {
    let mut first_byte = [0; 1];
    input.read_exact(&mut first_byte)?;

    match first_byte[0] {
        b'+' => {
            let element = read_element(input)?;
            Ok(vec![Token::String(element)])
        }
        b'-' => {
            let element = read_element(input)?;
            Ok(vec![Token::Error(element)])
        }
        b':' => {
            let element = read_element(input)?;
            let number = element
                .parse::<i64>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid integer"))?;
            Ok(vec![Token::Integer(number)])
        }
        b'$' => {
            let length = read_element(input)?
                .parse::<i32>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid integer"))?;
            if length == -1 {
                Ok(vec![Token::Null])
            } else {
                let element = read_element(input)?;
                Ok(vec![Token::String(element)])
            }
        }
        b'*' => {
            let number_of_elements = read_element(input)?.parse::<usize>().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid number of elements")
            })?;
            let mut elements = Vec::new();
            for _ in 0..number_of_elements {
                elements.extend(parse(input)?);
            }
            Ok(elements)
        }
        _ => Ok(Vec::new()),
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
    use io::BufReader;

    use super::*;

    #[test]
    fn test_parse_array() {
        let data = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let mut reader = BufReader::new(&data[..]);

        let result = parse(&mut reader).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Token::String("foo".to_string()));
        assert_eq!(result[1], Token::String("bar".to_string()));
    }
}
