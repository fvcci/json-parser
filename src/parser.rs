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
    UnexpectedEndOfFile(String),
    Expected(String),
    MatchingOpeningPairNotFound(String),
}

fn parse_object(tokens: &[lexical::Token]) -> Result<(Value, &[lexical::Token]), Vec<Error>> {
    Ok((Value::Null, &[]))
}

fn parse_array_elements(
    tokens: &[lexical::Token],
) -> Result<(Vec<Value>, &[lexical::Token]), Vec<Error>> {
    if tokens.is_empty() {
        return Err(vec![Error::UnexpectedEndOfFile("Expected ]".to_string())]);
    }

    let mut elements = Vec::<Value>::new();
    let mut errors = Vec::<Error>::new();
    let mut remaining_tokens = tokens;
    while remaining_tokens[0] != lexical::Token::Punctuation(']') {
        match remaining_tokens[0] {
            lexical::Token::Punctuation(',') => remaining_tokens = &remaining_tokens[1..],
            _ => match parse_value(remaining_tokens) {
                Ok((parsed_elements, new_remaining_tokens)) => {
                    elements.push(parsed_elements);
                    remaining_tokens = new_remaining_tokens;
                }
                Err(parse_errors) => {
                    remaining_tokens = &remaining_tokens[1..];
                    errors.extend(parse_errors);
                }
            },
        }
    }

    if errors.is_empty() {
        Ok((elements, remaining_tokens))
    } else {
        Err(errors)
    }
}

fn parse_array(tokens: &[lexical::Token]) -> Result<(Value, &[lexical::Token]), Vec<Error>> {
    let open_bracket = &tokens[0];
    assert!(*open_bracket == lexical::Token::Punctuation('['));

    if tokens.len() == 1 {
        return Err(vec![Error::UnexpectedEndOfFile("Expected ]".to_string())]);
    }

    match tokens[1] {
        lexical::Token::Punctuation(']') => Ok((Value::Array(Vec::new()), &tokens[1..])),
        _ => parse_array_elements(&tokens[1..])
            .map(|(parsed_value, remaining_tokens)| (Value::Array(parsed_value), remaining_tokens)),
    }
}

fn parse_value(tokens: &[lexical::Token]) -> Result<(Value, &[lexical::Token]), Vec<Error>> {
    if tokens.len() == 0 {
        return Err(vec![Error::UnexpectedEndOfFile(
            "Expected value".to_string(),
        )]);
    }

    let value = match &tokens[0] {
        lexical::Token::Null => Value::Null,
        lexical::Token::Bool(val) => Value::Bool(*val),
        lexical::Token::String(val) => Value::String((*val).as_str().to_string()),
        lexical::Token::Number(val) => Value::Number(*val),
        lexical::Token::Punctuation(c) => match *c {
            '{' => return parse_object(tokens),
            '[' => return parse_array(tokens),
            ':' => Err(vec![Error::Expected("{".to_string())])?,
            ',' => Err(vec![Error::Expected("JSON value".to_string())])?,
            '}' => Err(vec![Error::MatchingOpeningPairNotFound(
                "{ not found".to_string(),
            )])?,
            ']' => Err(vec![Error::MatchingOpeningPairNotFound(
                "[ not found".to_string(),
            )])?,
            a => panic!("{a} is not a valid punctuation in JSON"),
        },
    };

    Ok((value, &tokens[1..]))
}

pub fn parse(json: &str) -> Result<Value, Vec<Error>> {
    let tokens = lexical::Token::try_from_json(json)
        .map_err(|x| x.into_iter().map(Error::LiteralError).collect::<Vec<_>>())?;
    parse_value(&tokens[..]).map(|(parsed_value, _)| parsed_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    // These should be lexer tests
    #[ignore]
    #[test]
    fn garbage_input() {}

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
        assert_eq!(Ok(Value::Array(Vec::new())), parse("[]"));
        // assert_eq!(Ok(Value::Object(HashMap::new())), parse("{}"));
    }

    #[test]
    fn valid_array() {
        let json = r#"
            [
                false, "a", 1.0,
                [ false, "a" ],
                null
            ]
        "#;

        assert_eq!(
            Ok(Value::Array(vec![
                Value::Bool(false),
                Value::String("a".to_string()),
                Value::Number(1.0),
                Value::Null,
                Value::Array(vec![Value::Bool(false), Value::String("a".to_string())])
            ])),
            parse(json)
        );
    }

    // #[test]
    // fn missing_comma() {
    //     let json = r#" [ false "a", ]
    //     "#;

    //     assert_eq!(Err(), parse(json));
    // }
}
