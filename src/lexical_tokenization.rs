use std::num::ParseFloatError;

#[derive(Debug, PartialEq)]
enum Token {
    Null,
    Bool(bool),
    String(String),
    Number(f64),
    Punctuation(char),
}

#[derive(Debug, PartialEq)]
enum LexicalError {
    ExpectedLiteral(String),
    InvalidCharacter(String),
    InvalidString(String),
    InvalidNumber(ParseFloatError),
}

impl Token {
    fn is_punctuation(c: &char) -> bool {
        const PUNCTUATIONS: &'static [char] = &[',', ':', '{', '}', '[', ']'];
        PUNCTUATIONS.contains(&c)
    }

    fn tokenize_string(possible_string: &str) -> Result<Token, LexicalError> {
        if possible_string.len() == 0 {
            return Err(LexicalError::ExpectedLiteral(String::from(
                "Nothing to parse.",
            )));
        }

        let num_quotations = possible_string
            .chars()
            .fold(0, |acc, x| if x == '"' { acc + 1 } else { acc });

        if num_quotations % 2 == 1 {
            return Err(LexicalError::InvalidString(String::from(
                "String has unmatched quotation",
            )));
        }

        let first = possible_string.chars().nth(0);
        let last = possible_string.chars().nth(possible_string.len() - 1);
        if num_quotations != 2 || first.unwrap() != '"' || last.unwrap() != '"' {
            Err(LexicalError::InvalidString(String::from(
                "String quotations are invalid",
            )))
        } else {
            Ok(Token::String(String::from(possible_string)))
        }
    }

    fn tokenize_number(possible_string: &str) -> Result<Token, LexicalError> {
        match possible_string.parse::<f64>() {
            Ok(n) => Ok(Token::Number(n)),
            Err(e) => Err(LexicalError::InvalidNumber(e)),
        }
    }

    fn tokenize_token(token: &str) -> Result<Token, LexicalError> {
        if token.len() == 0 {
            return Err(LexicalError::ExpectedLiteral(String::from(
                "Nothing to parse.",
            )));
        }

        let c = token.chars().next().unwrap();
        if Token::is_punctuation(&c) {
            return Ok(Token::Punctuation(c));
        }

        match (c, token) {
            ('n', "null") => Ok(Token::Null),
            ('f', "false") => Ok(Token::Bool(false)),
            ('t', "true") => Ok(Token::Bool(true)),
            ('"', _) => Token::tokenize_string(token),
            ('0'..='9', _) => Token::tokenize_number(token),
            _ => Err(LexicalError::ExpectedLiteral(String::from(
                "Expected a JSON object, array, or literal.",
            ))),
        }
    }

    fn tokenize_tokens(possible_json: &str) -> Vec<Result<Token, LexicalError>> {
        let token_strings = tokenize_into_strings(&possible_json);

        let mut tokens = Vec::<Result<Token, LexicalError>>::new();
        for token in token_strings {
            tokens.push(Token::tokenize_token(&token));
        }

        tokens
    }
}

// Assume "    fdsdfds" cannot be a case and "" cannot be joined together
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
    use super::*;

    #[test]
    fn space_separated_garbage_should_be_separated() {
        let json = "this is garbage";
        assert_eq!(vec!["this", "is", "garbage"], tokenize_into_strings(json));
    }

    #[ignore]
    #[test]
    fn separate_adjacent_strings() {
        let json = r#"
                "d"fds"potato"
        "#;
        let expected = vec!["\"d\"", "fds", "\"potato\""];
        assert_eq!(expected, tokenize_into_strings(json));
    }

    #[test]
    fn space_in_string() {
        let json = "\"fjdsoif fds\"";
        assert_eq!(vec!["\"fjdsoif fds\""], tokenize_into_strings(json));
    }

    #[test]
    fn separate_on_punctuation() {
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

    // #[test]
    // fn space_separated_garbage_should_be_separated() {
    //     let json = "this is garbage";
    //     assert_eq!(
    //         vec![
    //             Token::String(String::from("this")),
    //             Token::String(String::from("is")),
    //             Token::String(String::from("garbage"))
    //         ],
    //         Token::tokenize_tokens(json)
    //     );
    // }

    // #[test]
    // fn separate_adjacent_strings() {
    //     let json = r#"
    //         {
    //             "": "hii",
    //             "d""potato"
    //         }
    //     "#;
    //     let expected = vec![
    //         Token::Punctuation('{'),
    //         Token::String(String::from("")),
    //         Token::Punctuation(':'),
    //         Token::String(String::from("hii")),
    //         Token::Punctuation(','),
    //         Token::String(String::from("d")),
    //         Token::String(String::from("potato")),
    //         Token::Punctuation('}'),
    //     ];
    //     assert_eq!(expected, Token::tokenize_tokens(json));
    // }

    // #[test]
    // fn separate_on_punctuation() {
    //     let json = r#"
    //         {
    //             "age": 30,
    //             "is_student": [false],
    //         }
    //     "#;
    //     let expected = vec![
    //         Token::Punctuation('{'),
    //         Token::String(String::from("age")),
    //         Token::Punctuation(':'),
    //         Token::Number(30.0),
    //         Token::Punctuation(','),
    //         Token::String(String::from("is_student")),
    //         Token::Punctuation(':'),
    //         Token::Punctuation('['),
    //         Token::Bool(false),
    //         Token::Punctuation(']'),
    //         Token::Punctuation('}'),
    //     ];
    //     assert_eq!(expected, Token::tokenize_tokens(json));
    // }
}
