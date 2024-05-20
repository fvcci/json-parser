use crate::lexical;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

pub fn parse(possible_json: &str) -> Result<Value, lexical::Error> {
    
}

#[cfg(test)]
mod tests {
    use super::*;

    // These should be lexer tests
    #[ignore]
    #[test]
    fn garbage_input() {}

    // These should be lexer tests
    #[ignore]
    #[test]
    fn can_parse_single_value_json() {
        assert_eq!(Ok(Value::Null), parse("null"));
        assert_eq!(Ok(Value::Bool(true)), parse("true"));
        assert_eq!(Ok(Value::Bool(false)), parse("false"));
        assert_eq!(Ok(Value::Number(12321.0)), parse("12321"));
        assert_eq!(
            Ok(Value::String(String::from("Hello World"))),
            parse("Hello World")
        );
        assert_eq!(Ok(Value::Array(Vec::new())), parse("[]"));
        assert_eq!(Ok(Value::Object(HashMap::new())), parse("{}"));
    }
}
