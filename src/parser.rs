use std::{cmp::min, error, fmt};

use crate::ast::Json;

#[derive(Debug, PartialEq)]
pub struct JsonParseError {
    pub message: String,
}

impl fmt::Display for JsonParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("ERROR: Invalid JSON - {}", self.message))
    }
}

impl error::Error for JsonParseError {}

type Parsed<'a> = Result<(Json, &'a str), JsonParseError>;

pub fn parse(json: &str) -> Result<Json, JsonParseError> {
    let (parsed, rest) = parse_value(json)?;
    if let Some(unexpected_char) = rest.trim_start().chars().nth(0) {
        fail(format!("Unexpected character: {unexpected_char}"))
    } else {
        Ok(parsed)
    }
}

fn fail<T>(message: String) -> Result<T, JsonParseError> {
    Err(JsonParseError { message: message })
}

fn consume<'a>(rest: &'a str, literal: &str, json_value: Json) -> Parsed<'a> {
    if let Some(rest) = rest.strip_prefix(literal) {
        Ok((json_value, rest))
    } else {
        fail(format!(
            "Expected '{}', but found '{}'",
            literal,
            &rest[..min(literal.len(), rest.len())]
        ))
    }
}

fn parse_value(mut rest: &str) -> Parsed {
    rest = rest.trim_start();
    let next_char = rest.chars().nth(0);
    match next_char {
        Some('n') => consume(rest, "null", Json::Null),
        Some('t') => consume(rest, "true", Json::Boolean(true)),
        Some('f') => consume(rest, "false", Json::Boolean(false)),
        Some('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9') => parse_number(rest),
        Some('"') => parse_string(rest),
        Some('[') => parse_array(rest),
        Some('{') => parse_object(rest),
        Some(unexpected_char) => fail(format!("Unexpected character: {unexpected_char}")),
        None => fail(format!("Unexpected end of input")),
    }
}

// TODO support non-standard numbers.
fn parse_number(rest: &str) -> Parsed {
    let mut number_string = String::new();

    let mut seen_decimal_point = false;

    for next_char in rest.chars() {
        if next_char.is_ascii_digit() {
            number_string.push(next_char);
        } else if next_char == '.' && !seen_decimal_point {
            number_string.push(next_char);
            seen_decimal_point = true;
        } else {
            break;
        }
    }

    return match number_string.parse::<f64>() {
        Ok(number) => Ok((Json::Number(number), &rest[number_string.len()..])),
        Err(_) => fail(format!("Expected number, found: {number_string}")),
    };
}

fn parse_string(rest: &str) -> Parsed {
    fail("String parsing not implemented".to_owned())
}

fn parse_array(rest: &str) -> Parsed {
    fail("Array parsing not implemented".to_owned())
}

fn parse_object(rest: &str) -> Parsed {
    fail("Object parsing not implemented".to_owned())
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::ast::Json;

    #[test]
    fn it_parses_null() {
        assert_eq!(parse("null"), Ok(Json::Null));
    }

    #[test]
    fn it_parses_true() {
        assert_eq!(parse("true"), Ok(Json::Boolean(true)));
    }

    #[test]
    fn it_parses_false() {
        assert_eq!(parse("false"), Ok(Json::Boolean(false)));
    }

    #[test]
    fn it_parses_an_integer() {
        assert_eq!(parse("123"), Ok(Json::Number(123.0)));
    }

    #[test]
    fn it_parses_a_decimal() {
        assert_eq!(parse("123.456"), Ok(Json::Number(123.456)));
    }

    #[test]
    fn it_parses_zero() {
        assert_eq!(parse("0"), Ok(Json::Number(0.0)));
    }
}
