use std::str::SplitWhitespace;

use crate::errors::{Error, ErrorCode};

#[derive(Debug, PartialEq)]
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
        for token in token_strings {
            if token == "\n" {
                line_number += 1;
            }
            match Token::try_from_token(&token, line_number) {
                Ok(t) => tokens.push(t),
                Err(error) => errors.push(error),
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(tokens)
        }
    }

    fn try_from_token(token: &str, line_number: usize) -> Result<Token, Error> {
        assert!(!token.is_empty());

        let c = token.chars().next().unwrap();
        if Token::is_punctuation(&c) {
            return Ok(Token::Punctuation(c));
        }

        match (c, token) {
            (' ', _) => Ok(Token::Whitespace(token.len())),
            ('\n', _) => Ok(Token::NewLine),
            ('n', "null") => Ok(Token::Null),
            ('f', "false") => Ok(Token::Bool("false".to_string())),
            ('t', "true") => Ok(Token::Bool("true".to_string())),
            ('"', _) => Token::try_from_string(token, line_number),
            ('-', _) => Ok(Token::Number(token.to_string())),
            ('0'..='9', _) => Ok(Token::Number(token.to_string())),
            _ => Err(Error::new(ErrorCode::ExpectedToken, line_number)),
        }
    }

    fn try_from_string(possible_string: &str, line_number: usize) -> Result<Token, Error> {
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
        if possible_string.len() == 1 || num_quotations != 2 || last != '"' {
            Err(Error::new(ErrorCode::ExpectedDoubleQuote, line_number))
        } else {
            Ok(Token::String(
                possible_string[1..possible_string.len() - 1].to_string(),
            ))
        }
    }

    fn is_punctuation(c: &char) -> bool {
        const PUNCTUATIONS: &'static [char] = &[',', ':', '{', '}', '[', ']'];
        PUNCTUATIONS.contains(&c)
    }
}

// struct Reader<'a> {
//     chars: Peekable<Chars<'a>>,
//     is_in_quotes: bool,
//     line_number: usize,
//     col_number: usize,
// }

// impl<'a> Reader<'a> {
//     fn new(possible_json: &'a str) -> Reader<'a> {
//         Reader {
//             chars: possible_json.chars().peekable(),
//             is_in_quotes: false,
//             line_number: 1,
//             col_number: 1,
//         }
//     }

//     fn read_whitespace(&mut self) {
//         match self.chars.peek() {
//             Some('\n') | Some('\r') => {
//                 self.chars.next();
//                 self.line_number += 1;
//                 self.col_number += 1;
//             }
//             Some(' ') | Some('\t') => {
//                 self.chars.next();
//                 self.col_number += 1;
//             }
//             _ => {}
//         }
//     }

//     fn peek(&mut self, num_tokens: usize) -> Result<Vec<Token>, Vec<Error>> {
//         self.read_whitespace();

//         let mut tokens = Vec::<Token>::new();
//         let mut errors = Vec::<Error>::new();
//         let mut cur_token = String::new();

//         while let Some(c) = self.chars.next() {
//             match c {
//                 '"' => {
//                     self.is_in_quotes = !self.is_in_quotes;
//                     cur_token.push(c);
//                 }
//                 '\\' if self.is_in_quotes => {
//                     cur_token.push('\\');
//                     if let Some(c) = self.chars.next() {
//                         cur_token.push(c);
//                     }
//                 }
//                 c if !self.is_in_quotes && Token::is_punctuation(&c) => {
//                     if !cur_token.is_empty() {
//                         match Token::try_from_token(&cur_token, self.line_number) {
//                             Ok(val) => tokens.push(val),
//                             Err(e) => errors.push(e),
//                         }
//                         tokens.push(Token::Punctuation(c));
//                     } else {
//                         cur_token.push(c);
//                     }
//                     cur_token.clear();
//                 }
//                 _ => {
//                     assert!(!c.is_whitespace());
//                     cur_token.push(c);
//                 }
//             }

//             if tokens.len() == num_tokens {
//                 break;
//             }
//         }

//         if !cur_token.is_empty() {
//             match Token::try_from_token(&cur_token, self.line_number) {
//                 Ok(val) => tokens.push(val),
//                 Err(e) => errors.push(e),
//             }
//         }

//         if errors.is_empty() {
//             Ok(tokens)
//         } else {
//             Err(errors)
//         }
//     }
// }

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
                Error::new(ErrorCode::ExpectedToken, 1),
                Error::new(ErrorCode::ExpectedToken, 1),
            ];
            let json = "this garbage";
            assert_eq!(Err(expected), Token::try_from_json(json));
        }

        #[test]
        fn fail_on_multiple_quotes_in_one_token() {
            let json = r#"
                "d"fds"potato"
            "#;
            let expected = vec![Error::new(ErrorCode::ExpectedDoubleQuote, 2)];
            assert_eq!(Err(expected), Token::try_from_json(json));
        }

        #[test]
        fn fail_on_unmatched_quotation() {
            let json = r#""fds"#;
            let expected = vec![Error::new(ErrorCode::ExpectedDoubleQuote, 1)];
            assert_eq!(Err(expected), Token::try_from_json(json));
        }

        #[test]
        fn fail_on_invalid_number() {
            let json = r#"11.3de2"#;
            let expected = vec![Token::Number("11.3de2".into())];
            assert_eq!(Ok(expected), Token::try_from_json(json));
        }

        #[test]
        fn pass_space_in_string() {
            let json = "\"fjdsoif fds\"";
            assert_eq!(
                Ok(vec![Token::String("fjdsoif fds".to_string())]),
                Token::try_from_json(json)
            );
        }

        #[test]
        fn should_tokenize_on_punctuation() {
            let json = r#" {"age":30,"is_student":[false]}"#;
            let expected = vec![
                Token::Whitespace(1),
                Token::Punctuation('{'),
                Token::String("age".into()),
                Token::Punctuation(':'),
                Token::Number("30".into()),
                Token::Punctuation(','),
                Token::String(String::from("is_student")),
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
