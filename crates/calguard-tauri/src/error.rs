use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppErrorDto {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
}

#[derive(Debug, Error)]
#[error("{message}")]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
}

impl AppError {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details,
            recoverable: true,
        }
    }

    pub fn db(error: impl std::fmt::Display) -> Self {
        Self::new("DB_ERROR", "本地数据库访问失败。", Some(error.to_string()))
    }

    pub fn file_not_found(details: impl Into<String>) -> Self {
        Self::new(
            "FILE_NOT_FOUND",
            "找不到该文件，请检查路径是否正确。",
            Some(details.into()),
        )
    }

    pub fn file_too_large(details: impl Into<String>) -> Self {
        Self::new(
            "FILE_TOO_LARGE",
            "该日历文件过大，请换一个较小的文件。",
            Some(details.into()),
        )
    }

    pub fn invalid_url(details: impl Into<String>) -> Self {
        Self::new(
            "INVALID_URL",
            "请输入有效的 HTTP/HTTPS URL。",
            Some(details.into()),
        )
    }

    pub fn remote_failed(details: impl Into<String>) -> Self {
        Self::new(
            "REMOTE_FETCH_FAILED",
            "无法下载远程日历，请检查网络或 URL。",
            Some(details.into()),
        )
    }

    pub fn remote_timeout() -> Self {
        Self::new("REMOTE_TIMEOUT", "远程日历响应超时，请稍后重试。", None)
    }

    pub fn parse_failed(details: impl Into<String>) -> Self {
        Self::new(
            "ICS_PARSE_FAILED",
            "无法解析该日历文件。",
            Some(details.into()),
        )
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        AppErrorDto {
            code: self.code.clone(),
            message: self.message.clone(),
            details: self.details.clone(),
            recoverable: self.recoverable,
        }
        .serialize(serializer)
    }
}
