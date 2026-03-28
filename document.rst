Cargo offers a suite of commands for project management. 
Rust Documentation
Rust Documentation
cargo new <project_name>: Creates a new Cargo package in a new directory.
cargo init: Creates a new Cargo package in the current directory.
cargo build: Compiles the project without running it, placing the executable in target/debug.
cargo check: Checks the project for errors without producing an executable, which is faster than building.
cargo test: Executes unit and integration tests.
cargo doc: Builds the project's documentation.

cargo build --release


CREATE TABLE email_accounts (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    access_token TEXT,
    refresh_token TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    last_sync BIGINT
);

CREATE TABLE emails (
    id SERIAL PRIMARY KEY,
    gmail_id TEXT NOT NULL,
    account_id INTEGER,
    subject TEXT,
    sender TEXT,
    receiver TEXT,
    body TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(account_id, gmail_id)
);


http://localhost:8080/gmail/login
http://localhost:8080/emails


http://localhost:8080/gmail/login