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
    tokens: &'a [lexical::Token],
    errors: Vec<Error>,
    line_number: usize,
    col_number: usize,
}

impl<'a> Parser<'a> {
    pub fn parse(json: &str) -> Result<Value, Vec<Error>> {
        let tokens = lexical::Token::try_from_json(json)?;

        let mut parser = Parser {
            tokens: &tokens[..],
            errors: Vec::<Error>::new(),
            line_number: 1,
            col_number: 1,
        };

        let value_opt = parser.parse_value();
        if !parser.errors.is_empty() {
            Err(parser.errors)
        } else if value_opt.is_none() || !parser.tokens.iter().all(|x| x.is_whitespace()) {
            Err(vec![Error::new(
                ErrorCode::EndOfFileExpected,
                parser.line_number,
                parser.col_number,
            )])
        } else {
            Ok(value_opt.unwrap())
        }
    }

    fn parse_value(&mut self) -> Option<Value> {
        self.parse_whitespace();

        if self.tokens.is_empty() {
            self.errors.push(Error::new(
                ErrorCode::EndOfFileWhileParsingValue,
                self.line_number,
                self.col_number,
            ));
            return None;
        }

        match &self.tokens[0] {
            lexical::Token::Null => {
                self.tokens = &self.tokens[1..];
                self.col_number += 4;
                Some(Value::Null)
            }
            lexical::Token::Bool(val) => {
                self.tokens = &self.tokens[1..];
                self.col_number += val.len();
                Some(Value::Bool(val.parse().unwrap()))
            }
            lexical::Token::String(val) => self.parse_string(&val),
            lexical::Token::Number(val) => self.parse_number(&val),
            lexical::Token::Punctuation(c) => match *c {
                '{' => self.parse_object(),
                '[' => self.parse_array(),
                ',' | '}' | ']' => {
                    self.errors.push(Error::new(
                        ErrorCode::ExpectedToken,
                        self.line_number,
                        self.col_number,
                    ));
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
        match self.tokens {
            [lexical::Token::Punctuation('[')] => {
                self.col_number += 1;
                self.tokens = &[];
                self.errors.push(Error::new(
                    ErrorCode::EndOfFileWhileParsing(']'),
                    self.line_number,
                    self.col_number,
                ));
                None
            }
            [lexical::Token::Punctuation('['), lexical::Token::Punctuation(']'), ..] => {
                self.tokens = &self.tokens[2..];
                self.col_number += 2;
                Some(Value::Array(Vec::new()))
            }
            [lexical::Token::Punctuation('['), ..] => {
                self.col_number += 1;
                self.tokens = &self.tokens[1..];
                self.parse_array_elements().map(Value::Array)
            }
            _ => {
                panic!("Arrays must start with '['");
            }
        }
    }

    fn parse_array_elements(&mut self) -> Option<Vec<Value>> {
        const END_OF_ELEMENTS: char = ']';

        if self.tokens.is_empty() {
            self.errors.push(Error::new(
                ErrorCode::EndOfFileWhileParsing(END_OF_ELEMENTS),
                self.line_number,
                self.col_number,
            ));
            return None;
        }

        let mut elements = Vec::<Value>::new();
        loop {
            self.parse_whitespace();
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
        match self.tokens {
            [lexical::Token::Punctuation('{')] => {
                self.col_number += 1;
                self.tokens = &[];
                self.errors.push(Error::new(
                    ErrorCode::EndOfFileWhileParsing('}'),
                    self.line_number,
                    self.col_number,
                ));
                None
            }
            [lexical::Token::Punctuation('{'), lexical::Token::Punctuation('}'), ..] => {
                self.col_number += 2;
                self.tokens = &self.tokens[2..];
                Some(Value::Object(HashMap::new()))
            }
            [lexical::Token::Punctuation('{'), ..] => {
                self.col_number += 1;
                self.tokens = &self.tokens[1..];
                self.parse_object_members().map(Value::Object)
            }
            _ => {
                panic!("Objects must start with '{{'");
            }
        }
    }

    fn parse_object_members(&mut self) -> Option<HashMap<String, Value>> {
        const END_OF_MEMBERS: char = '}';

        if self.tokens.is_empty() {
            self.errors.push(Error::new(
                ErrorCode::EndOfFileWhileParsing(END_OF_MEMBERS),
                self.line_number,
                self.col_number,
            ));
            return None;
        }

        let mut members = HashMap::<String, Value>::new();

        loop {
            self.parse_whitespace();
            match self.tokens {
                [lexical::Token::String(s), lexical::Token::Punctuation(':'), ..] => {
                    self.col_number += s.len() + 1;
                    self.tokens = &self.tokens[2..];
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
                [c, lexical::Token::Punctuation(':'), ..] => {
                    self.errors.push(Error::new(
                        ErrorCode::KeyMustBeAString,
                        self.line_number,
                        self.col_number,
                    ));
                    self.col_number += c.len() + 1;
                    self.tokens = &self.tokens[2..];
                    self.parse_value();
                }
                [lexical::Token::Punctuation(':'), ..] => {
                    self.errors.push(Error::new(
                        ErrorCode::KeyMustBeAString,
                        self.line_number,
                        self.col_number,
                    ));
                    self.col_number += 1;
                    self.tokens = &self.tokens[1..];
                    self.parse_value();
                }
                [lexical::Token::String(s), ..] => {
                    self.errors.push(Error::new(
                        ErrorCode::ExpectedColon,
                        self.line_number,
                        self.col_number,
                    ));
                    self.col_number += s.len();
                    self.tokens = &self.tokens[1..];
                }
                [token, ..] => {
                    self.errors.push(Error::new(
                        ErrorCode::KeyMustBeAString,
                        self.line_number,
                        self.col_number,
                    ));
                    self.col_number += token.len();
                    self.tokens = &self.tokens[1..];
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
                self.errors.push(Error::new(
                    ErrorCode::InvalidNumber(possible_number.to_string()),
                    self.line_number,
                    self.col_number,
                ));
                None
            }
        };
        self.tokens = &self.tokens[1..];
        self.col_number += possible_number.len();
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
            self.errors.push(Error::new(
                ErrorCode::ExpectedDoubleQuote,
                self.line_number,
                self.col_number,
            ));
            None
        } else {
            Some(Value::String(
                possible_string[1..possible_string.len() - 1].to_string(),
            ))
        };

        self.tokens = &self.tokens[1..];
        self.col_number += possible_string.len();

        ret
    }

    fn parse_sequence_separator(&mut self, end: char) -> bool {
        self.parse_whitespace();
        match self.tokens {
            c @ ([] | [lexical::Token::Punctuation(',')]) => {
                self.tokens = &[];
                self.errors.push(Error::new(
                    ErrorCode::EndOfFileWhileParsing(end),
                    self.line_number,
                    self.col_number,
                ));
                self.col_number += c.len();
                true
            }
            [lexical::Token::Punctuation(','), lexical::Token::Punctuation(possible_end), ..]
                if *possible_end == end =>
            {
                self.tokens = &self.tokens[2..];
                self.col_number += 1;
                self.errors.push(Error::new(
                    ErrorCode::ExpectedToken,
                    self.line_number,
                    self.col_number,
                ));
                self.col_number += 1;
                true
            }
            [lexical::Token::Punctuation(','), ..] => {
                self.tokens = &self.tokens[1..];
                self.col_number += 1;
                false
            }
            [lexical::Token::Punctuation(possible_end), ..] if *possible_end == end => {
                self.tokens = &self.tokens[1..];
                self.col_number += 1;
                true
            }
            [lexical::Token::NewLine, ..] => {
                panic!("Shouldn't be possible to encounter");
            }
            [token, ..] => {
                self.errors.push(Error::new(
                    ErrorCode::EndOfFileWhileParsing(end),
                    self.line_number,
                    self.col_number,
                ));
                self.col_number += token.len();
                false
            }
        }
    }

    fn parse_until_comma_or_end(&mut self, end: char) {
        let mut seen_non_comma_value = false;
        loop {
            self.parse_whitespace();
            match self.tokens {
                [] | [lexical::Token::Punctuation(','), ..] => {
                    break;
                }
                [lexical::Token::Punctuation(possible_end), ..] if *possible_end == end => {
                    break;
                }
                [lexical::Token::Punctuation(':'), ..] => {
                    self.col_number += 1;
                    self.tokens = &self.tokens[1..];
                    self.parse_value();
                }
                [c, ..] => {
                    self.col_number += c.len();
                    self.tokens = &self.tokens[1..];
                    seen_non_comma_value = true;
                }
            }
        }

        if seen_non_comma_value {
            self.errors.push(Error::new(
                ErrorCode::ExpectedCommaOrEndWhileParsing(end),
                self.line_number,
                self.col_number,
            ));
        }
    }

    fn parse_whitespace(&mut self) {
        loop {
            match self.tokens {
                [lexical::Token::NewLine, ..] => {
                    self.line_number += 1;
                    self.col_number = 1;
                    self.tokens = &self.tokens[1..];
                }
                [lexical::Token::Whitespace(num_chars), ..] => {
                    self.col_number += num_chars;
                    self.tokens = &self.tokens[1..];
                }
                _ => {
                    break;
                }
            }
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
        let expected = vec![Error::new(ErrorCode::InvalidNumber(json.to_string()), 1, 1)];
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
