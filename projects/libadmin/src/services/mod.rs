use std::path::Path;

use chrono::Duration;
use rusqlite::{OptionalExtension, params};

use crate::db::open_conn;
use crate::utils::{db_err, hash_password, parse_date, today, valid_id};

pub(crate) struct ReaderInput<'a> {
    pub(crate) id: &'a str,
    pub(crate) name: &'a str,
    pub(crate) password: Option<&'a str>,
    pub(crate) gender: &'a str,
    pub(crate) profession: &'a str,
    pub(crate) max_borrow: i64,
    pub(crate) borrow_days: i64,
    pub(crate) remark: &'a str,
}

pub(crate) struct BookInput<'a> {
    pub(crate) id: &'a str,
    pub(crate) title: &'a str,
    pub(crate) category: &'a str,
    pub(crate) keywords: &'a str,
    pub(crate) status: Option<&'a str>,
    pub(crate) remark: &'a str,
}

pub(crate) struct AdminInput<'a> {
    pub(crate) id: &'a str,
    pub(crate) name: &'a str,
    pub(crate) password: Option<&'a str>,
    pub(crate) level: i64,
    pub(crate) remark: &'a str,
}

pub(crate) struct ExceptionInput<'a> {
    pub(crate) exception_type: &'a str,
    pub(crate) amount: f64,
    pub(crate) status: &'a str,
    pub(crate) reader_id: &'a str,
    pub(crate) book_id: &'a str,
    pub(crate) borrow_id: Option<&'a str>,
    pub(crate) remark: &'a str,
}

fn normalize_id(id: &str) -> String {
    id.trim().to_uppercase()
}

fn required_password<'a>(password: Option<&'a str>, message: &str) -> Result<&'a str, String> {
    password
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| message.to_string())
}

pub(crate) fn create_reader(path: &Path, input: ReaderInput<'_>) -> Result<(), String> {
    let id = normalize_id(input.id);
    if !valid_id(&id) {
        return Err("读者 ID 格式错误".to_string());
    }
    let password = required_password(input.password, "新增读者必须填写初始密码")?;
    let conn = open_conn(path).map_err(db_err)?;
    conn.execute(
        "INSERT INTO readers (id, name, gender, profession, max_borrow, borrow_days, password_hash, remark)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            input.name.trim(),
            input.gender.trim(),
            input.profession.trim(),
            input.max_borrow,
            input.borrow_days,
            hash_password(password),
            input.remark
        ],
    )
    .map_err(db_err)?;
    Ok(())
}

pub(crate) fn update_reader(path: &Path, input: ReaderInput<'_>) -> Result<(), String> {
    let id = normalize_id(input.id);
    if !valid_id(&id) {
        return Err("读者 ID 格式错误".to_string());
    }
    let conn = open_conn(path).map_err(db_err)?;
    let changed = if let Some(password) = input.password.map(str::trim).filter(|p| !p.is_empty()) {
        conn.execute(
            "UPDATE readers SET name = ?1, gender = ?2, profession = ?3, max_borrow = ?4,
             borrow_days = ?5, password_hash = ?6, remark = ?7 WHERE id = ?8",
            params![
                input.name.trim(),
                input.gender.trim(),
                input.profession.trim(),
                input.max_borrow,
                input.borrow_days,
                hash_password(password),
                input.remark,
                id
            ],
        )
    } else {
        conn.execute(
            "UPDATE readers SET name = ?1, gender = ?2, profession = ?3, max_borrow = ?4,
             borrow_days = ?5, remark = ?6 WHERE id = ?7",
            params![
                input.name.trim(),
                input.gender.trim(),
                input.profession.trim(),
                input.max_borrow,
                input.borrow_days,
                input.remark,
                id
            ],
        )
    }
    .map_err(db_err)?;
    if changed == 0 {
        return Err("未找到该读者".to_string());
    }
    Ok(())
}

pub(crate) fn create_book(path: &Path, input: BookInput<'_>) -> Result<(), String> {
    let id = normalize_id(input.id);
    if !valid_id(&id) {
        return Err("书籍 ID 格式错误".to_string());
    }
    let conn = open_conn(path).map_err(db_err)?;
    conn.execute(
        "INSERT INTO books (id, title, category, keywords, status, remark)
         VALUES (?1, ?2, ?3, ?4, 'available', ?5)",
        params![
            id,
            input.title.trim(),
            input.category.trim(),
            input.keywords.trim(),
            input.remark
        ],
    )
    .map_err(db_err)?;
    Ok(())
}

pub(crate) fn update_book(path: &Path, input: BookInput<'_>) -> Result<(), String> {
    let id = normalize_id(input.id);
    if !valid_id(&id) {
        return Err("书籍 ID 格式错误".to_string());
    }
    let status = input.status.unwrap_or("available").trim();
    let conn = open_conn(path).map_err(db_err)?;
    let changed = conn
        .execute(
            "UPDATE books SET title = ?1, category = ?2, keywords = ?3, status = ?4, remark = ?5 WHERE id = ?6",
            params![
                input.title.trim(),
                input.category.trim(),
                input.keywords.trim(),
                status,
                input.remark,
                id
            ],
        )
        .map_err(db_err)?;
    if changed == 0 {
        return Err("未找到该图书".to_string());
    }
    Ok(())
}

pub(crate) fn create_admin(path: &Path, input: AdminInput<'_>) -> Result<(), String> {
    let id = normalize_id(input.id);
    if !valid_id(&id) {
        return Err("管理员 ID 格式错误".to_string());
    }
    let password = required_password(input.password, "新增管理员必须填写初始密码")?;
    let conn = open_conn(path).map_err(db_err)?;
    conn.execute(
        "INSERT INTO admins (id, name, password_hash, level, remark) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            id,
            input.name.trim(),
            hash_password(password),
            input.level,
            input.remark
        ],
    )
    .map_err(db_err)?;
    Ok(())
}

pub(crate) fn update_admin(path: &Path, input: AdminInput<'_>) -> Result<(), String> {
    let id = normalize_id(input.id);
    if !valid_id(&id) {
        return Err("管理员 ID 格式错误".to_string());
    }
    let conn = open_conn(path).map_err(db_err)?;
    let changed = if let Some(password) = input.password.map(str::trim).filter(|p| !p.is_empty()) {
        conn.execute(
            "UPDATE admins SET name = ?1, password_hash = ?2, level = ?3, remark = ?4 WHERE id = ?5",
            params![
                input.name.trim(),
                hash_password(password),
                input.level,
                input.remark,
                id
            ],
        )
    } else {
        conn.execute(
            "UPDATE admins SET name = ?1, level = ?2, remark = ?3 WHERE id = ?4",
            params![input.name.trim(), input.level, input.remark, id],
        )
    }
    .map_err(db_err)?;
    if changed == 0 {
        return Err("未找到该管理员".to_string());
    }
    Ok(())
}

pub(crate) fn delete_admin(
    path: &Path,
    admin_id: &str,
    current_admin_id: &str,
) -> Result<(), String> {
    let id = normalize_id(admin_id);
    if id == "A001" {
        return Err("默认最高权限管理员不可删除".to_string());
    }
    if id == current_admin_id {
        return Err("不能删除当前登录账号".to_string());
    }
    let conn = open_conn(path).map_err(db_err)?;
    let changed = conn
        .execute("DELETE FROM admins WHERE id = ?1", params![id])
        .map_err(db_err)?;
    if changed == 0 {
        return Err("未找到该管理员".to_string());
    }
    Ok(())
}

pub(crate) fn create_exception(path: &Path, input: ExceptionInput<'_>) -> Result<(), String> {
    let borrow_id = input
        .borrow_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<i64>().ok());
    let conn = open_conn(path).map_err(db_err)?;
    conn.execute(
        "INSERT INTO exceptions (exception_type, amount, status, process_date, reader_id, book_id, borrow_id, remark)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            input.exception_type,
            input.amount,
            input.status,
            today().to_string(),
            normalize_id(input.reader_id),
            normalize_id(input.book_id),
            borrow_id,
            input.remark
        ],
    )
    .map_err(db_err)?;
    Ok(())
}

pub(crate) fn create_borrow(
    path: &Path,
    reader_id: &str,
    book_id: &str,
    remark: &str,
) -> Result<(), String> {
    if !valid_id(reader_id) || !valid_id(book_id) {
        return Err("读者 ID 或书籍 ID 格式错误".to_string());
    }
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let (max_borrow, borrow_days): (i64, i64) = tx
        .query_row(
            "SELECT max_borrow, borrow_days FROM readers WHERE id = ?1",
            params![reader_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(db_err)?
        .ok_or_else(|| "读者不存在".to_string())?;
    let active: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM borrows WHERE reader_id = ?1 AND returned = 0",
            params![reader_id],
            |row| row.get(0),
        )
        .map_err(db_err)?;
    if active >= max_borrow {
        return Err("借书数量已达上限，无法继续借阅".to_string());
    }
    let status: Option<String> = tx
        .query_row(
            "SELECT status FROM books WHERE id = ?1",
            params![book_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_err)?;
    match status.as_deref() {
        Some("available") => {}
        Some("borrowed") => return Err("书籍已借出".to_string()),
        Some("discarded") => return Err("书籍已报废，无法借阅".to_string()),
        Some(_) => return Err("书籍状态异常，无法借阅".to_string()),
        None => return Err("书籍不存在".to_string()),
    }
    let borrow_date = today();
    let due_date = borrow_date + Duration::days(borrow_days);
    tx.execute(
        "INSERT INTO borrows (reader_id, book_id, borrow_date, due_date, returned, renew_count, remark)
         VALUES (?1, ?2, ?3, ?4, 0, 0, ?5)",
        params![
            reader_id,
            book_id,
            borrow_date.to_string(),
            due_date.to_string(),
            remark
        ],
    )
    .map_err(db_err)?;
    tx.execute(
        "UPDATE books SET status = 'borrowed' WHERE id = ?1",
        params![book_id],
    )
    .map_err(db_err)?;
    tx.commit().map_err(db_err)?;
    Ok(())
}

pub(crate) fn complete_return(
    path: &Path,
    actor_reader: Option<&str>,
    borrow_id: i64,
) -> Result<(), String> {
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let borrow: Option<(String, String, String, i64)> = tx
        .query_row(
            "SELECT reader_id, book_id, due_date, returned FROM borrows WHERE id = ?1",
            params![borrow_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(db_err)?;
    let Some((reader_id, book_id, due_date_text, returned)) = borrow else {
        return Err("借阅记录不存在".to_string());
    };
    if let Some(actor_reader) = actor_reader
        && actor_reader != reader_id
    {
        return Err("只能归还本人借阅的图书".to_string());
    }
    if returned != 0 {
        return Err("该图书已经归还".to_string());
    }
    let due_date = parse_date(&due_date_text)?;
    let now = today();
    if now > due_date {
        let existing: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM exceptions WHERE borrow_id = ?1 AND status != '已处理'",
                params![borrow_id],
                |row| row.get(0),
            )
            .map_err(db_err)?;
        if existing == 0 {
            let overdue_days = (now - due_date).num_days().max(1);
            tx.execute(
                "INSERT INTO exceptions (exception_type, amount, status, process_date, reader_id, book_id, borrow_id, remark)
                 VALUES ('超期', ?1, '待处理', ?2, ?3, ?4, ?5, ?6)",
                params![
                    overdue_days as f64,
                    now.to_string(),
                    reader_id,
                    book_id,
                    borrow_id,
                    format!("逾期 {overdue_days} 天，按 1 元/天生成赔偿")
                ],
            )
            .map_err(db_err)?;
        }
        tx.commit().map_err(db_err)?;
        return Err("图书已超期，已生成待处理异常记录，需管理员处理后完成归还".to_string());
    }
    finish_return_in_transaction(
        &tx,
        borrow_id,
        &reader_id,
        &book_id,
        &due_date_text,
        "正常归还",
    )?;
    tx.commit().map_err(db_err)?;
    Ok(())
}

fn finish_return_in_transaction(
    tx: &rusqlite::Transaction<'_>,
    borrow_id: i64,
    reader_id: &str,
    book_id: &str,
    due_date: &str,
    remark: &str,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO returns (reader_id, book_id, borrow_id, return_date, due_date, remark)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            reader_id,
            book_id,
            borrow_id,
            today().to_string(),
            due_date,
            remark
        ],
    )
    .map_err(db_err)?;
    tx.execute(
        "UPDATE borrows SET returned = 1 WHERE id = ?1",
        params![borrow_id],
    )
    .map_err(db_err)?;
    tx.execute(
        "UPDATE books SET status = 'available' WHERE id = ?1 AND status != 'discarded'",
        params![book_id],
    )
    .map_err(db_err)?;
    Ok(())
}

pub(crate) fn renew_borrow(
    path: &Path,
    actor_reader: Option<&str>,
    borrow_id: i64,
) -> Result<(), String> {
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let borrow: Option<(String, String, i64, i64)> = tx
        .query_row(
            "SELECT br.reader_id, br.due_date, br.returned, br.renew_count
             FROM borrows br WHERE br.id = ?1",
            params![borrow_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(db_err)?;
    let Some((reader_id, due_date_text, returned, renew_count)) = borrow else {
        return Err("借阅记录不存在".to_string());
    };
    if let Some(actor_reader) = actor_reader
        && actor_reader != reader_id
    {
        return Err("只能续借本人借阅的图书".to_string());
    }
    if returned != 0 {
        return Err("已归还图书不能续借".to_string());
    }
    if renew_count >= 2 {
        return Err("单本图书续借次数已达上限".to_string());
    }
    let due_date = parse_date(&due_date_text)?;
    if today() > due_date {
        return Err("书籍已超期，请先办理赔偿手续".to_string());
    }
    let borrow_days: i64 = tx
        .query_row(
            "SELECT borrow_days FROM readers WHERE id = ?1",
            params![reader_id],
            |row| row.get(0),
        )
        .map_err(db_err)?;
    let new_due = due_date + Duration::days(borrow_days);
    tx.execute(
        "UPDATE borrows SET due_date = ?1, renew_count = renew_count + 1,
         remark = TRIM(remark || ' 续借至' || ?1) WHERE id = ?2",
        params![new_due.to_string(), borrow_id],
    )
    .map_err(db_err)?;
    tx.commit().map_err(db_err)?;
    Ok(())
}

pub(crate) fn resolve_exception(path: &Path, exception_id: i64) -> Result<(), String> {
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let item: Option<(String, String, Option<i64>, String)> = tx
        .query_row(
            "SELECT exception_type, book_id, borrow_id, status FROM exceptions WHERE id = ?1",
            params![exception_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(db_err)?;
    let Some((exception_type, book_id, borrow_id, status)) = item else {
        return Err("异常记录不存在".to_string());
    };
    if status == "已处理" {
        return Err("异常记录已经处理".to_string());
    }
    tx.execute(
        "UPDATE exceptions SET status = '已处理', process_date = ?1 WHERE id = ?2",
        params![today().to_string(), exception_id],
    )
    .map_err(db_err)?;

    if exception_type.contains("丢失") {
        tx.execute(
            "UPDATE books SET status = 'discarded' WHERE id = ?1",
            params![book_id],
        )
        .map_err(db_err)?;
        if let Some(borrow_id) = borrow_id {
            tx.execute(
                "UPDATE borrows SET returned = 1 WHERE id = ?1 AND returned = 0",
                params![borrow_id],
            )
            .map_err(db_err)?;
        }
    } else if let Some(borrow_id) = borrow_id {
        let active: Option<(String, String, String)> = tx
            .query_row(
                "SELECT reader_id, book_id, due_date FROM borrows WHERE id = ?1 AND returned = 0",
                params![borrow_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(db_err)?;
        if let Some((reader_id, book_id, due_date)) = active {
            finish_return_in_transaction(
                &tx,
                borrow_id,
                &reader_id,
                &book_id,
                &due_date,
                "异常处理后归还",
            )?;
        }
    } else if exception_type.contains("损坏") {
        tx.execute(
            "UPDATE books SET status = 'available' WHERE id = ?1 AND status = 'borrowed'",
            params![book_id],
        )
        .map_err(db_err)?;
    }
    tx.commit().map_err(db_err)?;
    Ok(())
}

pub(crate) fn delete_reader_if_clear(path: &Path, reader_id: &str) -> Result<(), String> {
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let active: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM borrows WHERE reader_id = ?1 AND returned = 0",
            params![reader_id],
            |row| row.get(0),
        )
        .map_err(db_err)?;
    if active > 0 {
        return Err("尚有未归还图书，无法注销或删除读者".to_string());
    }
    let unpaid: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM exceptions WHERE reader_id = ?1 AND status != '已处理'",
            params![reader_id],
            |row| row.get(0),
        )
        .map_err(db_err)?;
    if unpaid > 0 {
        return Err("尚有未处理赔偿记录，无法注销或删除读者".to_string());
    }
    let changed = tx
        .execute("DELETE FROM readers WHERE id = ?1", params![reader_id])
        .map_err(db_err)?;
    if changed == 0 {
        return Err("未找到该读者".to_string());
    }
    tx.commit().map_err(db_err)?;
    Ok(())
}

pub(crate) fn delete_book_if_available(path: &Path, book_id: &str) -> Result<(), String> {
    let mut conn = open_conn(path).map_err(db_err)?;
    let tx = conn.transaction().map_err(db_err)?;
    let active: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM borrows WHERE book_id = ?1 AND returned = 0",
            params![book_id],
            |row| row.get(0),
        )
        .map_err(db_err)?;
    if active > 0 {
        return Err("书籍当前被借阅，无法删除".to_string());
    }
    let changed = tx
        .execute("DELETE FROM books WHERE id = ?1", params![book_id])
        .map_err(db_err)?;
    if changed == 0 {
        return Err("未找到该图书".to_string());
    }
    tx.commit().map_err(db_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use rusqlite::params;
    use uuid::Uuid;

    use super::*;
    use crate::db::{init_database, open_conn};

    struct TestDb {
        path: PathBuf,
    }

    impl TestDb {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("libadmin-test-{}.db", Uuid::new_v4()));
            init_database(&path).expect("test database should initialize");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDb {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    #[test]
    fn borrow_renew_return_flow_updates_database_state() {
        let db = TestDb::new();
        let book_id = "B0038";

        create_borrow(db.path(), "R001", book_id, "测试借阅").expect("borrow should succeed");

        let conn = open_conn(db.path()).expect("database should open");
        let (borrow_id, status, renew_count): (i64, String, i64) = conn
            .query_row(
                "SELECT br.id, b.status, br.renew_count
                 FROM borrows br JOIN books b ON br.book_id = b.id
                 WHERE br.reader_id = ?1 AND br.book_id = ?2 AND br.returned = 0",
                params!["R001", book_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("active borrow should exist");
        assert_eq!(status, "borrowed");
        assert_eq!(renew_count, 0);
        drop(conn);

        renew_borrow(db.path(), Some("R001"), borrow_id).expect("renew should succeed");
        let conn = open_conn(db.path()).expect("database should open");
        let renewed_count: i64 = conn
            .query_row(
                "SELECT renew_count FROM borrows WHERE id = ?1",
                params![borrow_id],
                |row| row.get(0),
            )
            .expect("borrow row should exist");
        assert_eq!(renewed_count, 1);
        drop(conn);

        complete_return(db.path(), Some("R001"), borrow_id).expect("return should succeed");
        let conn = open_conn(db.path()).expect("database should open");
        let (returned, final_status): (i64, String) = conn
            .query_row(
                "SELECT br.returned, b.status
                 FROM borrows br JOIN books b ON br.book_id = b.id
                 WHERE br.id = ?1",
                params![borrow_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("borrow row should remain for history");
        let return_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM returns WHERE borrow_id = ?1",
                params![borrow_id],
                |row| row.get(0),
            )
            .expect("return count should be readable");

        assert_eq!(returned, 1);
        assert_eq!(final_status, "available");
        assert_eq!(return_count, 1);
    }

    #[test]
    fn delete_book_rejects_book_with_active_borrow() {
        let db = TestDb::new();
        let book_id = "B0039";

        create_borrow(db.path(), "R001", book_id, "测试借阅").expect("borrow should succeed");
        let err = delete_book_if_available(db.path(), book_id)
            .expect_err("active borrowed book should not be deleted");

        assert!(err.contains("当前被借阅"));
    }

    #[test]
    fn reader_crud_services_manage_records() {
        let db = TestDb::new();

        create_reader(
            db.path(),
            ReaderInput {
                id: "r900",
                name: "测试读者",
                password: Some("reader-pass"),
                gender: "其他",
                profession: "测试员",
                max_borrow: 7,
                borrow_days: 45,
                remark: "初始备注",
            },
        )
        .expect("reader should be created");

        let conn = open_conn(db.path()).expect("database should open");
        let (name, max_borrow, borrow_days, password_hash): (String, i64, i64, String) = conn
            .query_row(
                "SELECT name, max_borrow, borrow_days, password_hash FROM readers WHERE id = 'R900'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("reader should exist");
        assert_eq!(name, "测试读者");
        assert_eq!(max_borrow, 7);
        assert_eq!(borrow_days, 45);
        assert_eq!(password_hash, hash_password("reader-pass"));
        drop(conn);

        update_reader(
            db.path(),
            ReaderInput {
                id: "R900",
                name: "更新读者",
                password: None,
                gender: "男",
                profession: "工程师",
                max_borrow: 8,
                borrow_days: 60,
                remark: "更新备注",
            },
        )
        .expect("reader should be updated");

        let conn = open_conn(db.path()).expect("database should open");
        let (name, profession, password_hash): (String, String, String) = conn
            .query_row(
                "SELECT name, profession, password_hash FROM readers WHERE id = 'R900'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("reader should remain");
        assert_eq!(name, "更新读者");
        assert_eq!(profession, "工程师");
        assert_eq!(password_hash, hash_password("reader-pass"));
        drop(conn);

        delete_reader_if_clear(db.path(), "R900").expect("clear reader should be deleted");
        let conn = open_conn(db.path()).expect("database should open");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM readers WHERE id = 'R900'",
                [],
                |row| row.get(0),
            )
            .expect("reader count should be readable");
        assert_eq!(count, 0);
    }

    #[test]
    fn book_crud_services_manage_records() {
        let db = TestDb::new();

        create_book(
            db.path(),
            BookInput {
                id: "b900",
                title: "测试图书",
                category: "测试",
                keywords: "rust;test",
                status: None,
                remark: "初始备注",
            },
        )
        .expect("book should be created");

        let conn = open_conn(db.path()).expect("database should open");
        let status: String = conn
            .query_row("SELECT status FROM books WHERE id = 'B900'", [], |row| {
                row.get(0)
            })
            .expect("book should exist");
        assert_eq!(status, "available");
        drop(conn);

        update_book(
            db.path(),
            BookInput {
                id: "B900",
                title: "更新图书",
                category: "工程",
                keywords: "rust;updated",
                status: Some("discarded"),
                remark: "更新备注",
            },
        )
        .expect("book should be updated");

        let conn = open_conn(db.path()).expect("database should open");
        let (title, status): (String, String) = conn
            .query_row(
                "SELECT title, status FROM books WHERE id = 'B900'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("book should remain");
        assert_eq!(title, "更新图书");
        assert_eq!(status, "discarded");
        drop(conn);

        delete_book_if_available(db.path(), "B900").expect("clear book should be deleted");
        let conn = open_conn(db.path()).expect("database should open");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM books WHERE id = 'B900'", [], |row| {
                row.get(0)
            })
            .expect("book count should be readable");
        assert_eq!(count, 0);
    }

    #[test]
    fn admin_crud_services_enforce_delete_guards() {
        let db = TestDb::new();

        create_admin(
            db.path(),
            AdminInput {
                id: "a900",
                name: "测试管理员",
                password: Some("admin-pass"),
                level: 4,
                remark: "初始备注",
            },
        )
        .expect("admin should be created");

        update_admin(
            db.path(),
            AdminInput {
                id: "A900",
                name: "更新管理员",
                password: None,
                level: 6,
                remark: "更新备注",
            },
        )
        .expect("admin should be updated");

        let conn = open_conn(db.path()).expect("database should open");
        let (name, level, password_hash): (String, i64, String) = conn
            .query_row(
                "SELECT name, level, password_hash FROM admins WHERE id = 'A900'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("admin should exist");
        assert_eq!(name, "更新管理员");
        assert_eq!(level, 6);
        assert_eq!(password_hash, hash_password("admin-pass"));
        drop(conn);

        let err = delete_admin(db.path(), "A001", "A900").expect_err("default admin is protected");
        assert!(err.contains("默认最高权限管理员"));

        let err = delete_admin(db.path(), "A900", "A900").expect_err("current admin is protected");
        assert!(err.contains("不能删除当前登录账号"));

        delete_admin(db.path(), "A900", "A001").expect("non-current admin should be deleted");
        let conn = open_conn(db.path()).expect("database should open");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM admins WHERE id = 'A900'", [], |row| {
                row.get(0)
            })
            .expect("admin count should be readable");
        assert_eq!(count, 0);
    }

    #[test]
    fn create_exception_normalizes_ids_and_records_details() {
        let db = TestDb::new();

        create_exception(
            db.path(),
            ExceptionInput {
                exception_type: "损坏",
                amount: 12.5,
                status: "待处理",
                reader_id: "r001",
                book_id: "b0038",
                borrow_id: Some("not-a-number"),
                remark: "封面损坏",
            },
        )
        .expect("exception should be created");

        let conn = open_conn(db.path()).expect("database should open");
        let (exception_type, amount, status, reader_id, book_id, borrow_id, remark): (
            String,
            f64,
            String,
            String,
            String,
            Option<i64>,
            String,
        ) = conn
            .query_row(
                "SELECT exception_type, amount, status, reader_id, book_id, borrow_id, remark
                 FROM exceptions
                 WHERE reader_id = 'R001' AND book_id = 'B0038'
                 ORDER BY id DESC
                 LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .expect("exception should exist");

        assert_eq!(exception_type, "损坏");
        assert_eq!(amount, 12.5);
        assert_eq!(status, "待处理");
        assert_eq!(reader_id, "R001");
        assert_eq!(book_id, "B0038");
        assert_eq!(borrow_id, None);
        assert_eq!(remark, "封面损坏");
    }
}
