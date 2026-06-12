use chrono::Duration;
use rusqlite::{Connection, OptionalExtension, params};
use std::{fs, path::Path};

use crate::{
    models::{Admin, Book, BorrowView, ExceptionView, Reader, ReturnView},
    util::{db_err, hash_password, parse_date, today, valid_id},
};

pub(crate) fn open_conn(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

pub(crate) fn init_database(path: &Path) -> rusqlite::Result<()> {
    let mut conn = open_conn(path)?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS readers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            gender TEXT NOT NULL DEFAULT '',
            profession TEXT NOT NULL DEFAULT '',
            max_borrow INTEGER NOT NULL DEFAULT 5,
            borrow_days INTEGER NOT NULL DEFAULT 30,
            password_hash TEXT NOT NULL,
            remark TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS books (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            category TEXT NOT NULL,
            keywords TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'available',
            remark TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS admins (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            level INTEGER NOT NULL DEFAULT 1,
            remark TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS borrows (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            reader_id TEXT NOT NULL,
            book_id TEXT NOT NULL,
            borrow_date TEXT NOT NULL,
            due_date TEXT NOT NULL,
            returned INTEGER NOT NULL DEFAULT 0,
            renew_count INTEGER NOT NULL DEFAULT 0,
            remark TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS returns (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            reader_id TEXT NOT NULL,
            book_id TEXT NOT NULL,
            borrow_id INTEGER NOT NULL,
            return_date TEXT NOT NULL,
            due_date TEXT NOT NULL,
            remark TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS exceptions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            exception_type TEXT NOT NULL,
            amount REAL NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT '待处理',
            process_date TEXT NOT NULL,
            reader_id TEXT NOT NULL,
            book_id TEXT NOT NULL,
            borrow_id INTEGER,
            remark TEXT NOT NULL DEFAULT ''
        );
        CREATE INDEX IF NOT EXISTS idx_books_status ON books(status);
        CREATE INDEX IF NOT EXISTS idx_borrows_reader ON borrows(reader_id, returned);
        CREATE INDEX IF NOT EXISTS idx_borrows_book ON borrows(book_id, returned);
        CREATE INDEX IF NOT EXISTS idx_exceptions_reader ON exceptions(reader_id, status);
        ",
    )?;
    seed_database(&mut conn)?;
    Ok(())
}

fn seed_database(conn: &mut Connection) -> rusqlite::Result<()> {
    if conn.query_row("SELECT COUNT(*) FROM readers", [], |row| {
        row.get::<_, i64>(0)
    })? == 0
    {
        let jobs = ["学生", "教师", "工程师", "医生", "职员", "自由职业"];
        let tx = conn.transaction()?;
        for i in 1..=80 {
            tx.execute(
                "INSERT INTO readers (id, name, gender, profession, max_borrow, borrow_days, password_hash, remark)
                 VALUES (?1, ?2, ?3, ?4, 5, 30, ?5, ?6)",
                params![
                    format!("R{i:03}"),
                    format!("读者{i:03}"),
                    if i % 2 == 0 { "女" } else { "男" },
                    jobs[(i as usize - 1) % jobs.len()],
                    hash_password(&format!("reader{i:03}")),
                    "系统初始化读者"
                ],
            )?;
        }
        tx.commit()?;
    }

    if conn.query_row("SELECT COUNT(*) FROM books", [], |row| row.get::<_, i64>(0))? == 0 {
        let categories = [
            "文学",
            "计算机",
            "历史",
            "经济",
            "艺术",
            "教育",
            "医学",
            "工程",
        ];
        let tx = conn.transaction()?;
        for i in 1..=220 {
            let category = categories[(i as usize - 1) % categories.len()];
            tx.execute(
                "INSERT INTO books (id, title, category, keywords, status, remark)
                 VALUES (?1, ?2, ?3, ?4, 'available', ?5)",
                params![
                    format!("B{i:04}"),
                    format!("{category}馆藏精选{i:03}"),
                    category,
                    format!("{category};馆藏;实验数据;第{i:03}册"),
                    "系统初始化图书"
                ],
            )?;
        }
        tx.commit()?;
    }

    if conn.query_row("SELECT COUNT(*) FROM admins", [], |row| {
        row.get::<_, i64>(0)
    })? == 0
    {
        let admins = [
            ("A001", "总管理员", "admin123", 9, "默认最高权限账号"),
            ("A002", "借还管理员", "admin123", 5, "负责借还业务"),
            ("A003", "馆藏管理员", "admin123", 5, "负责图书维护"),
            ("A004", "异常管理员", "admin123", 5, "负责赔偿处理"),
        ];
        let tx = conn.transaction()?;
        for (id, name, password, level, remark) in admins {
            tx.execute(
                "INSERT INTO admins (id, name, password_hash, level, remark)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, name, hash_password(password), level, remark],
            )?;
        }
        tx.commit()?;
    }

    if conn.query_row("SELECT COUNT(*) FROM borrows", [], |row| {
        row.get::<_, i64>(0)
    })? == 0
    {
        let now = today();
        let tx = conn.transaction()?;
        for i in 1..=20 {
            let reader_id = format!("R{i:03}");
            let book_id = format!("B{i:04}");
            let borrow_date = now - Duration::days(50 - i as i64);
            let due_date = borrow_date + Duration::days(30);
            tx.execute(
                "INSERT INTO borrows (reader_id, book_id, borrow_date, due_date, returned, renew_count, remark)
                 VALUES (?1, ?2, ?3, ?4, 1, 0, ?5)",
                params![
                    reader_id,
                    book_id,
                    borrow_date.to_string(),
                    due_date.to_string(),
                    "初始化历史借阅"
                ],
            )?;
            let borrow_id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO returns (reader_id, book_id, borrow_id, return_date, due_date, remark)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    format!("R{i:03}"),
                    format!("B{i:04}"),
                    borrow_id,
                    (due_date - Duration::days((i % 5) as i64)).to_string(),
                    due_date.to_string(),
                    "初始化归还记录"
                ],
            )?;
        }
        for i in 21..=32 {
            let reader_id = format!("R{i:03}");
            let book_id = format!("B{i:04}");
            let borrow_date = now - Duration::days((i % 18 + 3) as i64);
            let due_date = borrow_date + Duration::days(30);
            tx.execute(
                "INSERT INTO borrows (reader_id, book_id, borrow_date, due_date, returned, renew_count, remark)
                 VALUES (?1, ?2, ?3, ?4, 0, 0, ?5)",
                params![
                    reader_id,
                    book_id,
                    borrow_date.to_string(),
                    due_date.to_string(),
                    "初始化在借记录"
                ],
            )?;
            tx.execute(
                "UPDATE books SET status = 'borrowed' WHERE id = ?1",
                params![book_id],
            )?;
        }
        for i in 33..=37 {
            let reader_id = format!("R{i:03}");
            let book_id = format!("B{i:04}");
            let borrow_date = now - Duration::days(45 + i as i64 % 4);
            let due_date = borrow_date + Duration::days(30);
            tx.execute(
                "INSERT INTO borrows (reader_id, book_id, borrow_date, due_date, returned, renew_count, remark)
                 VALUES (?1, ?2, ?3, ?4, 0, 0, ?5)",
                params![
                    reader_id,
                    book_id,
                    borrow_date.to_string(),
                    due_date.to_string(),
                    "初始化超期记录"
                ],
            )?;
            let borrow_id = tx.last_insert_rowid();
            tx.execute(
                "UPDATE books SET status = 'borrowed' WHERE id = ?1",
                params![book_id],
            )?;
            tx.execute(
                "INSERT INTO exceptions (exception_type, amount, status, process_date, reader_id, book_id, borrow_id, remark)
                 VALUES ('超期', ?1, '待处理', ?2, ?3, ?4, ?5, ?6)",
                params![
                    ((now - due_date).num_days().max(1)) as f64,
                    now.to_string(),
                    format!("R{i:03}"),
                    format!("B{i:04}"),
                    borrow_id,
                    "初始化超期赔偿"
                ],
            )?;
        }
        tx.commit()?;
    }

    Ok(())
}

pub(crate) fn ensure_daily_backup(db_path: &Path) -> rusqlite::Result<()> {
    let Some(data_dir) = db_path.parent() else {
        return Ok(());
    };
    let backup_dir = data_dir.join("backups");
    let _ = fs::create_dir_all(&backup_dir);
    let backup_path = backup_dir.join(format!("libadmin-{}.db", today()));
    if !backup_path.exists() {
        let conn = open_conn(db_path)?;
        vacuum_into(&conn, &backup_path)?;
    }
    Ok(())
}

pub(crate) fn vacuum_into(conn: &Connection, path: &Path) -> rusqlite::Result<()> {
    let sql_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .replace('\'', "''");
    conn.execute_batch(&format!("VACUUM INTO '{sql_path}';"))
}

pub(crate) fn list_readers(path: &Path) -> rusqlite::Result<Vec<Reader>> {
    let conn = open_conn(path)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, gender, profession, max_borrow, borrow_days, remark
         FROM readers ORDER BY id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Reader {
            id: row.get(0)?,
            name: row.get(1)?,
            gender: row.get(2)?,
            profession: row.get(3)?,
            max_borrow: row.get(4)?,
            borrow_days: row.get(5)?,
            remark: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub(crate) fn get_reader(path: &Path, id: &str) -> rusqlite::Result<Option<Reader>> {
    let conn = open_conn(path)?;
    conn.query_row(
        "SELECT id, name, gender, profession, max_borrow, borrow_days, remark FROM readers WHERE id = ?1",
        params![id],
        |row| {
            Ok(Reader {
                id: row.get(0)?,
                name: row.get(1)?,
                gender: row.get(2)?,
                profession: row.get(3)?,
                max_borrow: row.get(4)?,
                borrow_days: row.get(5)?,
                remark: row.get(6)?,
            })
        },
    )
    .optional()
}

pub(crate) fn list_books(path: &Path) -> rusqlite::Result<Vec<Book>> {
    let conn = open_conn(path)?;
    let mut stmt = conn
        .prepare("SELECT id, title, category, keywords, status, remark FROM books ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(Book {
            id: row.get(0)?,
            title: row.get(1)?,
            category: row.get(2)?,
            keywords: row.get(3)?,
            status: row.get(4)?,
            remark: row.get(5)?,
        })
    })?;
    rows.collect()
}

pub(crate) fn list_admins(path: &Path) -> rusqlite::Result<Vec<Admin>> {
    let conn = open_conn(path)?;
    let mut stmt = conn.prepare("SELECT id, name, level, remark FROM admins ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(Admin {
            id: row.get(0)?,
            name: row.get(1)?,
            level: row.get(2)?,
            remark: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub(crate) fn list_active_borrows(
    path: &Path,
    reader_id: Option<&str>,
) -> rusqlite::Result<Vec<BorrowView>> {
    let conn = open_conn(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT br.id, br.reader_id, COALESCE(r.name, ''), br.book_id, COALESCE(b.title, ''),
               br.borrow_date, br.due_date, br.renew_count
        FROM borrows br
        LEFT JOIN readers r ON r.id = br.reader_id
        LEFT JOIN books b ON b.id = br.book_id
        WHERE br.returned = 0
          AND (?1 IS NULL OR br.reader_id = ?1)
        ORDER BY br.due_date, br.id DESC",
    )?;
    let rows = stmt.query_map(params![reader_id], |row| {
        Ok(BorrowView {
            id: row.get(0)?,
            reader_id: row.get(1)?,
            reader_name: row.get(2)?,
            book_id: row.get(3)?,
            title: row.get(4)?,
            borrow_date: row.get(5)?,
            due_date: row.get(6)?,
            renew_count: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub(crate) fn list_returns(
    path: &Path,
    reader_id: Option<&str>,
) -> rusqlite::Result<Vec<ReturnView>> {
    let conn = open_conn(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT ret.id, ret.reader_id, COALESCE(r.name, ''), ret.book_id, COALESCE(b.title, ''),
               ret.return_date, ret.due_date, ret.remark
        FROM returns ret
        LEFT JOIN readers r ON r.id = ret.reader_id
        LEFT JOIN books b ON b.id = ret.book_id
        WHERE (?1 IS NULL OR ret.reader_id = ?1)
        ORDER BY ret.return_date DESC, ret.id DESC",
    )?;
    let rows = stmt.query_map(params![reader_id], |row| {
        Ok(ReturnView {
            id: row.get(0)?,
            reader_id: row.get(1)?,
            reader_name: row.get(2)?,
            book_id: row.get(3)?,
            title: row.get(4)?,
            return_date: row.get(5)?,
            due_date: row.get(6)?,
            remark: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub(crate) fn list_exceptions(
    path: &Path,
    reader_id: Option<&str>,
) -> rusqlite::Result<Vec<ExceptionView>> {
    let conn = open_conn(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT ex.id, ex.exception_type, ex.amount, ex.status, ex.process_date,
               ex.reader_id, COALESCE(r.name, ''), ex.book_id, COALESCE(b.title, ''),
               ex.remark
        FROM exceptions ex
        LEFT JOIN readers r ON r.id = ex.reader_id
        LEFT JOIN books b ON b.id = ex.book_id
        WHERE (?1 IS NULL OR ex.reader_id = ?1)
        ORDER BY CASE WHEN ex.status = '已处理' THEN 1 ELSE 0 END, ex.id DESC",
    )?;
    let rows = stmt.query_map(params![reader_id], |row| {
        Ok(ExceptionView {
            id: row.get(0)?,
            exception_type: row.get(1)?,
            amount: row.get(2)?,
            status: row.get(3)?,
            process_date: row.get(4)?,
            reader_id: row.get(5)?,
            reader_name: row.get(6)?,
            book_id: row.get(7)?,
            title: row.get(8)?,
            remark: row.get(9)?,
        })
    })?;
    rows.collect()
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
