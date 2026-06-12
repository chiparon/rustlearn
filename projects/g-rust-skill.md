---
name: rust-project-build
description: Build, test, lint, format, and diagnose Rust projects using Cargo. Use this skill when working on Rust crates, Cargo workspaces, CLI tools, libraries, or backend services.
---

# Rust Project Build Skill

## Scope

Use this skill when the task involves a Rust project, including:

- creating a new Rust project
- fixing Cargo build errors
- configuring dependencies
- running tests
- running format and lint checks
- preparing a project for CI
- diagnosing workspace, feature, or target-related problems

Prefer Cargo-based workflows unless the repository explicitly uses another build system.

## Initial inspection

Before modifying code, inspect the project structure:

1. Check whether `Cargo.toml` exists.
2. Determine whether the repository is:
   - a single binary crate
   - a single library crate
   - a Cargo workspace
   - a mixed project containing Rust plus other languages
3. Read:
   - `Cargo.toml`
   - `Cargo.lock` if present
   - `rust-toolchain.toml` or `rust-toolchain` if present
   - `README.md` if it contains build instructions
   - `.cargo/config.toml` if present
   - CI files such as `.github/workflows/*.yml`

Do not assume a package name, binary name, feature name, or target triple without checking project files.

## Standard command order

Use this order unless the project documentation says otherwise:

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build
```
For a workspace, prefer:
```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace --all-targets --all-features
```
If --all-features fails because features are mutually exclusive, inspect Cargo.toml and choose the documented feature set.

Error diagnosis rules

When a command fails:

Identify the first real compiler or Cargo error.
Ignore repeated downstream errors until the root cause is fixed.
Classify the failure as one of:
syntax error
missing dependency
wrong crate feature
visibility/module path error
ownership or lifetime error
trait bound error
async/runtime mismatch
platform-specific dependency problem
test expectation failure
formatting or lint issue
Propose the smallest code change that fixes the root cause.
Re-run the narrowest relevant command after each fix.
After the narrow command passes, run the full standard command sequence.
Dependency management

When adding dependencies:

Prefer cargo add <crate> if cargo-edit is available.
Otherwise edit Cargo.toml directly.
Use existing version style in the repository.
Do not upgrade unrelated dependencies unless the task requires it.
If a dependency needs features, add only the required features.

Examples:
```bash
cargo add anyhow
cargo add tokio --features full
cargo add serde --features derive
```
For libraries, avoid unnecessary heavy dependencies.

Project creation

For a binary CLI project:
```bash
cargo new project_name
cd project_name
cargo run
```
For a library:
```bash
cargo new project_name --lib
cd project_name
cargo test
```
For a workspace:
```TOML
[workspace]
members = [
    "crates/*"
]
resolver = "2"
```
Use resolver = "2" for modern Cargo workspaces unless the repository has compatibility constraints.

Code quality policy

Prefer:

explicit error handling with Result
thiserror for library error types when appropriate
anyhow for application-level error aggregation when appropriate
small modules with clear public interfaces
unit tests for pure logic
integration tests for CLI or public API behavior
no unnecessary unsafe
no broad unwrap() or expect() in production code unless justified

Allow unwrap() and expect() in tests when they simplify test setup.

Testing policy

When adding functionality:

Add or update tests.
Prefer deterministic tests.
Avoid network, filesystem, or time dependency unless required.
For CLI projects, test command behavior if the project already uses CLI testing tools.
For libraries, test public API behavior rather than private implementation details.

Run:
```bash
cargo test
```
For workspaces:
```bash
cargo test --workspace --all-targets
```
Formatting and linting

Use Rustfmt and Clippy.
```bash
cargo fmt
cargo clippy --all-targets --all-features
```
If Clippy reports a warning:

Prefer changing the code.
Use #[allow(...)] only when there is a local, explicit reason.
Do not silence lint groups globally unless the repository already uses that policy.
Release build

For performance or release verification:
```bash
cargo build --release
```
If binary size or runtime performance matters, inspect Cargo.toml profiles before changing them.

CI recommendation

For GitHub Actions, recommend this minimal workflow:
```YAML
name: Rust CI

on:
  push:
  pull_request:

jobs:
  rust:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Format
        run: cargo fmt --all -- --check

      - name: Check
        run: cargo check --workspace --all-targets --all-features

      - name: Test
        run: cargo test --workspace --all-targets --all-features

      - name: Clippy
        run: cargo clippy --workspace --all-targets --all-features -- -D warnings
```
If the project is not a workspace, remove --workspace.

Final response format

When reporting back:

State what was inspected.
State which commands were run.
State which errors were found.
State what was changed.
State the final verification result.
If something could not be verified, say exactly why.

Do not claim the project builds unless the relevant Cargo command passed.