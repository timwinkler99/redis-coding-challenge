use std::{result::Result, usize};

pub fn parse_input(input: &str) -> Result<String, String> {
    match input.chars().next() {
        Some('$') => parse_bulk_string(input).map_err(|e| format!("Bulk string error: {}", e)),
        Some('*') => parse_array(input)
            .map(|arr| arr.join(","))
            .map_err(|e| format!("Array error: {}", e)),
        _ => Err("Invalid data type".to_string()),
    }
}

fn parse_bulk_string(input: &str) -> Result<String, &'static str> {
    if input == "$-1\r\n" {
        return Ok("null".to_string());
    }

    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes[0] != b'$' {
        return Err("Must start with $");
    }

    let mut pos = 1;

    // Find length end
    let len_end = match bytes[pos..].iter().position(|&b| b == b'\r') {
        Some(end) => end,
        None => return Err("Missing CRLF after length"),
    };

    // Parse lenght
    let length = std::str::from_utf8(&bytes[pos..pos + len_end])
        .map_err(|_| "Invalid lenght encoding")?
        .parse::<usize>()
        .map_err(|_| "Invalid lenght")?;

    pos = pos + len_end + 2; // Skip length and CRLF

    // Validate remaining length
    if pos + length + 2 > bytes.len() {
        return Err("Invalid content length");
    }

    // Extract content
    let content =
        std::str::from_utf8(&bytes[pos..pos + length]).map_err(|_| "Invalid UTF-8 in content")?;

    // Verify final CRLF
    if &bytes[pos + length..pos + length + 2] != b"\r\n" {
        return Err("Missing final CRLF");
    }

    Ok(content.to_string())
}

fn parse_array(input: &str) -> Result<Vec<String>, String> {
    if input == "*-1\r\n" {
        return Ok(vec!["null".to_string()]);
    }

    // Fast path for empty arrays
    if input == "*0\r\n" {
        return Ok(Vec::new());
    }

    let bytes = input.as_bytes();
    let mut pos = 1; // Skip initial '*'

    // Parse array length
    let len = match bytes[pos..].iter().position(|&b| b == b'\r') {
        Some(end) => {
            let len_str = std::str::from_utf8(&bytes[pos..pos + end])
                .map_err(|_| "Invalid length encoding")?;
            pos = pos + end + 2; // Skip CRLF
            len_str.parse::<usize>().map_err(|_| "Invalid length")?
        }
        None => return Err("Missing CRLF after length".to_string()),
    };

    let mut result = Vec::with_capacity(len);

    // Extract bulk strings
    for _ in 0..len {
        if pos >= bytes.len() || bytes[pos] != b'$' {
            return Err("Invalid input: array elements must start with $".to_string());
        }
        pos += 1;

        // Find length of bulk string
        let len_end = bytes[pos..]
            .iter()
            .position(|&b| b == b'\r')
            .ok_or("Missing CRLF after bulk string length")?;
        let bulk_len = std::str::from_utf8(&bytes[pos..pos + len_end])
            .map_err(|_| "Invalid bulk string length encoding")?
            .parse::<usize>()
            .map_err(|_| "Invalid bulk string length")?;
        pos += len_end + 2; // Skip CRLF

        // Extract bulk string content
        if pos + bulk_len + 2 > bytes.len() {
            return Err("Incomplete bulk string".to_string());
        }
        let content = std::str::from_utf8(&bytes[pos..pos + bulk_len])
            .map_err(|_| "Invalid UTF-8 in bulk string")?;
        result.push(content.to_string());
        pos += bulk_len + 2; // Skip content and CRLF
    }

    Ok(result)
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

        // Missing CRLF suffix
        assert!(parse_bulk_string("$6\r\nfoobar").is_err());

        // Missing content separator CRLF
        assert!(parse_bulk_string("$6foobar\r\n").is_err());
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
