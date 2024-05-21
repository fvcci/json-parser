#[derive(Debug, PartialEq)]
pub enum LiteralError {
    ExpectedLiteral(String, String),
    InvalidString(String, String),
    InvalidNumber(String),
}

#[derive(Debug, PartialEq)]
pub enum Token {
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

    fn try_from_string(possible_string: &str) -> Result<Token, LiteralError> {
        assert!(possible_string.len() == 0);

        let num_quotations = possible_string
            .chars()
            .fold(0, |acc, x| if x == '"' { acc + 1 } else { acc });

        if num_quotations % 2 == 1 {
            return Err(LiteralError::InvalidString(
                possible_string.to_string(),
                "String has unmatched quotation".to_string(),
            ));
        }

        let first = possible_string.chars().nth(0);
        let last = possible_string.chars().nth(possible_string.len() - 1);
        if num_quotations != 2 || first.unwrap() != '"' || last.unwrap() != '"' {
            Err(LiteralError::InvalidString(
                possible_string.to_string(),
                "Invalid String".to_string(),
            ))
        } else {
            Ok(Token::String(
                possible_string[1..possible_string.len() - 1].to_string(),
            ))
        }
    }

    fn try_from_number(possible_string: &str) -> Result<Token, LiteralError> {
        assert!(possible_string.len() == 0);
        match possible_string.parse::<f64>() {
            Ok(n) => Ok(Token::Number(n)),
            Err(_) => Err(LiteralError::InvalidNumber(possible_string.to_string())),
        }
    }

    fn try_from_token(token: &str) -> Result<Token, LiteralError> {
        if token.len() == 0 {
            return Err(LiteralError::ExpectedLiteral(
                token.to_string(),
                "Nothing to parse".to_string(),
            ));
        }

        let c = token.chars().next().unwrap();
        if Token::is_punctuation(&c) {
            return Ok(Token::Punctuation(c));
        }

        match (c, token) {
            ('n', "null") => Ok(Token::Null),
            ('f', "false") => Ok(Token::Bool(false)),
            ('t', "true") => Ok(Token::Bool(true)),
            ('"', _) => Token::try_from_string(token),
            ('0'..='9', _) => Token::try_from_number(token),
            _ => Err(LiteralError::ExpectedLiteral(
                token.to_string(),
                "Expected a JSON object, array, or literal".to_string(),
            )),
        }
    }

    pub fn try_from_json(possible_json: &str) -> Vec<Result<Token, LiteralError>> {
        let token_strings = tokenize_into_strings(&possible_json);

        let mut tokens = Vec::<Result<Token, LiteralError>>::new();
        for token in token_strings {
            tokens.push(Token::try_from_token(&token));
        }

        tokens
    }
}

fn tokenize_into_strings(possible_json: &str) -> Vec<String> {
    let mut is_in_quotes = false;
    let mut tokens = Vec::<String>::new();

    for c in possible_json.chars() {
        if c == '"' {
            is_in_quotes = !is_in_quotes;
            tokens.push(c.to_string());
        } else if is_in_quotes && c.is_whitespace() {
            tokens.push('\0'.to_string());
        } else if !is_in_quotes && Token::is_punctuation(&c) {
            tokens.push(format!(" {c} "));
        } else {
            tokens.push(c.to_string());
        }
    }

    tokens
        .join("")
        .split_whitespace()
        .map(|x| x.replace("\0", " ").to_string())
        .collect()
}

#[cfg(test)]
mod tests {

    mod tokenize_into_strings {
        use super::super::*;
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
            let expected = vec!["\"d\"", "fds", "\"potato\""];
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
        use super::super::*;

        #[test]
        fn fail_space_separated_garbage() {
            let expected: Vec<Result<Token, LiteralError>> = vec![
                Err(LiteralError::ExpectedLiteral(
                    "this".to_string(),
                    "Expected a JSON object, array, or literal".to_string(),
                )),
                Err(LiteralError::ExpectedLiteral(
                    "garbage".to_string(),
                    "Expected a JSON object, array, or literal".to_string(),
                )),
            ];
            let json = "this garbage";
            assert_eq!(expected, Token::try_from_json(json));
        }

        #[test]
        fn fail_on_multiple_quotes_in_one_token() {
            let json = r#"
                "d"fds"potato"
            "#;
            let expected = vec![Err(LiteralError::InvalidString(
                "\"d\"fds\"potato\"".to_string(),
                "Invalid String".to_string(),
            ))];
            assert_eq!(expected, Token::try_from_json(json));
        }

        #[test]
        fn fail_on_unmatched_quotation() {
            let json = r#""fds"#;
            let expected = vec![Err(LiteralError::InvalidString(
                "\"fds".to_string(),
                "String has unmatched quotation".to_string(),
            ))];
            assert_eq!(expected, Token::try_from_json(json));
        }

        #[test]
        fn fail_on_invalid_number() {
            let json = r#"11.3de2"#;
            let expected = vec![Err(LiteralError::InvalidNumber("11.3de2".to_string()))];
            assert_eq!(expected, Token::try_from_json(json));
        }

        #[test]
        fn pass_space_in_string() {
            let json = "\"fjdsoif fds\"";
            assert_eq!(
                vec![Ok(Token::String("fjdsoif fds".to_string()))],
                Token::try_from_json(json)
            );
        }

        #[test]
        fn should_separate_on_punctuation() {
            let json = r#"{"age":30,"is_student":[false]}"#;
            let expected = vec![
                Ok(Token::Punctuation('{')),
                Ok(Token::String(String::from("age"))),
                Ok(Token::Punctuation(':')),
                Ok(Token::Number(30.0)),
                Ok(Token::Punctuation(',')),
                Ok(Token::String(String::from("is_student"))),
                Ok(Token::Punctuation(':')),
                Ok(Token::Punctuation('[')),
                Ok(Token::Bool(false)),
                Ok(Token::Punctuation(']')),
                Ok(Token::Punctuation('}')),
            ];
            assert_eq!(expected, Token::try_from_json(json));
        }
    }
}
