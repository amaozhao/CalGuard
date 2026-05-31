use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIssue {
    pub line: Option<usize>,
    pub context: String,
    pub message: String,
}

impl ParseIssue {
    pub fn new(
        line: Option<usize>,
        context: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            line,
            context: context.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("ICS parse failed: {0}")]
    IcsParse(String),
    #[error("unsupported timezone: {0}")]
    UnsupportedTimezone(String),
    #[error("invalid date or time: {0}")]
    InvalidDateTime(String),
    #[error("invalid recurrence rule: {0}")]
    InvalidRecurrenceRule(String),
    #[error("serialization failed: {0}")]
    Serialization(String),
}
