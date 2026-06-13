use thiserror::Error;

pub(crate) type LibResult<T> = Result<T, LibError>;

#[derive(Debug, Error)]
pub(crate) enum LibError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    RuleViolation(String),
    #[error("数据库操作失败：{0}")]
    Db(#[from] rusqlite::Error),
}

impl LibError {
    pub(crate) fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub(crate) fn rule_violation(message: impl Into<String>) -> Self {
        Self::RuleViolation(message.into())
    }

    pub(crate) fn user_message(&self) -> String {
        self.to_string()
    }
}
