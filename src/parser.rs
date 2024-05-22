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

#[derive(Debug, PartialEq)]
pub enum Error {
    LiteralError(lexical::LiteralError),
    Expected(String),
    Unexpected(String),
}

fn parse_object<'a>(
    _possible_json_it: Peekable<impl Iterator<Item = &'a lexical::Token>>,
) -> Result<Value, Vec<Error>> {
    Ok(Value::Null)
}

fn parse_array_elements<'a>(
    _possible_json_it: Peekable<impl Iterator<Item = &'a lexical::Token>>,
) -> Result<Value, Vec<Error>> {
    Ok(Value::Null)
}

fn parse_array<'a>(
    mut possible_json_it: Peekable<impl Iterator<Item = &'a lexical::Token>>,
) -> Result<Value, Vec<Error>> {
    let optional_token = possible_json_it.peek();
    let reached_end_of_file = optional_token == None;
    if reached_end_of_file {
        return Err(vec![Error::Expected("]".to_string())]);
    }

    let mut _arr = Vec::<Value>::new();
    let mut _errs = Vec::<Value>::new();
    Ok(Value::Null)
}

fn parse_value<'a>(
    mut possible_json_it: Peekable<impl Iterator<Item = &'a lexical::Token>>,
) -> Result<Value, Vec<Error>> {
    let optional_token = possible_json_it.next();
    if optional_token == None {
        return Ok(Value::Null);
    }

    match optional_token.unwrap() {
        lexical::Token::Null => Ok(Value::Null),
        lexical::Token::Bool(val) => Ok(Value::Bool(*val)),
        lexical::Token::String(val) => Ok(Value::String((*val).as_str().to_string())),
        lexical::Token::Number(val) => Ok(Value::Number(*val)),
        lexical::Token::Punctuation(c) => match c {
            '{' => Ok(parse_object(possible_json_it)?),
            '[' => Ok(parse_array(possible_json_it)?),
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
    parse_value(tokens.iter().peekable())
}

#[cfg(test)]
mod tests {
    use super::*;

    // These should be lexer tests
    #[ignore]
    #[test]
    fn garbage_input() {}

    // These should be lexer tests
    #[test]
    fn pass_single_value_json() {
        assert_eq!(Ok(Value::Null), parse("null"));
        assert_eq!(Ok(Value::Bool(true)), parse("true"));
        assert_eq!(Ok(Value::Bool(false)), parse("false"));
        assert_eq!(Ok(Value::Number(12321.0)), parse("12321"));
        assert_eq!(
            Ok(Value::String(String::from("Hello World"))),
            parse("\"Hello World\"")
        );
        // assert_eq!(Ok(Value::Array(Vec::new())), parse("[]"));
        // assert_eq!(Ok(Value::Object(HashMap::new())), parse("{}"));
    }
}
