use std::collections::BTreeMap;

use crate::dsl::ToJson;

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Boolean(bool),
    String(String),
    Number(f64),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    fn object() -> Json {
        Json::Object(BTreeMap::new())
    }
    fn get(&self, property: &str) -> Option<&Json> {
        match self {
            Json::Object(properties) => properties.get(property),
            _ => None,
        }
    }
    fn set(self, property: &str, value: impl ToJson) -> Json {
        if let Json::Object(mut properties) = self {
            properties.insert(property.to_owned(), value.to_json());
            Json::Object(properties)
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::Json, dsl::ToJson};

    #[test]
    fn get_returns_the_value_of_a_property_if_called_on_an_object() {
        assert_eq!(
            [("foo", "bar")].to_json().get("foo"),
            Some(&"bar".to_json())
        );
    }
    #[test]
    fn get_returns_none_if_the_property_is_not_in_the_object() {
        assert_eq!([("foo", "bar")].to_json().get("baz"), None);
    }
    #[test]
    fn get_returns_none_if_called_on_a_non_object() {
        assert_eq!(Json::Null.get("foo"), None);
        assert_eq!("a string".to_json().get("foo"), None);
        assert_eq!(123.to_json().get("foo"), None);
        assert_eq!(true.to_json().get("foo"), None);
        assert_eq!([1, 2, 3].to_json().get("foo"), None);
    }

    #[test]
    fn set_can_be_chained_to_construct_an_object() {
        assert_eq!(
            Json::object().set("foo", "bar").set("baz", 123),
            [("foo", "bar".to_json()), ("baz", 123.to_json())].to_json()
        );
    }

    #[test]
    fn set_has_no_effect_on_a_non_object() {
        assert_eq!(Json::Null.set("foo", "bar"), Json::Null);
        assert_eq!("hello".to_json().set("foo", "bar"), "hello".to_json());
        assert_eq!(123.to_json().set("foo", "bar"), 123.to_json());
        assert_eq!(true.to_json().set("foo", "bar"), true.to_json());
        assert_eq!([1, 2, 3].to_json().set("foo", "bar"), [1, 2, 3].to_json());
    }
}
