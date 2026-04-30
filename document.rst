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


cargo watch -x run


➜  frontend git:(main) ✗ pkill -f vite
➜  frontend git:(main) ✗ lsof -i :5173

Front-end URL: Cloudfront: https://d2j48xaszdfk51.cloudfront.net/login

Back-end: ECS: http://52.23.197.236:8080/




13656* cargo clippy
13657* cd backend
13658* cargo clippy
13659* cargo fmt --all\ncargo clippy --all -- -D warnings
13660* cargo fmt
13661* cargo lint




.. Always do this before commit
cargo fmt
cargo clippy -- -D warnings
cargo check

cargo install cargo-modules
cargo modules structure
cargo modules structure --no-fns
cargo modules structure --no-fns --no-types --no-traits
cargo modules structure --focus-on email
cargo modules structure --focus-on chat
cargo modules structure --max-depth 2
cargo modules structure --no-fns --max-depth 3


Wayve/
└── 📂 backend/
    ├── 📂 src/
    │   ├── 📂 call/
    │   │   └── 📂 handlers/
    │   │       └── 📄 fn call_ws          # WebSocket signaling for calls
    │   ├── 📂 chat/
    │   │   └── 📂 handlers/
    │   │       ├── 📄 fn chat_ws          # Real-time chat messaging
    │   │       ├── 📄 fn get_messages     # History retrieval
    │   │       └── 📄 ChatSession         # Session state management
    │   ├── 📂 drive/
    │   │   └── 📂 handlers/
    │   │       ├── 📄 fn upload_file
    │   │       └── 📄 fn get_files
    │   ├── 📂 email/
    │   │   ├── 📁 auth/
    │   │   │   └── 📄 refresh_access_token
    │   │   ├── 📁 handlers/
    │   │   │   ├── 📄 gmail_login         # OAuth Initiation
    │   │   │   ├── 📄 oauth_callback      # Token exchange
    │   │   │   ├── 📄 send                # Outbound mail
    │   │   │   ├── 📄 get_me              # Profile info
    │   │   │   └── 📄 save_public_key     # End-to-end encryption setup
    │   │   ├── 📁 sync/
    │   │   │   ├── 📄 sync_all            # Full mailbox synchronization
    │   │   │   ├── 📄 process_batch       # Background processing logic
    │   │   │   └── 📄 fetch_ids/details   # IMAP/API fetching logic
    │   │   └── 📁 utils/
    │   │       ├── 📄 extract_body
    │   │       └── 📄 decode_base64       # MIME handling
    │   ├── 📂 routes/                     # API Endpoint definitions
    │   │   ├── 📄 account / auth
    │   │   └── 📄 email / user
    │   ├── 📂 security/
    │   │   ├── 📄 encryption              # Likely PGP or AES logic
    │   │   └── 📄 jwt                     # Session token management
    │   ├── 📂 scheduler/
    │   │   └── 📄 handler                 # Cron/Task scheduling
    │   ├── 📄 main.rs                     # Entry point & Server setup
    │   └── 📄 cargo.toml                  # Dependencies
    └── ...


├── 📂 frontend/
│   ├── 📂 src/
│   │   ├── 📂 api/
│   │   ├── 📂 assets/
│   │   ├── 📂 auth/
│   │   ├── 📂 call/
│   │   ├── 📂 chat/
│   │   ├── 📂 components/
│   │   ├── 📂 crypto/
│   │   ├── 📂 drive/
│   │   ├── 📂 emails/
│   │   ├── 📂 home/
│   │   ├── 📂 pages/
│   │   ├── 📂 scheduler/
│   │   ├── 📂 security/
│   │   ├── 📄 api.ts
│   │   ├── 📄 App.tsx
│   │   └── 📄 config.ts
│   └── ...
├── 📂 nginx/
│   └── 📄 nginx.conf
├── 📄 docker-compose.yml
└── 📄 init.sql


modules/
  email/
    api/
    service/
    repo/
    integration/

  chat/
    websocket/
    service/
    repo/

  drive/
  scheduler/


email/
├── api/
│   └── email_api.rs
├── services/
│   ├── email_service.rs
│   └── email_sync_service.rs
├── repositories/
│   └── email_repo.rs
├── integrations/
│   └── gmail_client.rs