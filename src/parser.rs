use std::{collections::BTreeMap, error, fmt, iter::Peekable, str::Chars};

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

    if let Some(unexpected_char) = rest.peek().map(|c| c.to_owned()) {
        fail(format!(
            "Unexpected character: {unexpected_char}, {} chars remaining",
            rest.count()
        ))
    } else {
        Ok(parsed)
    }
}

fn fail<T>(message: impl Into<String>) -> Result<T, JsonParseError> {
    Err(JsonParseError {
        message: message.into(),
    })
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

fn peek_or_fail(rest: &mut Peekable<Chars>) -> Result<char, JsonParseError> {
    match rest.peek() {
        Some(c) => Ok(*c),
        None => fail("Unexpected end of input"),
    }
}

fn next_or_fail(rest: &mut Peekable<Chars>) -> Result<char, JsonParseError> {
    match rest.next() {
        Some(c) => Ok(c),
        None => fail("Unexpected end of input"),
    }
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

    let value = match peek_or_fail(rest)? {
        'n' => consume(rest, "null", Json::Null),
        't' => consume(rest, "true", Json::Boolean(true)),
        'f' => consume(rest, "false", Json::Boolean(false)),
        '-' | '0'..='9' => parse_number(rest),
        '"' => parse_string_value(rest),
        '[' => parse_array(rest),
        '{' => parse_object(rest),
        unexpected_char => fail(format!("Unexpected character: {unexpected_char}")),
    };

    skip_whitespace(rest);

    value
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
            return fail("Missing digits after point in number");
        }
        while advance_if(|c| "0123456789".contains(c)) {}
    }

    if advance_if(|c| c == 'e' || c == 'E') {
        advance_if(|c| c == '-' || c == '+');

        if !advance_if(|c| "0123456789".contains(c)) {
            return fail("Missing digits after exponent in number");
        }
        while advance_if(|c| "0123456789".contains(c)) {}
    }

    return match number_string.parse::<f64>() {
        Ok(number) => Ok(Json::Number(number)),
        Err(_) => fail(format!("Expected number, found: {number_string}")),
    };
}

fn parse_string_value(rest: &mut Peekable<Chars>) -> Result<Json, JsonParseError> {
    let parsed_string = parse_string(rest)?;

    Ok(Json::String(parsed_string))
}

fn parse_string(rest: &mut Peekable<Chars>) -> Result<String, JsonParseError> {
    let mut parsed_string = String::new();

    let first_char = next_or_fail(rest)?;
    if first_char != '"' {
        return fail(format!("Expected a string, found '{}'", first_char));
    }

    loop {
        match next_or_fail(rest)? {
            '"' => break,
            '\\' => parsed_string.push_str(&parse_string_escape_char(rest)?),
            regular_char => parsed_string.push(regular_char),
        }
    }

    Ok(parsed_string)
}

fn parse_string_escape_char(rest: &mut Peekable<Chars>) -> Result<String, JsonParseError> {
    let mut codepoints = Vec::new();

    loop {
        codepoints.push(parse_string_escape_as_codepoint(rest)?);

        if peek_or_fail(rest)? == '\\' {
            next_or_fail(rest)?;
            continue;
        } else {
            break;
        }
    }

    let mut decoded = String::new();

    for decoding_result in char::decode_utf16(codepoints) {
        match decoding_result {
            Ok(decoded_char) => decoded.push(decoded_char),
            Err(_) => return fail("Unpaired UTF-16 surrogate in string"),
        }
    }

    Ok(decoded)
}

fn parse_string_escape_as_codepoint(rest: &mut Peekable<Chars>) -> Result<u16, JsonParseError> {
    match next_or_fail(rest)? {
        '"' => Ok(34),
        '\\' => Ok(92),
        '/' => Ok(47),
        'b' => Ok(8),
        'f' => Ok(12),
        'n' => Ok(10),
        'r' => Ok(13),
        't' => Ok(9),
        'u' => parse_utf16_hex_escaped_codepoint(rest),
        _ => fail("Invalid escape sequence in string"),
    }
}

fn parse_utf16_hex_escaped_codepoint(rest: &mut Peekable<Chars>) -> Result<u16, JsonParseError> {
    let mut hex_digits = String::new();

    for _ in 0..4 {
        let next_char = next_or_fail(rest)?;
        if next_char.is_ascii_hexdigit() {
            hex_digits.push(next_char);
        } else {
            return fail("Invalid hex digit in unicode escape sequence");
        }
    }

    u16::from_str_radix(&hex_digits, 16).or_else(|_| fail("Invalid hex codepoint"))
}

fn parse_array(rest: &mut Peekable<Chars>) -> Result<Json, JsonParseError> {
    if next_or_fail(rest)? != '[' {
        return fail("Expected array");
    }

    skip_whitespace(rest);

    let mut items = Vec::new();

    if peek_or_fail(rest)? == ']' {
        rest.next();
    } else {
        loop {
            let item = parse_value(rest)?;
            items.push(item);

            match next_or_fail(rest)? {
                ']' => break,
                ',' => continue,
                unexpected_char => {
                    return fail(format!("Expected ',' or ']', found '{unexpected_char}'"))
                }
            }
        }
    }

    Ok(Json::Array(items))
}

fn parse_object(rest: &mut Peekable<Chars>) -> Result<Json, JsonParseError> {
    if next_or_fail(rest)? != '{' {
        return fail("Expected array");
    }

    skip_whitespace(rest);

    let mut properties = BTreeMap::new();

    if peek_or_fail(rest)? == '}' {
        rest.next();
    } else {
        loop {
            let key = parse_string(rest)?;
            skip_whitespace(rest);

            if next_or_fail(rest)? != ':' {
                return fail("Missing colon after object key");
            }

            let value = parse_value(rest)?;

            properties.insert(key, value);

            match next_or_fail(rest)? {
                '}' => break,
                ',' => {
                    skip_whitespace(rest);
                    continue;
                }
                unexpected_char => {
                    return fail(format!("Expected ',' or '}}', found '{unexpected_char}'"))
                }
            }
        }
    }

    Ok(Json::Object(properties))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

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
    fn it_rejects_typos() {
        assert!(parse("nul").is_err());
        assert!(parse("folse").is_err());
        assert!(parse("flase").is_err());
        assert!(parse("truee").is_err());
        assert!(parse("rue").is_err());
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
    fn it_rejects_invalid_numbers() {
        assert!(parse(".34").is_err());
        assert!(parse("145.65.2").is_err());
        assert!(parse("+23").is_err());
        assert!(parse("--23").is_err());
        assert!(parse("-hello").is_err());
        assert!(parse("00").is_err());
        assert!(parse("67.").is_err());
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
    fn it_parses_an_empty_string() {
        assert_eq!(parse(r#""""#), Ok(Json::String("".to_owned())));
    }

    #[test]
    fn it_parses_a_regular_ascii_string() {
        assert_eq!(
            parse(r#""this is a string.""#),
            Ok(Json::String("this is a string.".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_unicode_string() {
        assert_eq!(
            parse(r#""😃 or 🙁?""#),
            Ok(Json::String("😃 or 🙁?".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_string_with_an_escaped_double_quote() {
        assert_eq!(
            parse(r#""double \" quote""#),
            Ok(Json::String("double \" quote".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_string_with_an_escaped_backslash() {
        assert_eq!(
            parse(r#""back \\ slash""#),
            Ok(Json::String("back \\ slash".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_string_with_an_escaped_solidus() {
        assert_eq!(
            parse(r#""forward \/ slash""#),
            Ok(Json::String("forward / slash".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_string_with_an_escaped_backspace() {
        assert_eq!(
            parse(r#""back \b space""#),
            Ok(Json::String("back \x08 space".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_string_with_an_escaped_formfeed() {
        assert_eq!(
            parse(r#""form \f feed""#),
            Ok(Json::String("form \x0C feed".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_string_with_an_escaped_linefeed() {
        assert_eq!(
            parse(r#""line \n feed""#),
            Ok(Json::String("line \n feed".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_string_with_an_escaped_carriage_return() {
        assert_eq!(
            parse(r#""carriage \r return""#),
            Ok(Json::String("carriage \r return".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_string_with_an_escaped_tab() {
        assert_eq!(
            parse(r#""horizontal \t tab""#),
            Ok(Json::String("horizontal \t tab".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_string_with_a_unicode_escape_sequence() {
        assert_eq!(
            parse(r#""unicode \u0041 literal""#),
            Ok(Json::String("unicode A literal".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_string_with_contiguous_non_surrogate_unicode_escape_sequences() {
        assert_eq!(
            parse(r#""\u0021\u003D""#),
            Ok(Json::String("!=".to_owned()))
        );
    }

    #[test]
    fn it_parses_a_string_with_an_escaped_unicode_surrogate_pair() {
        assert_eq!(
            parse(r#""\uD83D\uDE02""#),
            Ok(Json::String("😂".to_owned()))
        );
    }

    #[test]
    fn it_rejects_invalid_string_escape_sequences() {
        assert!(parse(r#""\""#).is_err());
        assert!(parse(r#""\d""#).is_err());
        assert!(parse(r#""\0""#).is_err());
        assert!(parse(r#""\ n""#).is_err());
        assert!(parse(r#""\u001""#).is_err());
        assert!(parse(r#""\u ""#).is_err());
    }

    #[test]
    fn it_parses_an_empty_array() {
        assert_eq!(parse("[]"), Ok(Json::Array(vec!())));
        assert_eq!(parse(" [  ] "), Ok(Json::Array(vec!())));
    }

    #[test]
    fn it_parses_an_array_with_one_item() {
        assert_eq!(parse("[123]"), Ok(Json::Array(vec!(Json::Number(123.0)))));
    }

    #[test]
    fn it_parses_an_array_with_multiple_items() {
        assert_eq!(
            parse("[null,true,false]"),
            Ok(Json::Array(vec!(
                Json::Null,
                Json::Boolean(true),
                Json::Boolean(false)
            )))
        );
    }

    #[test]
    fn it_parses_an_array_with_multiple_items_surrounded_by_whitespace() {
        assert_eq!(
            parse(" [ 1 ,\t2 ,\n3\r]  \n"),
            Ok(Json::Array(vec!(
                Json::Number(1.0),
                Json::Number(2.0),
                Json::Number(3.0)
            )))
        );
    }

    #[test]
    fn it_parses_a_nested_array() {
        assert_eq!(
            parse("[[], [[], [null]]]"),
            Ok(Json::Array(vec!(
                Json::Array(vec!()),
                Json::Array(vec!(Json::Array(vec!()), Json::Array(vec!(Json::Null))))
            )))
        );
    }

    #[test]
    fn it_rejects_an_invalid_array() {
        assert!(parse("[").is_err());
        assert!(parse("[]]").is_err());
        assert!(parse("[,null]").is_err());
        assert!(parse("[true,]").is_err());
        assert!(parse(",[]").is_err());
    }

    #[test]
    fn it_parses_an_empty_object() {
        assert_eq!(parse("{}"), Ok(Json::Object(BTreeMap::new())));
        assert_eq!(parse("\n{ } "), Ok(Json::Object(BTreeMap::new())));
    }

    #[test]
    fn it_parses_an_object_with_a_single_property() {
        assert_eq!(
            parse(r#"{"key":"value"}"#),
            Ok(Json::Object(BTreeMap::from([(
                "key".to_owned(),
                Json::String("value".to_owned())
            )])))
        );
        assert_eq!(
            parse(" { \"key\"\t:\n\"value\"   }\n"),
            Ok(Json::Object(BTreeMap::from([(
                "key".to_owned(),
                Json::String("value".to_owned())
            )])))
        );
        assert_eq!(
            parse(r#"{"the count": 123}"#),
            Ok(Json::Object(BTreeMap::from([(
                "the count".to_owned(),
                Json::Number(123.0)
            )])))
        );
    }

    #[test]
    fn it_parses_an_object_with_multiple_properties() {
        assert_eq!(
            parse(r#"{"name": "Andrew", "age": 63}"#),
            Ok(Json::Object(BTreeMap::from([
                ("name".to_owned(), Json::String("Andrew".to_owned())),
                ("age".to_owned(), Json::Number(63.0))
            ])))
        );
        assert_eq!(
            parse(r#"{ "prop": null ,"😃" : true, " 123\n " :false }"#),
            Ok(Json::Object(BTreeMap::from([
                ("prop".to_owned(), Json::Null),
                ("😃".to_owned(), Json::Boolean(true)),
                (" 123\n ".to_owned(), Json::Boolean(false))
            ])))
        );
    }

    #[test]
    fn it_parses_a_nested_object() {
        assert_eq!(
            parse(r#"{"middle": {"inner": {}}}"#),
            Ok(Json::Object(BTreeMap::from([(
                "middle".to_owned(),
                Json::Object(BTreeMap::from([(
                    "inner".to_owned(),
                    Json::Object(BTreeMap::from([]))
                )]))
            ),])))
        );
    }

    #[test]
    fn it_rejects_an_invalid_object() {
        assert!(parse("{").is_err());
        assert!(parse("{}}").is_err());
        assert!(parse("{unquoted: true}").is_err());
        assert!(parse(r#"{trailing: "comma",}"#).is_err());
        assert!(parse(r#"{"no value"}"#).is_err());
        assert!(parse(r#"{"missing": "comma" "between": "properties"}"#).is_err());
    }
}
