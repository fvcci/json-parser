use std::collections::HashMap;

pub fn parse(_: &str) -> Value {
    Value::Object(HashMap::new())
}

#[derive(Debug, PartialEq)]
enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use Value::*;

    #[test]
    fn can_parse_empty_object() {
        let test_in = r"
            {}";
        let expected = Object(HashMap::new());
        assert_eq!(expected, parse(test_in));
    }
}
