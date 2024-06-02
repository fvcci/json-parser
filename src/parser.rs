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

fn parse_object_members(
    tokens: &[lexical::Token],
) -> (
    Result<HashMap<String, Value>, Vec<Error>>,
    &[lexical::Token],
) {
    if tokens.is_empty() {
        return (
            Err(vec![Error::UnexpectedEndOfFile("Expected '}'".to_string())]),
            &[],
        );
    }

    let mut members = HashMap::<String, Value>::new();
    let mut errors = Vec::<Error>::new();
    let mut remaining_tokens = tokens;

    const END_OF_MEMBERS: lexical::Token = lexical::Token::Punctuation('}');

    loop {
        match remaining_tokens {
            [lexical::Token::String(s), lexical::Token::Punctuation(':'), ..] => {
                remaining_tokens = &remaining_tokens[2..];
                match parse_value(remaining_tokens) {
                    (Ok(value), next_remaining_tokens) => {
                        remaining_tokens = next_remaining_tokens;
                        members.insert(s.as_str().to_string(), value);
                    }
                    (Err(parse_errors), next_remaining_tokens) => {
                        remaining_tokens = next_remaining_tokens;
                        errors.extend(parse_errors);
                    }
                }
            }
            [lexical::Token::String(_), ..] => {
                remaining_tokens = &remaining_tokens[1..];
                errors.push(Error::Expected(":".to_string()));
            }
            [lexical::Token::Punctuation(':'), ..] => {
                remaining_tokens = &remaining_tokens[1..];
                errors.push(Error::Expected("object key".to_string()));
            }
            _ => {
                errors.push(Error::Expected("object member".to_string()));
            }
        }

        let mut seen_non_comma_value = false;
        loop {
            match remaining_tokens {
                [] => {
                    break;
                }
                [END_OF_MEMBERS, ..] => {
                    break;
                }
                [lexical::Token::Punctuation(','), ..] => {
                    break;
                }
                _ => {
                    remaining_tokens = &remaining_tokens[1..];
                    seen_non_comma_value = true;
                }
            }
        }
        if seen_non_comma_value {
            errors.push(Error::Expected(",".to_string()));
        }

        // println!("{:?}", remaining_tokens);

        match remaining_tokens {
            [] => {
                errors.push(Error::UnexpectedEndOfFile(
                    "Expected ',' or '}'".to_string(),
                ));
                break;
            }
            [lexical::Token::Punctuation(',')] => {
                errors.push(Error::UnexpectedEndOfFile("Expected '}'".to_string()));
                remaining_tokens = &[];
                break;
            }
            [lexical::Token::Punctuation(','), END_OF_MEMBERS, ..] => {
                errors.push(Error::Expected("value".to_string()));
                remaining_tokens = &remaining_tokens[2..];
                break;
            }
            [lexical::Token::Punctuation(','), ..] => {
                remaining_tokens = &remaining_tokens[1..];
            }
            [END_OF_MEMBERS, ..] => {
                remaining_tokens = &remaining_tokens[1..];
                break;
            }
            [_, ..] => {
                errors.push(Error::Expected(",".to_string()));
            }
        }
    }

    if errors.is_empty() {
        (Ok(members), remaining_tokens)
    } else {
        (Err(errors), remaining_tokens)
    }
}

fn parse_object(tokens: &[lexical::Token]) -> (Result<Value, Vec<Error>>, &[lexical::Token]) {
    match tokens {
        [lexical::Token::Punctuation('{')] => (
            Err(vec![Error::UnexpectedEndOfFile("Expected '}'".to_string())]),
            &[],
        ),
        [lexical::Token::Punctuation('{'), lexical::Token::Punctuation('}')] => {
            (Ok(Value::Object(HashMap::new())), &tokens[2..])
        }
        [lexical::Token::Punctuation('{'), ..] => {
            let (parsed_elements, remaining_tokens) = parse_object_members(&tokens[1..]);
            (parsed_elements.map(Value::Object), remaining_tokens)
        }
        _ => {
            panic!("Objects must start with '{{'");
        }
    }
}

fn parse_array_elements(
    tokens: &[lexical::Token],
) -> (Result<Vec<Value>, Vec<Error>>, &[lexical::Token]) {
    if tokens.is_empty() {
        return (
            Err(vec![Error::UnexpectedEndOfFile("Expected ']'".to_string())]),
            &[],
        );
    }

    let mut elements = Vec::<Value>::new();
    let mut errors = Vec::<Error>::new();
    let mut remaining_tokens = tokens;

    const END_OF_ELEMENTS: lexical::Token = lexical::Token::Punctuation(']');

    loop {
        match parse_value(remaining_tokens) {
            (Ok(parsed_elements), next_remaining_tokens) => {
                remaining_tokens = next_remaining_tokens;
                elements.push(parsed_elements);
            }
            (Err(parse_errors), next_remaining_tokens) => {
                remaining_tokens = next_remaining_tokens;
                errors.extend(parse_errors);
            }
        }

        match remaining_tokens {
            [] => {
                errors.push(Error::UnexpectedEndOfFile(
                    "Expected ',' or ']'".to_string(),
                ));
                break;
            }
            [lexical::Token::Punctuation(',')] => {
                errors.push(Error::UnexpectedEndOfFile("Expected ']'".to_string()));
                remaining_tokens = &[];
                break;
            }
            [lexical::Token::Punctuation(','), END_OF_ELEMENTS, ..] => {
                errors.push(Error::Expected("value".to_string()));
                remaining_tokens = &remaining_tokens[2..];
                break;
            }
            [lexical::Token::Punctuation(','), ..] => {
                remaining_tokens = &remaining_tokens[1..];
            }
            [END_OF_ELEMENTS, ..] => {
                remaining_tokens = &remaining_tokens[1..];
                break;
            }
            [_, ..] => {
                errors.push(Error::Expected(",".to_string()));
            }
        }
    }

    if errors.is_empty() {
        assert!(
            remaining_tokens.is_empty() || remaining_tokens[0] == lexical::Token::Punctuation(',')
        );
        (Ok(elements), remaining_tokens)
    } else {
        (Err(errors), remaining_tokens)
    }
}

fn parse_array(tokens: &[lexical::Token]) -> (Result<Value, Vec<Error>>, &[lexical::Token]) {
    match tokens {
        [lexical::Token::Punctuation('[')] => (
            Err(vec![Error::UnexpectedEndOfFile("Expected ']'".to_string())]),
            &[],
        ),
        [lexical::Token::Punctuation('['), lexical::Token::Punctuation(']')] => {
            (Ok(Value::Array(Vec::new())), &tokens[2..])
        }
        [lexical::Token::Punctuation('['), ..] => {
            let (parsed_elements, remaining_tokens) = parse_array_elements(&tokens[1..]);
            (parsed_elements.map(Value::Array), remaining_tokens)
        }
        _ => {
            panic!("Arrays must start with '['");
        }
    }
}

fn parse_value(tokens: &[lexical::Token]) -> (Result<Value, Vec<Error>>, &[lexical::Token]) {
    if tokens.is_empty() {
        return (
            Err(vec![Error::UnexpectedEndOfFile(
                "Expected value".to_string(),
            )]),
            &[],
        );
    }

    let value = match &tokens[0] {
        lexical::Token::Null => Ok(Value::Null),
        lexical::Token::Bool(val) => Ok(Value::Bool(*val)),
        lexical::Token::String(val) => Ok(Value::String((*val).as_str().to_string())),
        lexical::Token::Number(val) => Ok(Value::Number(*val)),
        lexical::Token::Punctuation(c) => match *c {
            '{' => return parse_object(tokens),
            '[' => return parse_array(tokens),
            ':' => Err(vec![Error::Expected("{".to_string())]),
            ',' => return (Err(vec![Error::Expected("value".to_string())]), tokens),
            '}' => Err(vec![Error::MatchingOpeningPairNotFound(
                "{ not found".to_string(),
            )]),
            ']' => Err(vec![Error::MatchingOpeningPairNotFound(
                "[ not found".to_string(),
            )]),
            a => panic!("{a} is not a valid punctuation in JSON"),
        },
    };

    (value, &tokens[1..])
}

pub fn parse(json: &str) -> Result<Value, Vec<Error>> {
    let tokens = lexical::Token::try_from_json(json)
        .map_err(|x| x.into_iter().map(Error::LiteralError).collect::<Vec<_>>())?;
    match parse_value(&tokens[..]) {
        (Err(mut errors), [_, ..]) => {
            errors.push(Error::Expected("end of file".to_string()));
            Err(errors)
        }
        (Ok(_), [_, ..]) => Err(vec![Error::Expected("end of file".to_string())]),
        (parsed_json, []) => parsed_json,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(Ok(Value::Object(HashMap::new())), parse("{}"));
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
                Value::Array(vec![Value::Bool(false), Value::String("a".to_string())]),
                Value::Null,
            ])),
            parse(json)
        );
    }

    #[test]
    fn fail_missing_comma() {
        assert_eq!(
            Err(vec![Error::Expected(",".to_string())]),
            parse(r#"[false "a"]"#)
        );
    }

    #[test]
    fn fail_many_commas() {
        assert_eq!(
            Err(vec![
                Error::Expected("value".to_string()),
                Error::Expected("value".to_string()),
                Error::Expected("value".to_string())
            ]),
            parse(r#"[,,]"#)
        );
    }

    #[test]
    fn fail_unclosed_array() {
        assert_eq!(
            Err(vec![Error::UnexpectedEndOfFile(
                "Expected ',' or ']'".to_string()
            ),]),
            parse("[true")
        );
    }

    #[test]
    fn fail_trailing_comma_array() {
        assert_eq!(
            Err(vec![Error::UnexpectedEndOfFile("Expected ']'".to_string())]),
            parse("[true,")
        );
    }

    #[test]
    fn fail_more_than_one_json_value() {
        assert_eq!(
            Err(vec![Error::Expected("end of file".to_string())]),
            parse("null null")
        )
    }

    #[test]
    fn fail_unopened_array() {
        assert_eq!(
            Err(vec![Error::MatchingOpeningPairNotFound(
                "{ not found".to_string()
            )]),
            parse("[false, }]")
        )
    }

    #[test]
    fn include_elements_errors() {
        assert_eq!(
            Err(vec![
                Error::Expected("value".to_string()),
                Error::Expected("value".to_string())
            ]),
            parse("[[ , false], ]")
        )
    }

    #[test]
    fn fail_on_no_key() {
        assert_eq!(
            Err(vec![
                Error::Expected("object key".to_string()),
                Error::Expected(",".to_string()),
            ]),
            parse(r#"{ : true}"#)
        )
    }

    #[test]
    fn fail_on_no_semi_colon() {
        assert_eq!(
            Err(vec![Error::Expected(":".to_string())]),
            parse(r#"{"a"}"#)
        )
    }

    #[test]
    fn pass_valid_object() {
        let json = r#"
            {
                "a": null,
                "b": {
                    "c": null
                }
            }
        "#;
        let mut obj = HashMap::<String, Value>::new();
        obj.insert("a".to_string(), Value::Null);
        let mut b = HashMap::<String, Value>::new();
        b.insert("c".to_string(), Value::Null);
        obj.insert("b".to_string(), Value::Object(b));
        assert_eq!(Ok(Value::Object(obj)), parse(json))
    }
}
