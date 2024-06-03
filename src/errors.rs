use std::{fmt, fmt::Display};

#[derive(Debug, PartialEq)]
pub enum ErrorCode {
    ExpectedToken,
    ExpectedDoubleQuote,
    InvalidNumber(String),
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorCode::ExpectedToken => {
                f.write_str("Expected a JSON object, array, string, number, bool, or null.")
            }
            ErrorCode::ExpectedDoubleQuote => f.write_str("Expected '\"'"),
            ErrorCode::InvalidNumber(value) => write!(f, "Invalid number: {value}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Error {
    code: ErrorCode,
    line: usize,
}

impl Error {
    pub fn new(code: ErrorCode, line: usize) -> Self {
        Error { code, line }
    }

    pub fn get_line(&self) -> usize {
        self.line
    }

    pub fn get_code(&self) -> &ErrorCode {
        &self.code
    }
}
