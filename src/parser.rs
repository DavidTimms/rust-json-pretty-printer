use std::{cmp::min, error, fmt};

use crate::ast::Json;

#[derive(Debug)]
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

fn parse_number(rest: &str) -> Parsed {
    fail("Number parsing not implemented".to_owned())
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
