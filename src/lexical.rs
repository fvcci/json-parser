use crate::errors::{Error, ErrorCode};

#[derive(Debug, PartialEq)]
pub enum Token {
    NewLine,
    Null,
    Bool(bool),
    String(String),
    Number(f64),
    Punctuation(char),
}

impl Token {
    fn is_punctuation(c: &char) -> bool {
        const PUNCTUATIONS: &'static [char] = &[',', ':', '{', '}', '[', ']'];
        PUNCTUATIONS.contains(&c)
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

    fn try_from_number(possible_string: &str, line_number: usize) -> Result<Token, Error> {
        assert!(!possible_string.is_empty());
        match possible_string.parse::<f64>() {
            Ok(n) => Ok(Token::Number(n)),
            Err(_) => Err(Error::new(
                ErrorCode::InvalidNumber(possible_string.to_string()),
                line_number,
            )),
        }
    }

    fn try_from_token(token: &str, line_number: usize) -> Result<Token, Error> {
        assert!(!token.is_empty());

        let c = token.chars().next().unwrap();
        if Token::is_punctuation(&c) {
            return Ok(Token::Punctuation(c));
        }

        match (c, token) {
            ('\n', _) => Ok(Token::NewLine),
            ('n', "null") => Ok(Token::Null),
            ('f', "false") => Ok(Token::Bool(false)),
            ('t', "true") => Ok(Token::Bool(true)),
            ('"', _) => Token::try_from_string(token, line_number),
            ('-', _) => Token::try_from_number(token, line_number),
            ('0'..='9', _) => Token::try_from_number(token, line_number),
            _ => Err(Error::new(ErrorCode::ExpectedToken, line_number)),
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

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(tokens)
        }
    }
}

fn tokenize_into_strings(possible_json: &str) -> Vec<String> {
    let mut is_in_quotes = false;
    let mut tokens = Vec::<char>::new();
    let mut chars = possible_json.chars();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                is_in_quotes = !is_in_quotes;
                tokens.push(c);
            }
            '\\' if is_in_quotes => {
                tokens.push('\\');
                if let Some(c) = chars.next() {
                    tokens.push(c);
                }
            }
            '\n' => {
                tokens.push(' ');
                tokens.push('\0');
                tokens.push(' ');
            }
            c if is_in_quotes && c.is_whitespace() => {
                tokens.push('\0');
            }
            c if !is_in_quotes && Token::is_punctuation(&c) => {
                tokens.push(' ');
                tokens.push(c);
                tokens.push(' ');
            }
            _ => {
                tokens.push(c);
            }
        }
    }

    tokens
        .iter()
        .collect::<String>()
        .split_whitespace()
        .map(|x| {
            if x == "\0" {
                "\n".to_string()
            } else {
                x.replace("\0", " ").to_string()
            }
        })
        .filter(|x| x != "\n") // TODO temporary
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod tokenize_into_strings {
        use super::*;
        #[test]
        fn fail_space_separated_garbage() {
            let json = "this is garbage";
            assert_eq!(vec!["this", "is", "garbage"], tokenize_into_strings(json));
        }

        #[ignore]
        #[test]
        fn fail_on_multiple_quotes_in_one_token() {
            let json = r#"
                "d"fds"potato"
            "#;
            let expected = vec!["\n", "\"d\"", "fds", "\"potato\"", "\n"];
            assert_eq!(expected, tokenize_into_strings(json));
        }

        #[test]
        fn pass_space_in_string() {
            let json = "\" fjdsoif fds\" fd";
            assert_eq!(vec!["\" fjdsoif fds\"", "fd"], tokenize_into_strings(json));
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
            let expected = vec![Error::new(
                ErrorCode::InvalidNumber("11.3de2".to_string()),
                1,
            )];
            assert_eq!(Err(expected), Token::try_from_json(json));
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
            let json = r#"{"age":30,"is_student":[false]}"#;
            let expected = vec![
                Token::Punctuation('{'),
                Token::String(String::from("age")),
                Token::Punctuation(':'),
                Token::Number(30.0),
                Token::Punctuation(','),
                Token::String(String::from("is_student")),
                Token::Punctuation(':'),
                Token::Punctuation('['),
                Token::Bool(false),
                Token::Punctuation(']'),
                Token::Punctuation('}'),
            ];
            assert_eq!(Ok(expected), Token::try_from_json(json));
        }
    }
}
