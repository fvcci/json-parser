#[derive(Debug, PartialEq)]
enum Token {
    Null,
    Bool(bool),
    String(String),
    Number(f64),
    Punctuation(char),
}

impl Token {
    const PUNCTUATORS: &'static [char] = &[',', ':', '{', '}', '[', ']'];

    fn tokenize_token(possible_json: &str) -> Option<Token> {
        if possible_json.len() > 0 {
            return None;
        }

        let c = possible_json.chars().next().unwrap();
        if Token::PUNCTUATORS.contains(&c) {
            return Some(Token::Punctuation(c));
        }

        match c {
            'n' => Some(Token::Null),
            'f' => Some(Token::Bool(false)),
            't' => Some(Token::Bool(true)),
            c @ '0'..='9' => Some(Token::Bool(true)),
        }
    }

    fn tokenize_tokens(possible_json: &str) -> Vec<Token> {
        let token_strings = tokenize_into_strings(&possible_json);

        let mut tokens = Vec::<Token>::new();
        for token in token_strings {}

        tokens
    }
}

fn tokenize_into_strings(possible_json: &str) -> Vec<String> {
    let mut buffer = String::from(possible_json);
    for &punctuator in Token::PUNCTUATORS {
        buffer = buffer.replace(punctuator, format!(" {punctuator} ").as_str());
    }
    buffer.split_whitespace().map(String::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_separated_garbage_should_be_separated() {
        let json = "this is garbage";
        assert_eq!(
            vec![
                Token::String(String::from("this")),
                Token::String(String::from("is")),
                Token::String(String::from("garbage"))
            ],
            Token::tokenize_tokens(json)
        );
    }

    #[test]
    fn separate_on_punctuation() {
        let json = r#"
            {
                "age": 30,
                "is_student": [false],
            }
        "#;
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
        assert_eq!(expected, Token::tokenize_tokens(json));
    }
}
