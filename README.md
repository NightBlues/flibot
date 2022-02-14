Flibot is a simple telegram bot that helps searching and downloading books from flibusta library through tor.


# Build
- `cargo install sqlx-cli`
- `sqlx migrate run`
- `export DATABASE_URL=sqlite://db.sqlite`
- `cargo sqlx prepare`
- `SQLX_OFFLINE=true cargo build`

# Run
- `export DATABASE_URL=sqlite://db.sqlite`
- `export TELOXIDE_TOKEN=...`
- `RUST_LOG=debug cargo run`

