# Worklog

## 2026-06-12 refactor-manusskilled

### Goal

- Continue the Rust-only library management system under `projects/libadmin`.
- Use `projects/Rust 项目架构设计指南.md` as the refactor reference.
- Keep the application behavior stable while improving Rust project structure, module boundaries, and test coverage.

### Context

- Confirmed the work branch as `refactor-manusskilled`.
- Used the WSL Rust toolchain for formatting, checking, and testing.
- Kept unrelated worktree changes out of the refactor scope.

### Completed Work

- Applied binary/library separation:
  - reduced `src/main.rs` to a thin Tokio entrypoint;
  - moved application startup and router construction into `src/app.rs`;
  - exposed `libadmin::run()` from `src/lib.rs`.
- Split the previous monolithic Rust implementation into focused modules:
  - `src/models.rs` for session and domain/view structs;
  - `src/forms.rs` for Axum request/query DTOs;
  - `src/db/mod.rs` for SQLite setup, seed data, backups, and query helpers;
  - `src/services/mod.rs` for borrow, return, renew, exception, and delete-guard business logic;
  - `src/utils.rs` for password hashing, date parsing, ID validation, and database error formatting;
  - `src/web/views.rs` for escaping, layout, table rendering, status labels, metrics, flash messages, and CSS;
  - `src/web/handlers/*` for auth, dashboard, reader, admin, and shared HTTP handler concerns.
- Tightened crate visibility with `pub(crate)` module APIs instead of exposing internals publicly.
- Added regression tests:
  - ID validation, password hashing, and date parsing tests in `utils`;
  - SQLite-backed service tests for borrow, renew, return, and active-borrow delete protection.
- Created commit `49b9060 Refactor libadmin Rust architecture`.

### Verification

Executed in WSL from `projects/libadmin`:

```bash
cargo fmt -- --check
cargo check
cargo test
```

Results:

- `cargo fmt -- --check` passed.
- `cargo check` passed.
- `cargo test` passed with 6 tests.

### Notes

- The refactor intentionally avoided Python and kept the system implemented in Rust.
- Existing unrelated untracked files such as `projects/trial2/target/` were not staged or committed.
- Follow-up work can introduce a unified error enum with `thiserror` if the project needs stronger typed errors across the database, service, and web layers.

## 2026-06-12 refactor-gptskilled

### Goal

- Continue the Rust refactor on the `refactor-gptskilled` branch.
- Follow the Rust project workflow documented in `projects/g-rust-skill.md`.
- Keep application behavior unchanged while reducing the size and responsibility of `src/main.rs`.

### Completed Work

- Confirmed the active branch is `refactor-gptskilled`.
- Split the original monolithic `src/main.rs` into focused modules:
  - `src/auth.rs`: session lookup, authorization checks, redirects, and forbidden responses.
  - `src/db.rs`: SQLite setup, seed data, backups, queries, borrow/return transactions, renewals, exception resolution, and delete guards.
  - `src/forms.rs`: Axum form and query DTOs.
  - `src/models.rs`: app state, sessions, domain models, and view rows.
  - `src/routes.rs`: router registration, page handlers, and form submission handlers.
  - `src/util.rs`: password hashing, dates, ID validation, database error formatting, and filter helpers.
  - `src/views.rs`: HTML escaping, layout, tables, status labels, metrics, flash messages, and CSS.
- Reduced `src/main.rs` to application startup, database initialization, daily backup, router mounting, and listener binding.
- Fixed the Clippy baseline issues encountered during the refactor:
  - boxed the large authorization error response type;
  - collapsed nested `if` branches reported by Clippy.
- Created commit `eccc94e Refactor libadmin Rust modules`.

### Verification

Executed in WSL with the Rust toolchain:

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build
```

All commands passed. `cargo test` currently reports 0 tests.

### Follow-Up

- Add integration tests for `db.rs` borrow, return, renew, exception, and delete guard paths.
- Continue splitting `routes.rs` by public, reader, and admin concerns.
- Fix the existing mojibake text in UI/database seed strings before adding more user-facing copy.
