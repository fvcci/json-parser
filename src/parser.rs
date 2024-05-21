use std::collections::HashMap;
use std::iter::Peekable;

use crate::lexical;

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

enum Error {
    LiteralError(lexical::LiteralError),
    Expected(String),
    Unexpected(String),
}

pub fn parse_object<'a>(
    _possible_json_it: impl Iterator<Item = &'a lexical::Token>,
) -> Result<Value, Vec<Error>> {
    Ok(Value::Null)
}

pub fn parse_array<'a>(
    possible_json_it: impl Iterator<Item = &'a lexical::Token>,
) -> Result<Value, Vec<Error>> {
    let optional_token = possible_json_it.next();
    if optional_token == None {
        return Err(vec![Error::Expected("[".to_string())]);
    }
    let arr = Vec::<Value>::new();
}

pub fn parse_value<'a>(
    mut possible_json_it: impl Iterator<Item = &'a lexical::Token>,
) -> Result<Option<Value>, Vec<Error>> {
    let optional_token = possible_json_it.next();
    if optional_token == None {
        return Ok(None);
    }

    match optional_token.unwrap() {
        lexical::Token::Null => Ok(Some(Value::Null)),
        lexical::Token::Bool(val) => Ok(Some(Value::Bool(*val))),
        lexical::Token::String(val) => Ok(Some(Value::String((*val).as_str().to_string()))),
        lexical::Token::Number(val) => Ok(Some(Value::Number(*val))),
        lexical::Token::Punctuation(c) => match c {
            '{' => Ok(Some(parse_object(possible_json_it)?)),
            '[' => Ok(Some(parse_array(possible_json_it)?)),
            ':' => Err(vec![Error::Expected("{".to_string())]),
            ',' => Err(vec![Error::Unexpected(",".to_string())]),
            '}' => Err(vec![Error::Expected("{".to_string())]),
            ']' => Err(vec![Error::Expected("]".to_string())]),
            a => panic!("{a} is not a valid punctuation in JSON"),
        },
    }
}

pub fn parse(possible_json: &str) -> Result<Value, Vec<Error>> {
    let tokens = lexical::Token::try_from_json(possible_json)
        .map_err(|x| x.into_iter().map(Error::LiteralError).collect::<Vec<_>>())?;
    Ok(parse_value(tokens.iter())?.unwrap_or(Value::Null))
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
