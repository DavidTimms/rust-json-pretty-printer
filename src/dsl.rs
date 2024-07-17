use crate::ast::Json;

trait ToJson {
    fn to_json(&self) -> Json;
}

impl ToJson for bool {
    fn to_json(&self) -> Json {
        Json::Boolean(*self)
    }
}

impl ToJson for String {
    fn to_json(&self) -> Json {
        Json::String(self.clone())
    }
}

impl ToJson for str {
    fn to_json(&self) -> Json {
        Json::String(self.to_owned())
    }
}

impl ToJson for f64 {
    fn to_json(&self) -> Json {
        Json::Number(*self)
    }
}

impl ToJson for f32 {
    fn to_json(&self) -> Json {
        Json::Number(f64::from(*self))
    }
}

impl ToJson for i32 {
    fn to_json(&self) -> Json {
        Json::Number(f64::from(*self))
    }
}

impl<T: ToJson> ToJson for [T] {
    fn to_json(&self) -> Json {
        Json::Array(self.iter().map(ToJson::to_json).collect())
    }
}

impl<T: ToJson> ToJson for Vec<T> {
    fn to_json(&self) -> Json {
        Json::Array(self.iter().map(ToJson::to_json).collect())
    }
}

impl<T: ToJson> ToJson for Option<T> {
    fn to_json(&self) -> Json {
        match self {
            None => Json::Null,
            Some(t) => t.to_json(),
        }
    }
}

impl<T: ToJson> ToJson for [(&str, T)] {
    fn to_json(&self) -> Json {
        Json::Object(
            self.iter()
                .map(|(key, value)| ((*key).to_owned(), value.to_json()))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{ast::Json, dsl::ToJson};

    #[test]
    fn a_bool_is_converted_to_a_json_boolean() {
        assert_eq!(true.to_json(), Json::Boolean(true));
        assert_eq!(false.to_json(), Json::Boolean(false));
    }

    #[test]
    fn a_string_is_converted_to_a_json_string() {
        assert_eq!(
            "hello world!".to_owned().to_json(),
            Json::String("hello world!".to_owned())
        );
    }

    #[test]
    fn a_str_is_converted_to_a_json_string() {
        assert_eq!(
            "hello world!".to_json(),
            Json::String("hello world!".to_owned())
        );
    }

    #[test]
    fn a_f64_is_converted_to_a_json_number() {
        assert_eq!(123.456.to_json(), Json::Number(123.456));
    }

    #[test]
    fn a_f32_is_converted_to_a_json_number() {
        assert_eq!((123.0 as f32).to_json(), Json::Number(123.0));
    }

    #[test]
    fn an_i32_is_converted_to_a_json_number() {
        assert_eq!((123 as i32).to_json(), Json::Number(123.0));
    }

    #[test]
    fn an_array_is_converted_to_a_json_array() {
        assert_eq!(
            [1, 2, 3].to_json(),
            Json::Array(vec!(
                Json::Number(1.0),
                Json::Number(2.0),
                Json::Number(3.0)
            ))
        );
    }

    #[test]
    fn a_vec_is_converted_to_a_json_array() {
        assert_eq!(
            vec!(1, 2, 3).to_json(),
            Json::Array(vec!(
                Json::Number(1.0),
                Json::Number(2.0),
                Json::Number(3.0)
            ))
        );
    }

    #[test]
    fn an_option_which_is_none_is_converted_to_a_json_null() {
        assert_eq!(None::<i32>.to_json(), Json::Null);
    }

    #[test]
    fn an_option_which_is_some_is_converted_to_the_inner_types_json() {
        assert_eq!(Some(true).to_json(), Json::Boolean(true));
        assert_eq!(Some(123).to_json(), Json::Number(123.0));
    }

    #[test]
    fn an_array_of_key_value_pairs_is_converted_to_a_json_object() {
        assert_eq!(
            [("foo", 12), ("bar", 34),].to_json(),
            Json::Object(BTreeMap::from([
                ("foo".to_owned(), Json::Number(12.0)),
                ("bar".to_owned(), Json::Number(34.0))
            ]))
        );
    }
}
