# Worklog

## 2026-06-13 typed-errors

### Goal

- Follow `review/definition-of-done.md` and close the highest-priority B-line gap.
- Replace service-layer `Result<(), String>` errors with typed Rust errors using `thiserror`.
- Preserve existing successful workflows while allowing handlers to map service failures to HTTP status codes.

### Completed Work

- Added `thiserror` and a new `errors` module.
- Introduced `LibError` variants:
  - `InvalidInput`
  - `NotFound`
  - `RuleViolation`
  - `Db`
- Replaced service-layer string errors with `LibResult<T>`.
- Kept database errors as `LibError::Db(#[from] rusqlite::Error)` instead of formatting them immediately into strings.
- Mapped service errors in HTTP handlers:
  - `InvalidInput` / `RuleViolation` -> `400 Bad Request`
  - `NotFound` -> `404 Not Found`
  - `Db` -> `500 Internal Server Error`
- Updated service tests to assert error variants, and added coverage for `InvalidInput` and `NotFound`.
- Added an HTTP integration test that verifies a missing book borrow request returns `404`.
- Updated README with transaction/concurrency notes and explicit security limitations for password hashing and in-memory sessions.

### Verification

Executed in WSL from `projects/libadmin`:

```bash
cargo fmt -- --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Results:

- Format check passed.
- Compile check passed.
- `cargo test` passed with 16 tests total: 11 unit/service tests and 5 HTTP integration tests.
- Clippy passed with warnings denied.

## 2026-06-13 integration-tests-ci

### Goal

- Follow `review/claudereview-perspective.md` and prioritize HTTP-layer integration tests.
- Make the Axum router constructible from tests without opening a network listener.
- Add CI so the local Rust quality gate is reproducible on push and pull requests.

### Completed Work

- Extracted `build_router(db_path)` from `run()` and exported it from the library API.
- Kept `run()` focused on data-directory setup, daily backup, listener binding, and serving.
- Added `tests/http_auth.rs` integration tests using `tower::ServiceExt::oneshot`.
- Covered real HTTP flows:
  - unauthenticated `/admin` redirects to login;
  - reader sessions receive `403 Forbidden` on admin pages;
  - admin login reaches the dashboard;
  - reader borrow, renew, and return work through form routes and update SQLite state.
- Added a `tower` dev-dependency with the `util` feature for router testing.
- Added `.github/workflows/libadmin-rust.yml` for `fmt`, `check`, `test`, and `clippy -D warnings` on `projects/libadmin` changes.

### Verification

Executed in WSL from `projects/libadmin`:

```bash
cargo fmt -- --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Results:

- Format check passed.
- Compile check passed.
- `cargo test` passed with 14 tests total: 10 unit/service tests and 4 HTTP integration tests.
- Clippy passed with warnings denied.

## 2026-06-13 refactor-manusskilled

### Goal

- Continue from `review/claudereview.md` using `refactor-manusskilled` as the baseline.
- Start with the highest-priority direction: move remaining admin CRUD SQL out of HTTP handlers.
- Preserve behavior while improving service-layer consistency and test coverage.

### Completed Work

- Moved admin reader/book/admin/exception write operations from `web/handlers/admin.rs` into `services/mod.rs`.
- Added service input structs:
  - `ReaderInput`
  - `BookInput`
  - `AdminInput`
  - `ExceptionInput`
- Added service functions for admin CRUD paths:
  - `create_reader` / `update_reader`
  - `create_book` / `update_book`
  - `create_admin` / `update_admin` / `delete_admin`
  - `create_exception`
- Kept `admin.rs` focused on authorization, form mapping, redirects, and rendering.
- Fixed Clippy baseline issues encountered while running the quality gate:
  - boxed auth error responses in `shared.rs`;
  - collapsed nested `if` checks in services and logout handling.
- Expanded service-layer tests from 6 to 10 total tests.

### Verification

Executed in WSL from `projects/libadmin`:

```bash
cargo fmt
cargo fmt -- --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Results:

- Format check passed.
- Compile check passed.
- `cargo test` passed with 10 tests.
- Clippy passed with warnings denied.

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
