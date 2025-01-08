use std::{result::Result, usize};

const CLRF: &str = "\r\n";

pub fn parse_input(input: &str) -> Result<String, String> {
    match input.chars().next() {
        Some('$') => parse_bulk_string(input).map_err(|e| format!("Bulk string error: {}", e)),
        Some('*') => parse_array(input)
            .map(|arr| arr.join(","))
            .map_err(|e| format!("Array error: {}", e)),
        _ => Err("Invalid data type".to_string()),
    }
}

fn parse_bulk_string(input: &str) -> Result<String, String> {
    if input == "$-1\r\n" {
        return Ok("null".to_string());
    }

    input
        .strip_prefix('$')
        .ok_or("Must start with $")?
        .strip_suffix(CLRF)
        .ok_or("Must end with CRLF")?
        .split_once(CLRF)
        .map(|(_, content)| content.to_string())
        .ok_or("Missing content separator CRLF".to_string())
}

fn parse_array(input: &str) -> Result<Vec<String>, String> {
    if input == "*-1\r\n" {
        return Ok(vec!["null".to_string()]);
    }

    let (len, rest) = input[1..]
        .split_once(CLRF)
        .map(|(len, rest)| (len.parse::<usize>().unwrap(), rest.to_string()))
        .unwrap();

    let extract_bulk_string = |rest: String| -> Result<(String, String), String> {
        let start_first = rest.find(CLRF).ok_or("Missing first CRLF")?;
        let start_second = rest[start_first + 2..]
            .find(CLRF)
            .map(|pos| pos + start_first + CLRF.len())
            .ok_or("Missing second CRLF")?;

        let element_str = &rest[..start_second + CLRF.len()];
        let next_rest = rest[start_second + CLRF.len()..].to_string();

        parse_bulk_string(element_str).map(|element| (element, next_rest))
    };

    (0..len)
        .try_fold(
            (Vec::new(), rest),
            |(mut acc, current_rest), _| -> Result<(Vec<String>, String), String> {
                match current_rest.chars().next() {
                    Some('$') => {
                        let (element, next_rest) = extract_bulk_string(current_rest)?;
                        acc.push(element);
                        Ok((acc, next_rest))
                    }
                    _ => Err("Invalid input: array elements must start with $".to_string()),
                }
            },
        )
        .map(|(vec, _)| vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bulk_string() {
        assert_eq!(parse_bulk_string("$6\r\nfoobar\r\n").unwrap(), "foobar");
        assert_eq!(parse_bulk_string("$-1\r\n").unwrap(), ("null"));

        // Error cases
        // Missing $ prefix
        assert!(parse_bulk_string("6\r\nfoobar\r\n").is_err());
        assert_eq!(
            parse_bulk_string("6\r\nfoobar\r\n").unwrap_err(),
            "Must start with $"
        );

        // Missing CRLF suffix
        assert!(parse_bulk_string("$6\r\nfoobar").is_err());
        assert_eq!(
            parse_bulk_string("$6\r\nfoobar").unwrap_err(),
            "Must end with CRLF"
        );

        // Missing content separator CRLF
        assert!(parse_bulk_string("$6foobar\r\n").is_err());
        assert_eq!(
            parse_bulk_string("$6foobar\r\n").unwrap_err(),
            "Missing content separator CRLF"
        );
    }

    #[test]
    fn test_parse_array() {
        assert_eq!(
            parse_array("*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n").unwrap(),
            vec!["foo", "bar"]
        );
        assert_eq!(parse_array("*-1\r\n").unwrap(), vec!["null"]);
    }
}
