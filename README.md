# rustlearn

A repo that records my Rust language learning, CS110L practice, and small
application projects. It is both a study notebook and a capacity proof: notes,
experiments, working code, reviews, refactors, and verification history stay in
one place.

## Learning Path

- 2026-06-11: set up `rustc` and `cargo` in WSL, then started with small syntax
  experiments under `language/`.
- 2026-06-11: moved from basic grammar into ownership, memory, Cargo, and
  CS110L reading/practice.
- 2026-06-12: switched to project-driven learning and built the first local
  library-management demo from the database course report.
- 2026-06-12 to 2026-06-13: refactored the demo into a structured Rust web
  application with tests, CI, and documented review follow-up work.

## Repository Map

- `language/`: focused Rust language experiments and notes, including grammar,
  functions, enums, closures, memory, and ownership.
- `CS110L/assignment/`: CS110L assignment work and progress notes.
- `projects/trial1/` and `projects/trial2/`: early Cargo project experiments.
- `projects/libadmin/`: Rust + Axum + SQLite library management system.
- `review/`: review notes, definition-of-done material, and follow-up guidance
  used to drive the `libadmin` refactor.

## Main Project: libadmin

`projects/libadmin` is the main applied project in this repo. It started as a
single-file Rust implementation and was refactored into a maintainable web
application:

- thin binary entrypoint in `src/main.rs`;
- application startup and router construction in `src/app.rs`;
- domain/view models in `src/models.rs`;
- request/query forms in `src/forms.rs`;
- SQLite setup, seed data, backups, and query helpers in `src/db/`;
- business logic in `src/services/`;
- HTTP handlers split by auth, dashboard, reader, admin, and shared concerns in
  `src/web/handlers/`;
- HTML rendering helpers and styles in `src/web/views.rs`;
- utility code for password hashing, validation, dates, and database errors in
  `src/utils.rs`;
- typed service errors in `src/errors.rs`.

Implemented behavior includes reader/admin login, cookie sessions, reader
profile maintenance, book search, borrow/return/renew flows, admin CRUD for
readers/books/admins, exception reporting and compensation handling, SQLite seed
data, and backup support.

## Quality Gate

The libadmin worklog records the current quality gate as:

```bash
cd projects/libadmin
cargo fmt -- --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Recent verification passed with 16 tests total: 11 unit/service tests and 5 HTTP
integration tests. CI for the same Rust checks lives in
`.github/workflows/libadmin-rust.yml`.

## Notes

- Rust build output such as `target/` is ignored and should not be committed.
- `timeline.txt` is a local personal timeline file and is no longer tracked.
- Project-level implementation details and run commands live in
  `projects/libadmin/README.md`.
- Refactor history and rationale live in `projects/libadmin/WORKLOG.md`.
