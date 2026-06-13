#[derive(Clone)]
pub(crate) struct Session {
    pub(crate) role: String,
    pub(crate) user_id: String,
    pub(crate) display_name: String,
}

#[derive(Clone)]
pub(crate) struct Reader {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) gender: String,
    pub(crate) profession: String,
    pub(crate) max_borrow: i64,
    pub(crate) borrow_days: i64,
    pub(crate) remark: String,
}

#[derive(Clone)]
pub(crate) struct Book {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) category: String,
    pub(crate) keywords: String,
    pub(crate) status: String,
    pub(crate) remark: String,
}

#[derive(Clone)]
pub(crate) struct Admin {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) level: i64,
    pub(crate) remark: String,
}

pub(crate) struct BorrowView {
    pub(crate) id: i64,
    pub(crate) reader_id: String,
    pub(crate) reader_name: String,
    pub(crate) book_id: String,
    pub(crate) title: String,
    pub(crate) borrow_date: String,
    pub(crate) due_date: String,
    pub(crate) renew_count: i64,
}

pub(crate) struct ReturnView {
    pub(crate) id: i64,
    pub(crate) reader_id: String,
    pub(crate) reader_name: String,
    pub(crate) book_id: String,
    pub(crate) title: String,
    pub(crate) return_date: String,
    pub(crate) due_date: String,
    pub(crate) remark: String,
}

pub(crate) struct ExceptionView {
    pub(crate) id: i64,
    pub(crate) exception_type: String,
    pub(crate) amount: f64,
    pub(crate) status: String,
    pub(crate) process_date: String,
    pub(crate) reader_id: String,
    pub(crate) reader_name: String,
    pub(crate) book_id: String,
    pub(crate) title: String,
    pub(crate) remark: String,
}
