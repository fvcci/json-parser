// TODO unify error message system and incorporate line numbers
#[derive(Debug, PartialEq)]
pub struct Message {
    line_number: u64,
    line_contents: String,
    description: String,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedEndOfFile(Message),
    Expected(Message),
    MatchingOpeningPairNotFound(Message),
    ExpectedLiteral(Message),
    InvalidString(Message),
    InvalidNumber(Message),
}
