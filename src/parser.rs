use std::{error, fmt, iter::Peekable, str::Chars};

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

pub fn parse(json: &str) -> Result<Json, JsonParseError> {
    let mut rest = json.chars().peekable();
    let parsed = parse_value(&mut rest)?;
    skip_whitespace(&mut rest);

    if let Some(unexpected_char) = rest.peek() {
        fail(format!("Unexpected character: {unexpected_char}"))
    } else {
        Ok(parsed)
    }
}

fn fail<T>(message: String) -> Result<T, JsonParseError> {
    Err(JsonParseError { message: message })
}

fn consume(
    rest: &mut Peekable<Chars>,
    literal: &str,
    json_value: Json,
) -> Result<Json, JsonParseError> {
    for expected_char in literal.chars() {
        match rest.next() {
            None => return fail("Unexpected end of input".to_owned()),
            Some(actual_char) if actual_char == expected_char => continue,
            Some(actual_char) => {
                return fail(format!(
                    "Expected '{expected_char}', but found '{actual_char}'",
                ))
            }
        }
    }
    Ok(json_value)
}

fn skip_whitespace(rest: &mut Peekable<Chars>) {
    while let Some(next_char) = rest.peek() {
        if " \n\r\t".contains(*next_char) {
            rest.next();
            continue;
        }
        break;
    }
}

fn parse_value(rest: &mut Peekable<Chars>) -> Result<Json, JsonParseError> {
    skip_whitespace(rest);

    match rest.peek() {
        Some('n') => consume(rest, "null", Json::Null),
        Some('t') => consume(rest, "true", Json::Boolean(true)),
        Some('f') => consume(rest, "false", Json::Boolean(false)),
        Some('-' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9') => parse_number(rest),
        Some('"') => parse_string(rest),
        Some('[') => parse_array(rest),
        Some('{') => parse_object(rest),
        Some(unexpected_char) => fail(format!("Unexpected character: {unexpected_char}")),
        None => fail(format!("Unexpected end of input")),
    }
}

fn parse_number(rest: &mut Peekable<Chars>) -> Result<Json, JsonParseError> {
    let mut number_string = String::new();

    let mut advance_if = |predicate: fn(char) -> bool| -> bool {
        match rest.next_if(|arg0: &char| predicate(*arg0)) {
            Some(next_char) => {
                number_string.push(next_char);
                true
            }
            None => false,
        }
    };

    advance_if(|c| c == '-');

    if !advance_if(|c| c == '0') {
        if !advance_if(|c| "123456789".contains(c)) {
            return fail(format!(
                "Unexpected character in number: {}",
                rest.peek().unwrap()
            ));
        }

        while advance_if(|c| "0123456789".contains(c)) {}
    }

    if advance_if(|c| c == '.') {
        if !advance_if(|c| "0123456789".contains(c)) {
            return fail("Missing digits after point in number".to_owned());
        }
        while advance_if(|c| "0123456789".contains(c)) {}
    }

    if advance_if(|c| c == 'e' || c == 'E') {
        advance_if(|c| c == '-' || c == '+');

        if !advance_if(|c| "0123456789".contains(c)) {
            return fail("Missing digits after exponent in number".to_owned());
        }
        while advance_if(|c| "0123456789".contains(c)) {}
    }

    return match number_string.parse::<f64>() {
        Ok(number) => Ok(Json::Number(number)),
        Err(_) => fail(format!("Expected number, found: {number_string}")),
    };
}

fn parse_string(rest: &mut Peekable<Chars>) -> Result<Json, JsonParseError> {
    rest.peek();
    fail("String parsing not implemented".to_owned())
}

fn parse_array(rest: &mut Peekable<Chars>) -> Result<Json, JsonParseError> {
    rest.peek();
    fail("Array parsing not implemented".to_owned())
}

fn parse_object(rest: &mut Peekable<Chars>) -> Result<Json, JsonParseError> {
    rest.peek();
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
        assert_eq!(parse("120.056"), Ok(Json::Number(120.056)));
    }

    #[test]
    fn it_parses_zero() {
        assert_eq!(parse("0"), Ok(Json::Number(0.0)));
    }

    #[test]
    fn it_parses_negative_numbers() {
        assert_eq!(parse("-123"), Ok(Json::Number(-123.0)));
    }

    #[test]
    fn it_parses_numbers_with_exponents() {
        assert_eq!(parse("10e23"), Ok(Json::Number(10.0e23)));
        assert_eq!(parse("10E23"), Ok(Json::Number(10.0e23)));
        assert_eq!(parse("10e+23"), Ok(Json::Number(10.0e23)));
        assert_eq!(parse("10e-23"), Ok(Json::Number(10.0e-23)));
    }

    #[test]
    fn it_parses_inputs_with_leading_whitespace() {
        assert_eq!(parse("   null"), Ok(Json::Null));
        assert_eq!(parse("\tnull"), Ok(Json::Null));
        assert_eq!(parse("\nnull"), Ok(Json::Null));
        assert_eq!(parse("\rnull"), Ok(Json::Null));
    }

    #[test]
    fn it_parses_inputs_with_trailing_whitespace() {
        assert_eq!(parse("null   "), Ok(Json::Null));
        assert_eq!(parse("null\t"), Ok(Json::Null));
        assert_eq!(parse("null\n"), Ok(Json::Null));
        assert_eq!(parse("null\r"), Ok(Json::Null));
    }

    #[test]
    fn it_rejects_typos() {
        assert!(parse("nul").is_err());
        assert!(parse("folse").is_err());
        assert!(parse("flase").is_err());
        assert!(parse("truee").is_err());
        assert!(parse("rue").is_err());
    }

    #[test]
    fn it_rejects_invalid_numbers() {
        assert!(parse(".34").is_err());
        assert!(parse("145.65.2").is_err());
        assert!(parse("+23").is_err());
        assert!(parse("--23").is_err());
        assert!(parse("-hello").is_err());
        assert!(parse("00").is_err());
        assert!(parse("67.").is_err());
    }
}
