# Multibuffer Walkthrough Sample Fixture

This multi-file Rust workspace serves as the reproducible laboratory for Zed's
multibuffer capabilities under `lessons/zed-03-multibuffer.toml` and
`docs/walkthroughs/multibuffer.md`.

## Project Structure

- `src/main.rs`: Entry point initializing `SessionStore` and routing requests.
- `src/session.rs`: Core types (`SessionToken`, `SessionRecord`, `SessionStatus`).
- `src/handlers.rs`: HTTP request handlers validating tokens and extracting claims.
- `src/store.rs`: Memory store maintaining session lookups and token validation.
- `tests/session_test.rs`: Integration tests for session lifecycle and verification.
