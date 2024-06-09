use std::{fmt, fmt::Display};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorCode {
    ExpectedToken,
    ExpectedDoubleQuote,
    ExpectedColon,
    ExpectedCommaOrEndWhileParsing(char),
    KeyMustBeAString,
    InvalidNumber,
    EndOfFileExpected,
    EndOfFileWhileParsing(char),
    EndOfFileWhileParsingValue,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorCode::ExpectedToken => {
                f.write_str("Expected a JSON object, array, string, number, bool, or null.")
            }
            ErrorCode::ExpectedDoubleQuote => f.write_str("Expected '\"'"),
            ErrorCode::ExpectedColon => f.write_str("Expected ':'"),
            ErrorCode::ExpectedCommaOrEndWhileParsing(end) => match end {
                ']' => f.write_str("Expected ',' or ']' while parsing array"),
                '}' => f.write_str("Expected ',' or '}' whiel parsing object"),
                _ => panic!("Only arrays or objects are supported"),
            },
            ErrorCode::KeyMustBeAString => f.write_str("Key must be a string"),
            ErrorCode::InvalidNumber => write!(f, "Invalid number"),
            ErrorCode::EndOfFileWhileParsing(c) => match c {
                ']' => f.write_str("End of file while parsing a list"),
                '}' => f.write_str("End of file while parsing an object"),
                _ => panic!("Only arrays or objects are supported"),
            },
            ErrorCode::EndOfFileExpected => f.write_str("End of file expected"),
            ErrorCode::EndOfFileWhileParsingValue => {
                f.write_str("End of file while parsing a value")
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Error {
    code: ErrorCode,
    line: usize,
    col: usize,
}

impl Error {
    pub fn new(code: ErrorCode, line: usize, col: usize) -> Self {
        Error { code, line, col }
    }
}
