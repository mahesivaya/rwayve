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


CREATE TABLE email_accounts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    email TEXT NOT NULL,
    provider TEXT DEFAULT 'gmail',

    access_token TEXT,
    refresh_token TEXT,
    token_expiry TIMESTAMP,

    last_sync TIMESTAMP,

    created_at TIMESTAMP DEFAULT NOW(),

    UNIQUE(user_id, email)
);

CREATE TABLE meetings (
  id SERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  date DATE NOT NULL,
  start_time TIME NOT NULL,
  end_time TIME NOT NULL,
  created_at TIMESTAMP DEFAULT NOW()
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


CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE messages (
    id SERIAL PRIMARY KEY,
    sender_id INT,
    receiver_id INT,
    content TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);


CREATE TABLE meetings (
  id SERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  date DATE NOT NULL,
  start_time TIME NOT NULL,
  end_time TIME NOT NULL,
  created_at TIMESTAMP DEFAULT NOW()
);

http://localhost:8080/gmail/login
http://localhost:8080/emails


http://localhost:8080/gmail/login


.. Keeps the data in tables 
docker-compose down
docker-compose up --build


⚡ BONUS: Use cargo watch (🔥 game changer)

Install:

cargo install cargo-watch

Run:

cargo watch -x run

👉 auto rebuild on file change
👉 no docker rebuild needed


src/
├── main.rs
├── prelude.rs

├── config/          # env, constants
├── models/          # DB structs
├── dto/             # request/response shapes (NEW)

├── handlers/        # HTTP layer ONLY
├── services/        # business logic (IMPORTANT)
├── repositories/    # DB queries (NEW, CLEAN)

├── auth/            # oauth + jwt
├── utils/



cargo build ---> cargo check

install:
cargo install cargo-watch

for stopped app: cargo watch -x check
for running app: cargo watch -x run



➜  frontend git:(main) ✗ pkill -f vite
➜  frontend git:(main) ✗ lsof -i :5173

Front-end URL: Cloudfront: https://d2j48xaszdfk51.cloudfront.net/login

Back-end: ECS: http://52.23.197.236:8080/




