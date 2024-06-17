use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter, Write},
};

use crate::ast::Json;

impl fmt::Display for Json {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Json::Null => formatter.write_str("null"),
            Json::Boolean(true) => formatter.write_str("true"),
            Json::Boolean(false) => formatter.write_str("false"),
            Json::String(string) => formatter.write_str(&display_json_string(string)),
            Json::Number(number) => formatter.write_fmt(format_args!("{number}")),
            Json::Array(array) => display_json_array(array, formatter),
            Json::Object(object) => display_json_object(object, formatter),
        }
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

fn display_json_array(items: &Vec<Json>, formatter: &mut Formatter) -> Result<(), fmt::Error> {
    formatter.write_char('[')?;

    for (index, item) in items.into_iter().enumerate() {
        if index > 0 {
            formatter.write_char(',')?;
        }
        item.fmt(formatter)?;
    }

    formatter.write_char(']')?;
    Ok(())
}

fn display_json_object(
    object: &BTreeMap<String, Json>,
    formatter: &mut Formatter,
) -> Result<(), fmt::Error> {
    formatter.write_char('{')?;

    for (index, (key, value)) in object.into_iter().enumerate() {
        if index > 0 {
            formatter.write_char(',')?;
        }
        formatter.write_str(&display_json_string(&key))?;
        formatter.write_char(':')?;
        value.fmt(formatter)?;
    }

    formatter.write_char('}')?;
    Ok(())
}
