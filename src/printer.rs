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
    output.write_char('[')?;

    for (index, item) in items.into_iter().enumerate() {
        if index > 0 {
            output.write_char(',')?;
        }
        display_json(item, output, indent, level)?;
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
    output.write_char('{')?;

    for (index, (key, value)) in object.into_iter().enumerate() {
        if index > 0 {
            output.write_char(',')?;
        }
        output.write_str(&display_json_string(&key))?;
        output.write_char(':')?;
        display_json(value, output, indent, level)?;
    }

    output.write_char('}')?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{ast::Json, printer::json_to_string};

    #[test]
    fn it_prints_null() {
        assert_eq!(json_to_string(&Json::Null, 2), "null");
    }
}
