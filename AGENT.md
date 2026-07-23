# AGENT.md — Crateria hardening contract

Enforce on every change in this repository.

## Rust and license

- Strict Rust from first principles. Prefer `std` before crates.
- License: **Apache-2.0** only for this project and new code.
- Vetted dependencies only; no `unsafe` without a short safety comment.

## Constraints

- Max **250 lines** per `.rs` file (split at functional boundaries).
- **Zero** `.unwrap()` or `.expect()` in production (non-test) code.
  Tests may use `unwrap`/`expect` only when the failure message is clear.
- Fallible APIs use `Result` / `Option` with explicit error types.

## Testing

- Target about **3:1** test lines to production lines where practical.
- Protocol, parsing, path safety, and package index logic must have
  **proptest** (or equivalent property) tests.
- Prefer deterministic unit tests; integration tests for CLI boundaries.

## Git edges

- Default branch: **`master`**.
- After each hardening stage barrier: declarative commit message and push to `origin master`.
