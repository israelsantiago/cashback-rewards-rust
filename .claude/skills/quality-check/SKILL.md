# Quality check

Run the project quality gate:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

For PostgreSQL integration coverage, set `DATABASE_URL` to a disposable test database.
