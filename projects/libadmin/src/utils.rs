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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_id_accepts_expected_ascii_identifiers() {
        assert!(valid_id("R001"));
        assert!(valid_id("reader_01"));
        assert!(valid_id("book-2024"));
    }

    #[test]
    fn valid_id_rejects_invalid_identifiers() {
        assert!(!valid_id("A"));
        assert!(!valid_id("reader 01"));
        assert!(!valid_id("读者001"));
        assert!(!valid_id("x".repeat(33).as_str()));
    }

    #[test]
    fn parse_date_uses_iso_day_format() {
        assert!(parse_date("2026-06-12").is_ok());
        assert!(parse_date("2026/06/12").is_err());
    }

    #[test]
    fn hash_password_is_salted_and_stable() {
        let first = hash_password("secret");
        let second = hash_password("secret");

        assert_eq!(first, second);
        assert_ne!(first, hash_password("other"));
        assert_eq!(first.len(), 64);
    }
}
