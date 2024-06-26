use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum Json {
    Null,
    Boolean(bool),
    String(String),
    Number(f64),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}
