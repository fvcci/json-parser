use std::collections::HashMap;

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

fn parse_object(tokens: &[lexical::Token]) -> Result<Value, Vec<Error>> {
    Ok(Value::Null)
}

fn parse_array_elements(tokens: &[lexical::Token]) -> Result<Vec<Value>, Vec<Error>> {
    let token = tokens;
    // .next()
    // .ok_or(vec![Error::Unexpected("End of file".to_string())])?;

    let mut elements = parse_array_elements(tokens);
    Ok(Vec::new())
}

fn parse_array(mut tokens: &[lexical::Token]) -> Result<Value, Vec<Error>> {
    let open_bracket = &tokens[0];
    assert!(*open_bracket == lexical::Token::Punctuation('['));

    let token = &tokens[1];
    // .peek()
    // .ok_or(vec![Error::Unexpected("End of file".to_string())])?;

    match token {
        lexical::Token::Punctuation(']') => Ok(Value::Array(vec![])),
        _ => Ok(Value::Array(parse_array_elements(
            &tokens[1..tokens.len()],
        )?)),
    }
}

fn parse_value(tokens: &[lexical::Token]) -> Result<Value, Vec<Error>> {
    let token = &tokens[0];
    // .peek()
    // .ok_or(Err(vec![Error::Unexpected("End of file".to_string())]))?;

    match token {
        lexical::Token::Null => Ok(Value::Null),
        lexical::Token::Bool(val) => Ok(Value::Bool(*val)),
        lexical::Token::String(val) => Ok(Value::String((*val).as_str().to_string())),
        lexical::Token::Number(val) => Ok(Value::Number(*val)),
        lexical::Token::Punctuation(c) => match *c {
            '{' => Ok(parse_object(tokens)?),
            '[' => Ok(parse_array(tokens)?),
            ':' => Err(vec![Error::Expected("{".to_string())]),
            ',' => Err(vec![Error::Unexpected(",".to_string())]),
            '}' => Err(vec![Error::Expected("{".to_string())]),
            ']' => Err(vec![Error::Expected("]".to_string())]),
            a => panic!("{a} is not a valid punctuation in JSON"),
        },
    }
}

pub fn parse(json: &str) -> Result<Value, Vec<Error>> {
    let tokens = lexical::Token::try_from_json(json)
        .map_err(|x| x.into_iter().map(Error::LiteralError).collect::<Vec<_>>())?;
    parse_value(&tokens[0..tokens.len()])
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
