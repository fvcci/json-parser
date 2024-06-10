use std::{cmp::min, collections::VecDeque, iter::Peekable, str::Chars};

use crate::errors::{Error, ErrorCode};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    NewLine,
    Whitespace(usize),
    Null,
    Bool(String),
    String(String),
    Number(String),
    Punctuation(char),
}

impl Token {
    pub fn is_whitespace(&self) -> bool {
        match self {
            Self::NewLine => true,
            Self::Whitespace(_) => true,
            _ => false,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::NewLine => 1,
            Self::Whitespace(spaces) => *spaces,
            Self::Null => 4,
            Self::Bool(b) => b.len(),
            Self::String(s) => s.len(),
            Self::Number(s) => s.len(),
            Self::Punctuation(_) => 1,
        }
    }

    pub fn try_from_json(possible_json: &str) -> Result<Vec<Token>, Vec<Error>> {
        let token_strings = tokenize_into_strings(&possible_json);

        let mut tokens = Vec::<Token>::new();
        let mut errors = Vec::<Error>::new();
        let mut line_number = 1usize;
        let mut col_number = 1usize;
        for token in token_strings {
            if token == "\n" {
                line_number += 1;
                col_number = 1;
            }
            match Token::try_from_token(&token) {
                Some(t) => tokens.push(t),
                None => errors.push(Error::new(
                    ErrorCode::ExpectedToken,
                    line_number,
                    col_number,
                )),
            }
            col_number += token.len();
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(tokens)
        }
    }

    fn try_from_token(token: &str) -> Option<Token> {
        assert!(!token.is_empty());

        let c = token.chars().next().unwrap();
        if token.len() == 1 && Token::is_punctuation(&c) {
            return Some(Token::Punctuation(c));
        }

        match (c, token) {
            (' ', _) => Some(Token::Whitespace(token.len())),
            ('\n', _) => Some(Token::NewLine),
            ('n', "null") => Some(Token::Null),
            ('f', "false") => Some(Token::Bool("false".to_string())),
            ('t', "true") => Some(Token::Bool("true".to_string())),
            ('"', _) => Some(Token::String(token.to_string())),
            ('-', _) => Some(Token::Number(token.to_string())),
            ('0'..='9', _) => Some(Token::Number(token.to_string())),
            _ => None,
        }
    }

    fn is_punctuation(c: &char) -> bool {
        const PUNCTUATIONS: &'static [char] = &[',', ':', '{', '}', '[', ']'];
        PUNCTUATIONS.contains(&c)
    }
}

pub struct Reader<'a> {
    chars: Peekable<Chars<'a>>,
    buffer: Vec<Result<Token, Error>>,
    line: usize,
    col: usize,
}

impl<'a> Reader<'a> {
    pub fn new(possible_json: &'a str) -> Reader<'a> {
        Reader {
            chars: possible_json.chars().peekable(),
            buffer: Vec::<Result<Token, Error>>::new(),
            line: 1,
            col: 1,
        }
    }

    pub fn next(&mut self, num_tokens: usize) -> Vec<Result<Token, Error>> {
        self.read_in(num_tokens);
        self.buffer
            .drain(..min(self.buffer.len(), num_tokens))
            .collect()
    }

    pub fn peek(&mut self, num_tokens: usize) -> Vec<Result<Token, Error>> {
        self.read_in(num_tokens);
        self.buffer[..min(self.buffer.len(), num_tokens)].to_vec()
    }

    fn read_in(&mut self, num_tokens: usize) {
        if self.buffer.len() >= num_tokens {
            return;
        }

        let mut is_in_quotes = false;
        let mut cur_token = String::new();

        while let Some(c) = self.chars.next() {
            match c {
                '"' => {
                    is_in_quotes = !is_in_quotes;
                    cur_token.push('"');
                }
                c @ (',' | ':' | '{' | '}' | '[' | ']') if !is_in_quotes => {
                    if cur_token.is_empty() {
                        self.buffer.push(Ok(Token::Punctuation(c)));
                    } else {
                        self.buffer.push(
                            Token::try_from_token(&cur_token)
                                .ok_or(self.create_error(ErrorCode::ExpectedToken)),
                        );
                        cur_token.clear();
                        self.buffer.push(Ok(Token::Punctuation(c)));
                    }
                }
                c if !is_in_quotes && c.is_whitespace() => {
                    if !cur_token.is_empty() {
                        self.buffer.push(
                            Token::try_from_token(&cur_token)
                                .ok_or(self.create_error(ErrorCode::ExpectedToken)),
                        );
                        cur_token.clear();
                    }
                    self.read_whitespace();
                }
                c => {
                    cur_token.push(c);
                }
            }
            self.col += 1;

            if self.buffer.len() >= num_tokens {
                break;
            }
        }

        assert!(self.buffer.is_empty() || self.buffer.len() - 1 <= num_tokens);
        if !cur_token.is_empty() {
            assert!(
                self.buffer.len() < num_tokens,
                "All required tokens must not have been parsed. Found {:?} {:?}",
                self.buffer,
                cur_token
            );
            self.buffer.push(
                Token::try_from_token(&cur_token)
                    .ok_or(self.create_error(ErrorCode::ExpectedToken)),
            );
        }
    }

    pub fn create_error(&self, code: ErrorCode) -> Error {
        Error::new(code, self.line, self.col)
    }

    fn read_whitespace(&mut self) {
        while let Some(c) = self.chars.peek() {
            if !c.is_whitespace() {
                break;
            }

            match c {
                '\n' | '\r' => {
                    self.line += 1;
                    self.col = 1;
                }
                ' ' | '\t' => {
                    self.col += 1;
                }
                _ => {
                    panic!("{c} is not a whitespace");
                }
            }
            self.chars.next();
        }
    }
}

fn tokenize_into_strings(possible_json: &str) -> Vec<String> {
    let mut is_in_quotes = false;
    let mut tokens = Vec::<String>::new();
    let mut cur_token = String::new();
    let mut chars = possible_json.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                is_in_quotes = !is_in_quotes;
                cur_token.push(c);
            }
            '\\' if is_in_quotes => {
                cur_token.push('\\');
                if let Some(c) = chars.next() {
                    cur_token.push(c);
                }
            }
            c if !is_in_quotes && (Token::is_punctuation(&c) || c == '\n' || c == '\r') => {
                if !cur_token.is_empty() {
                    tokens.push(cur_token);
                }
                cur_token = String::new();
                tokens.push(c.to_string());
            }
            c if !is_in_quotes && c.is_whitespace() => {
                if !cur_token.is_empty() {
                    tokens.push(cur_token);
                    cur_token = String::new();
                }
                cur_token.push(' ');
                while let Some(c) = chars.peek() {
                    if *c == ' ' || *c == '\t' {
                        cur_token.push(' ');
                        chars.next();
                    } else {
                        break;
                    }
                }

                if !cur_token.is_empty() {
                    tokens.push(cur_token);
                    cur_token = String::new();
                }
            }
            _ => {
                cur_token.push(c);
            }
        }
    }

    if !cur_token.is_empty() {
        tokens.push(cur_token);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    mod reader {
        use super::*;

        #[test]
        fn pass_json() {
            let json = r#"{ "age" : 30 , "is_student" : [false] }"#;
            let mut reader = Reader::new(json);

            assert_eq!(
                vec![
                    Ok(Token::Punctuation('{')),
                    Ok(Token::String("\"age\"".to_string()))
                ],
                reader.peek(2)
            );

            assert_eq!(
                vec![
                    Ok(Token::Punctuation('{')),
                    Ok(Token::String("\"age\"".to_string()))
                ],
                reader.next(2)
            );

            assert_eq!(
                vec![
                    Ok(Token::Punctuation(':')),
                    Ok(Token::Number("30".to_string())),
                    Ok(Token::Punctuation(',')),
                    Ok(Token::String("\"is_student\"".to_string())),
                    Ok(Token::Punctuation(':')),
                    Ok(Token::Punctuation('[')),
                    Ok(Token::Bool("false".to_string())),
                    Ok(Token::Punctuation(']')),
                    Ok(Token::Punctuation('}'))
                ],
                reader.next(11)
            );
        }

        #[test]
        fn pass_single_token() {
            let mut reader = Reader::new(r#""}, \n ""#);

            assert_eq!(
                vec![Ok(Token::String(r#""}, \n ""#.to_string()))],
                reader.next(1)
            );
        }

        #[test]
        fn pass_invalid_json() {
            let mut reader = Reader::new(r#"[,,]"#);
            assert_eq!(vec![Ok(Token::Punctuation('[')),], reader.peek(1));
            assert_eq!(vec![Ok(Token::Punctuation('[')),], reader.next(1));
            assert_eq!(vec![Ok(Token::Punctuation(',')),], reader.next(1));
            assert_eq!(vec![Ok(Token::Punctuation(',')),], reader.next(1));
            assert_eq!(vec![Ok(Token::Punctuation(']')),], reader.next(1));
        }
    }

    mod tokenize_into_strings {
        use super::*;
        #[test]
        fn fail_space_separated_garbage() {
            let json = "this is garbage";
            assert_eq!(
                vec!["this", " ", "is", " ", "garbage"],
                tokenize_into_strings(json)
            );
        }

        #[test]
        fn fail_on_multiple_quotes_in_one_token() {
            let json = r#"
                "d"fds"potato"
            "#;
            let expected = vec![
                "\n",
                "                ",
                "\"d\"fds\"potato\"",
                "\n",
                "            ",
            ];
            assert_eq!(expected, tokenize_into_strings(json));
        }

        #[test]
        fn pass_space_in_string() {
            let json = "\" fjdsoif fds\" fd";
            assert_eq!(
                vec!["\" fjdsoif fds\"", " ", "fd"],
                tokenize_into_strings(json)
            );
        }

        #[test]
        fn should_separate_on_punctuation() {
            let json = r#"{"age":30,"is_student":[false]}"#;
            let expected = vec![
                "{",
                "\"age\"",
                ":",
                "30",
                ",",
                "\"is_student\"",
                ":",
                "[",
                "false",
                "]",
                "}",
            ];
            assert_eq!(expected, tokenize_into_strings(json));
        }
    }

    mod token {
        use super::*;

        #[test]
        fn fail_space_separated_garbage() {
            let expected = vec![
                Error::new(ErrorCode::ExpectedToken, 1, 1),
                Error::new(ErrorCode::ExpectedToken, 1, 6),
            ];
            let json = "this garbage";
            assert_eq!(Err(expected), Token::try_from_json(json));
        }

        #[test]
        fn pass_space_in_string() {
            let json = "\"fjdsoif fds\"";
            assert_eq!(
                Ok(vec![Token::String("\"fjdsoif fds\"".to_string())]),
                Token::try_from_json(json)
            );
        }

        #[test]
        fn should_tokenize_on_punctuation() {
            let json = r#" {"age":30,"is_student":[false]}"#;
            let expected = vec![
                Token::Whitespace(1),
                Token::Punctuation('{'),
                Token::String("\"age\"".into()),
                Token::Punctuation(':'),
                Token::Number("30".into()),
                Token::Punctuation(','),
                Token::String("\"is_student\"".into()),
                Token::Punctuation(':'),
                Token::Punctuation('['),
                Token::Bool("false".into()),
                Token::Punctuation(']'),
                Token::Punctuation('}'),
            ];
            assert_eq!(Ok(expected), Token::try_from_json(json));
        }
    }
}
