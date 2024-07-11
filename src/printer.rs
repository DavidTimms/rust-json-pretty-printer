use std::{
    collections::BTreeMap,
    fmt::{self, Write},
};

use crate::ast::Json;

impl fmt::Display for Json {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        display_json(self, formatter, 2, 0)
    }
}

pub fn json_to_string(value: &Json, indent: u64) -> String {
    let mut output = String::new();
    display_json(value, &mut output, indent, 0).expect("Failed to write JSON to string");
    output
}

fn display_json<W: Write>(
    value: &Json,
    output: &mut W,
    indent: u64,
    level: u64,
) -> Result<(), fmt::Error> {
    match value {
        Json::Null => output.write_str("null"),
        Json::Boolean(true) => output.write_str("true"),
        Json::Boolean(false) => output.write_str("false"),
        Json::String(string) => output.write_str(&display_json_string(string)),
        Json::Number(number) => output.write_fmt(format_args!("{number}")),
        Json::Array(array) => display_json_array(array, output, indent, level),
        Json::Object(object) => display_json_object(object, output, indent, level),
    }
}

fn display_json_string(string: &str) -> String {
    let mut escaped = String::new();

    escaped.push('"');

    for c in string.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{C}' => escaped.push_str("\\f"),
            '\u{8}' => escaped.push_str("\\b"),
            '\x00'..='\x1F' => {
                let mut codepoints = [0 as u16; 2];
                c.encode_utf16(&mut codepoints);
                escaped.push_str(&format!("\\u{:04X}", codepoints[0]));
            }
            other => escaped.push(other),
        }
    }

    escaped.push('"');

    escaped
}

fn display_json_array<W: Write>(
    items: &Vec<Json>,
    output: &mut W,
    indent: u64,
    level: u64,
) -> Result<(), fmt::Error> {
    let child_level = level + 1;

    if items.is_empty() {
        output.write_str("[]")?;
        return Ok(());
    }

    output.write_str("[\n")?;

    for (index, item) in items.into_iter().enumerate() {
        for _ in 0..(child_level * indent) {
            output.write_char(' ')?;
        }

        display_json(item, output, indent, child_level)?;

        if index < items.len() - 1 {
            output.write_char(',')?;
        }

        output.write_char('\n')?;
    }

    for _ in 0..(level * indent) {
        output.write_char(' ')?;
    }

    output.write_char(']')?;
    Ok(())
}

fn display_json_object<W: Write>(
    object: &BTreeMap<String, Json>,
    output: &mut W,
    indent: u64,
    level: u64,
) -> Result<(), fmt::Error> {
    let child_level = level + 1;

    if object.is_empty() {
        output.write_str("{}")?;
        return Ok(());
    }

    output.write_str("{\n")?;

    for (index, (key, value)) in object.into_iter().enumerate() {
        for _ in 0..(child_level * indent) {
            output.write_char(' ')?;
        }

        output.write_str(&display_json_string(&key))?;

        output.write_str(": ")?;

        display_json(value, output, indent, child_level)?;

        if index < object.len() - 1 {
            output.write_char(',')?;
        }

        output.write_char('\n')?;
    }

    for _ in 0..(level * indent) {
        output.write_char(' ')?;
    }

    output.write_char('}')?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{ast::Json, printer::json_to_string};

    #[test]
    fn it_prints_null() {
        assert_eq!(json_to_string(&Json::Null, 2), "null");
    }

    #[test]
    fn it_prints_booleans() {
        assert_eq!(json_to_string(&Json::Boolean(true), 2), "true");
        assert_eq!(json_to_string(&Json::Boolean(false), 2), "false");
    }

    #[test]
    fn it_prints_numbers() {
        assert_eq!(json_to_string(&Json::Number(0.0), 2), "0");
        assert_eq!(json_to_string(&Json::Number(234.0), 2), "234");
        assert_eq!(json_to_string(&Json::Number(-234.0), 2), "-234");
        assert_eq!(json_to_string(&Json::Number(123.456), 2), "123.456");
        assert_eq!(json_to_string(&Json::Number(10000.00001), 2), "10000.00001");
        assert_eq!(
            json_to_string(&Json::Number(0.00000000001), 2),
            "0.00000000001"
        );
        assert_eq!(
            json_to_string(&Json::Number(2405946039048539.0), 2),
            "2405946039048539"
        );
    }

    #[test]
    fn it_prints_ascii_strings() {
        assert_eq!(
            json_to_string(&Json::String("This is a string.".to_owned()), 2),
            r#""This is a string.""#
        );
    }

    #[test]
    fn it_prints_non_ascii_strings() {
        assert_eq!(
            json_to_string(&Json::String("😃 or 🙁?".to_owned()), 2),
            r#""😃 or 🙁?""#
        );
    }

    #[test]
    fn it_prints_a_string_with_an_escaped_double_quote() {
        assert_eq!(
            json_to_string(&Json::String("double \" quote".to_owned()), 2),
            r#""double \" quote""#
        );
    }

    #[test]
    fn it_prints_a_string_with_an_escaped_backslash() {
        assert_eq!(
            json_to_string(&Json::String("back \\ slash".to_owned()), 2),
            r#""back \\ slash""#
        );
    }

    #[test]
    fn it_prints_a_string_with_a_solidus_without_escaping_it() {
        assert_eq!(
            json_to_string(&Json::String("forward / slash".to_owned()), 2),
            r#""forward / slash""#
        );
    }

    #[test]
    fn it_prints_a_string_with_an_escaped_backspace() {
        assert_eq!(
            json_to_string(&Json::String("back \x08 space".to_owned()), 2),
            r#""back \b space""#
        );
    }

    #[test]
    fn it_prints_a_string_with_an_escaped_formfeed() {
        assert_eq!(
            json_to_string(&Json::String("form \x0C feed".to_owned()), 2),
            r#""form \f feed""#,
        );
    }

    #[test]
    fn it_prints_a_string_with_an_escaped_linefeed() {
        assert_eq!(
            json_to_string(&Json::String("line \n feed".to_owned()), 2),
            r#""line \n feed""#,
        );
    }

    #[test]
    fn it_prints_a_string_with_an_escaped_carriage_return() {
        assert_eq!(
            json_to_string(&Json::String("carriage \r return".to_owned()), 2),
            r#""carriage \r return""#,
        );
    }

    #[test]
    fn it_prints_a_string_with_an_escaped_tab() {
        assert_eq!(
            json_to_string(&Json::String("horizontal \t tab".to_owned()), 2),
            r#""horizontal \t tab""#,
        );
    }

    #[test]
    fn it_prints_a_string_with_an_escaped_control_characters() {
        assert_eq!(
            json_to_string(&Json::String("null \x00 character".to_owned()), 2),
            r#""null \u0000 character""#,
        );
        assert_eq!(
            json_to_string(&Json::String("unit \x1F separator".to_owned()), 2),
            r#""unit \u001F separator""#,
        );
    }

    #[test]
    fn it_prints_an_array_with_one_element_per_line_with_2_space_indent() {
        assert_eq!(
            json_to_string(
                &Json::Array(vec!(Json::Null, Json::Boolean(true), Json::Boolean(false))),
                2
            ),
            "[\n  null,\n  true,\n  false\n]",
        );
    }

    #[test]
    fn it_prints_an_empty_array_on_one_line() {
        assert_eq!(json_to_string(&Json::Array(vec!()), 2), "[]",);
    }

    #[test]
    fn it_prints_an_array_with_one_element_per_line_with_4_space_indent() {
        assert_eq!(
            json_to_string(
                &Json::Array(vec!(Json::Null, Json::Boolean(true), Json::Boolean(false))),
                4
            ),
            "[\n    null,\n    true,\n    false\n]",
        );
    }

    #[test]
    fn it_prints_a_nested_array_with_increasing_levels_of_indentation() {
        assert_eq!(
            json_to_string(
                &Json::Array(vec!(Json::Null, Json::Array(vec!(Json::Array(vec!()))))),
                2
            ),
            "[\n  null,\n  [\n    []\n  ]\n]",
        );
    }

    #[test]
    fn it_prints_an_empty_object_on_one_line() {
        assert_eq!(json_to_string(&Json::Object(BTreeMap::from([])), 2), "{}",);
    }

    #[test]
    fn it_prints_an_object_with_one_key_per_line_with_2_space_indent() {
        assert_eq!(
            json_to_string(
                &Json::Object(BTreeMap::from([
                    ("key1".to_owned(), Json::String("value1".to_owned())),
                    ("key2".to_owned(), Json::String("value2".to_owned()))
                ])),
                2
            ),
            "{\n  \"key1\": \"value1\",\n  \"key2\": \"value2\"\n}",
        );
    }

    #[test]
    fn it_prints_an_object_with_one_key_per_line_with_4_space_indent() {
        assert_eq!(
            json_to_string(
                &Json::Object(BTreeMap::from([
                    ("key1".to_owned(), Json::String("value1".to_owned())),
                    ("key2".to_owned(), Json::String("value2".to_owned()))
                ])),
                4
            ),
            "{\n    \"key1\": \"value1\",\n    \"key2\": \"value2\"\n}",
        );
    }

    #[test]
    fn it_prints_a_nested_object_with_increasing_levels_of_indentation() {
        assert_eq!(
            json_to_string(
                &Json::Object(BTreeMap::from([(
                    "deeply".to_owned(),
                    Json::Object(BTreeMap::from([(
                        "nested".to_owned(),
                        Json::Object(BTreeMap::from([(
                            "object".to_owned(),
                            Json::Object(BTreeMap::from([]))
                        )]))
                    )]))
                )])),
                2
            ),
            "{\n  \"deeply\": {\n    \"nested\": {\n      \"object\": {}\n    }\n  }\n}",
        );
    }
}
