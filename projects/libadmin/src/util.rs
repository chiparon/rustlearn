use chrono::{Local, NaiveDate};
use sha2::{Digest, Sha256};

pub(crate) fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"libadmin:");
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn today() -> NaiveDate {
    Local::now().date_naive()
}

pub(crate) fn parse_date(value: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| format!("日期格式无效：{value}"))
}

pub(crate) fn db_err(err: rusqlite::Error) -> String {
    format!("数据库操作失败：{err}")
}

pub(crate) fn valid_id(id: &str) -> bool {
    let len = id.len();
    (2..=32).contains(&len)
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub(crate) fn matches_text(value: &str, needle: &Option<String>) -> bool {
    let Some(needle) = needle.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return true;
    };
    value.to_lowercase().contains(&needle.to_lowercase())
}

pub(crate) fn matches_status(value: &str, status: &Option<String>) -> bool {
    let Some(status) = status.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return true;
    };
    value == status
}
