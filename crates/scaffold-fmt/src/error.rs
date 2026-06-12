use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("unterminated string starting at byte {offset}")]
    UnterminatedString { offset: usize },
    #[error("unexpected closing delimiter `{found}` at byte {offset}")]
    UnexpectedClose { found: char, offset: usize },
    #[error("expected closing delimiter `{expected}` before end of file")]
    UnclosedList { expected: char },
    #[error("quote shorthand at byte {offset} is missing an expression")]
    MissingQuotedExpression { offset: usize },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl FormatError {
    #[must_use]
    pub const fn primary_offset(&self) -> usize {
        match self {
            Self::UnterminatedString { offset }
            | Self::UnexpectedClose { offset, .. }
            | Self::MissingQuotedExpression { offset } => *offset,
            Self::UnclosedList { .. } | Self::Io(_) => 0,
        }
    }
}
