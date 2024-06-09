use std::collections::HashMap;

use crate::{
    errors::{Error, ErrorCode},
    lexical,
};

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

pub struct Parser<'a> {
    reader: lexical::Reader<'a>,
    errors: Vec<Error>,
}

impl<'a> Parser<'a> {
    pub fn parse(json: &'a str) -> Result<Value, Vec<Error>> {
        let mut parser = Parser {
            reader: lexical::Reader::new(json),
            errors: Vec::<Error>::new(),
        };

        let value_opt = parser.parse_value();
        if !parser.errors.is_empty() {
            Err(parser.errors)
        } else if value_opt.is_none()
            || !parser
                .reader
                .next(usize::MAX)
                .iter()
                .all(|x| x.clone().is_ok_and(|y| y.is_whitespace()))
        {
            Err(vec![parser
                .reader
                .create_error(ErrorCode::EndOfFileExpected)])
        } else {
            Ok(value_opt.unwrap())
        }
    }

    fn parse_value(&mut self) -> Option<Value> {
        match self.reader.next(1).as_slice() {
            [] => {
                self.errors.push(
                    self.reader
                        .create_error(ErrorCode::EndOfFileWhileParsingValue),
                );
                None
            }
            [Err(error), ..] => {
                self.errors.push(*error);
                None
            }
            [Ok(lexical::Token::Null), ..] => Some(Value::Null),
            [Ok(lexical::Token::Bool(val)), ..] => Some(Value::Bool(val.parse().unwrap())),
            [Ok(lexical::Token::String(val)), ..] => self.parse_string(&val),
            [Ok(lexical::Token::Number(val)), ..] => self.parse_number(&val),
            [Ok(lexical::Token::Punctuation(c)), ..] => match *c {
                '{' => self.parse_object(),
                '[' => self.parse_array(),
                ',' | '}' | ']' | '|' => {
                    self.errors
                        .push(self.reader.create_error(ErrorCode::ExpectedToken));
                    None
                }
                a => panic!("{a} is not a valid punctuation in JSON"),
            },
            a => {
                panic!("{a:?} Shouldn't be possible to encounter");
            }
        }
    }

    fn parse_array(&mut self) -> Option<Value> {
        match self.reader.peek(2).as_slice() {
            [Err(error), ..] => {
                self.errors.push(*error);
                self.reader.next(1);
                None
            }
            [Ok(lexical::Token::Punctuation('['))] => {
                self.reader.next(1);
                self.errors.push(
                    self.reader
                        .create_error(ErrorCode::EndOfFileWhileParsing(']')),
                );
                None
            }
            [Ok(lexical::Token::Punctuation('[')), Ok(lexical::Token::Punctuation(']')), ..] => {
                self.reader.next(2);
                Some(Value::Array(Vec::new()))
            }
            [Ok(lexical::Token::Punctuation('[')), ..] => {
                self.reader.next(1);
                self.parse_array_elements().map(Value::Array)
            }
            _ => {
                panic!("Arrays must start with '['");
            }
        }
    }

    fn parse_array_elements(&mut self) -> Option<Vec<Value>> {
        const END_OF_ELEMENTS: char = ']';

        if self.reader.peek(1).is_empty() {
            self.reader
                .create_error(ErrorCode::EndOfFileWhileParsing(END_OF_ELEMENTS));
            return None;
        }

        let mut elements = Vec::<Value>::new();
        loop {
            if let Some(element) = self.parse_value() {
                elements.push(element);
            }

            self.parse_until_comma_or_end(END_OF_ELEMENTS);

            let reached_end = self.parse_sequence_separator(END_OF_ELEMENTS);
            if reached_end {
                break;
            }
        }

        Some(elements)
    }

    fn parse_object(&mut self) -> Option<Value> {
        match self.reader.peek(2).as_slice() {
            [Err(error), ..] => {
                self.errors.push(*error);
                self.reader.next(1);
                None
            }
            [Ok(lexical::Token::Punctuation('{'))] => {
                self.reader.next(1);
                self.errors.push(
                    self.reader
                        .create_error(ErrorCode::EndOfFileWhileParsing('}')),
                );
                None
            }
            [Ok(lexical::Token::Punctuation('{')), Ok(lexical::Token::Punctuation('}')), ..] => {
                self.reader.next(2);
                Some(Value::Object(HashMap::new()))
            }
            [Ok(lexical::Token::Punctuation('{')), ..] => {
                self.reader.next(1);
                self.parse_object_members().map(Value::Object)
            }
            _ => {
                panic!("Objects must start with '{{'");
            }
        }
    }

    fn parse_object_members(&mut self) -> Option<HashMap<String, Value>> {
        const END_OF_MEMBERS: char = '}';

        if self.reader.peek(1).is_empty() {
            self.errors.push(
                self.reader
                    .create_error(ErrorCode::EndOfFileWhileParsing(END_OF_MEMBERS)),
            );
            return None;
        }

        let mut members = HashMap::<String, Value>::new();

        loop {
            match self.reader.peek(2).as_slice() {
                [Err(error), ..] => {
                    self.errors.push(*error);
                    self.reader.next(1);
                }
                [Ok(lexical::Token::String(s)), Ok(lexical::Token::Punctuation(':')), ..] => {
                    self.reader.next(2);
                    match self.parse_string(&s) {
                        Some(Value::String(key)) => {
                            if let Some(value) = self.parse_value() {
                                members.insert(key, value);
                            }
                        }
                        Some(_) => {
                            panic!("Shouldn't be possible");
                        }
                        None => {
                            self.parse_value();
                        }
                    }
                }
                [_, Ok(lexical::Token::Punctuation(':')), ..] => {
                    self.errors
                        .push(self.reader.create_error(ErrorCode::KeyMustBeAString));
                    self.reader.next(2);
                    self.parse_value();
                }
                [Ok(lexical::Token::Punctuation(':')), ..] => {
                    self.errors
                        .push(self.reader.create_error(ErrorCode::KeyMustBeAString));
                    self.reader.next(1);
                    self.parse_value();
                }
                [Ok(lexical::Token::String(s)), ..] => {
                    self.errors
                        .push(self.reader.create_error(ErrorCode::ExpectedColon));
                    self.reader.next(1);
                }
                [_, ..] => {
                    self.errors
                        .push(self.reader.create_error(ErrorCode::KeyMustBeAString));
                    self.reader.next(1);
                }
                [] => {
                    panic!("Shouldn't be able to get an empty list");
                }
            }

            self.parse_until_comma_or_end(END_OF_MEMBERS);

            let reached_end = self.parse_sequence_separator(END_OF_MEMBERS);
            if reached_end {
                break;
            }
        }

        Some(members)
    }

    fn parse_number(&mut self, possible_number: &str) -> Option<Value> {
        assert!(!possible_number.is_empty());
        let ret = match possible_number.parse::<f64>() {
            Ok(n) => Some(Value::Number(n)),
            Err(_) => {
                self.errors
                    .push(self.reader.create_error(ErrorCode::InvalidNumber));
                None
            }
        };
        self.reader.next(1);
        ret
    }

    fn parse_string(&mut self, possible_string: &str) -> Option<Value> {
        assert!(!possible_string.is_empty());

        let mut chars = possible_string.chars().peekable();
        let mut num_quotations = 0;

        while let Some(c) = chars.next() {
            match (c, chars.peek()) {
                ('\\', Some('"')) => {
                    chars.next();
                }
                ('"', _) => {
                    num_quotations += 1;
                }
                _ => {}
            }
        }

        let first = possible_string.chars().next().unwrap();
        assert!(first == '"');

        let last = possible_string.chars().last().unwrap();
        let ret = if possible_string.len() == 1 || num_quotations != 2 || last != '"' {
            self.errors
                .push(self.reader.create_error(ErrorCode::ExpectedDoubleQuote));
            None
        } else {
            Some(Value::String(
                possible_string[1..possible_string.len() - 1].to_string(),
            ))
        };

        self.reader.next(1);

        ret
    }

    fn parse_sequence_separator(&mut self, end: char) -> bool {
        match self.reader.peek(2).as_slice() {
            [Err(error), ..] => {
                self.errors.push(*error);
                self.reader.next(1);
                false
            }
            c @ ([] | [Ok(lexical::Token::Punctuation(','))]) => {
                self.reader.next(1);
                self.errors.push(
                    self.reader
                        .create_error(ErrorCode::EndOfFileWhileParsing(end)),
                );
                true
            }
            [Ok(lexical::Token::Punctuation(',')), Ok(lexical::Token::Punctuation(possible_end)), ..]
                if *possible_end == end =>
            {
                self.reader.next(2);
                self.errors
                    .push(self.reader.create_error(ErrorCode::ExpectedToken));
                true
            }
            [Ok(lexical::Token::Punctuation(',')), ..] => {
                self.reader.next(1);
                false
            }
            [Ok(lexical::Token::Punctuation(possible_end)), ..] if *possible_end == end => {
                self.reader.next(1);
                true
            }
            [token, ..] => {
                self.errors.push(
                    self.reader
                        .create_error(ErrorCode::EndOfFileWhileParsing(end)),
                );
                false
            }
        }
    }

    fn parse_until_comma_or_end(&mut self, end: char) {
        let mut seen_non_comma_value = false;
        loop {
            match self.reader.peek(1).as_slice() {
                [] | [Ok(lexical::Token::Punctuation(',')), ..] => {
                    break;
                }
                [Ok(lexical::Token::Punctuation(possible_end)), ..] if *possible_end == end => {
                    break;
                }
                [Ok(lexical::Token::Punctuation(':')), ..] => {
                    self.reader.next(1);
                    self.parse_value();
                }
                [c, ..] => {
                    self.reader.next(1);
                    seen_non_comma_value = true;
                }
            }
        }

        if seen_non_comma_value {
            self.errors.push(
                self.reader
                    .create_error(ErrorCode::ExpectedCommaOrEndWhileParsing(end)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_single_value_json() {
        assert_eq!(Ok(Value::Null), Parser::parse("null"));
        assert_eq!(Ok(Value::Bool(true)), Parser::parse("true"));
        assert_eq!(Ok(Value::Bool(false)), Parser::parse("false"));
        assert_eq!(Ok(Value::Number(12321.0)), Parser::parse("12321"));
        assert_eq!(
            Ok(Value::String(String::from("Hello World"))),
            Parser::parse("\"Hello World\"")
        );
        assert_eq!(Ok(Value::Array(Vec::new())), Parser::parse("[]"));
        assert_eq!(Ok(Value::Object(HashMap::new())), Parser::parse("{}"));
    }

    #[test]
    fn pass_valid_array() {
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
            Parser::parse(json)
        );
    }

    #[test]
    fn fail_missing_comma() {
        assert_eq!(
            Err(vec![Error::new(
                ErrorCode::ExpectedCommaOrEndWhileParsing(']'),
                1,
                11
            ),]),
            Parser::parse(r#"[false "a"]"#)
        );
    }

    #[test]
    fn fail_many_commas() {
        assert_eq!(
            Err(vec![
                Error::new(ErrorCode::ExpectedToken, 1, 2),
                Error::new(ErrorCode::ExpectedToken, 1, 3),
                Error::new(ErrorCode::ExpectedToken, 1, 4)
            ]),
            Parser::parse(r#"[,,]"#)
        );
    }

    #[test]
    fn fail_unclosed_array() {
        assert_eq!(
            Err(vec![Error::new(
                ErrorCode::EndOfFileWhileParsing(']'),
                1,
                6
            ),]),
            Parser::parse("[true")
        );
    }

    #[test]
    fn fail_trailing_comma_array() {
        assert_eq!(
            Err(vec![Error::new(
                ErrorCode::EndOfFileWhileParsing(']'),
                1,
                6
            ),]),
            Parser::parse("[true,")
        );
    }

    #[test]
    fn fail_more_than_one_json_value() {
        assert_eq!(
            Err(vec![Error::new(ErrorCode::EndOfFileExpected, 1, 5)]),
            Parser::parse("null null")
        )
    }

    #[test]
    fn fail_unopened_object() {
        assert_eq!(
            Err(vec![
                Error::new(ErrorCode::ExpectedToken, 1, 9),
                Error::new(ErrorCode::ExpectedCommaOrEndWhileParsing(']'), 1, 10),
            ]),
            Parser::parse("[false, }]")
        )
    }

    #[test]
    fn include_elements_errors() {
        assert_eq!(
            Err(vec![
                Error::new(ErrorCode::ExpectedToken, 1, 4),
                Error::new(ErrorCode::ExpectedToken, 1, 14),
            ]),
            Parser::parse("[[ , false], ]")
        )
    }

    #[test]
    fn fail_on_no_key() {
        assert_eq!(
            Err(vec![Error::new(ErrorCode::KeyMustBeAString, 1, 3)]),
            Parser::parse(r#"{ : true}"#)
        )
    }

    #[test]
    fn fail_on_no_semi_colon() {
        assert_eq!(
            Err(vec![Error::new(ErrorCode::ExpectedColon, 1, 2),]),
            Parser::parse(r#"{"a"}"#)
        )
    }

    #[test]
    fn fail_on_multiple_quotes_in_one_token() {
        let json = r#"
            "d"fds"potato"
        "#;
        let expected = vec![Error::new(ErrorCode::ExpectedDoubleQuote, 2, 13)];
        assert_eq!(Err(expected), Parser::parse(json));
    }

    #[test]
    fn fail_on_unmatched_quotation() {
        let json = r#""fds"#;
        let expected = vec![Error::new(ErrorCode::ExpectedDoubleQuote, 1, 1)];
        assert_eq!(Err(expected), Parser::parse(json));
    }

    #[test]
    fn fail_on_invalid_number() {
        let json = r#"11.3de2"#;
        let expected = vec![Error::new(ErrorCode::InvalidNumber, 1, 1)];
        assert_eq!(Err(expected), Parser::parse(json));
    }

    #[test]
    fn pass_valid_object() {
        let json = r#"
            {
                "a": null,
                "b": [
                    null,
                    {}
                ]
            }
        "#;
        let obj = Value::Object(
            vec![
                ("a".to_string(), Value::Null),
                (
                    "b".to_string(),
                    Value::Array(vec![Value::Null, Value::Object(HashMap::new())]),
                ),
            ]
            .into_iter()
            .collect(),
        );
        assert_eq!(Ok(obj), Parser::parse(json))
    }
}
