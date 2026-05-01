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



Make body_encrypted/body_iv nullable in init.sql + provide migration SQL

Refactor sync.rs to fetch headers only (format=metadata)

Create body_worker.rs that fills missing bodies in the background

Add GET /api/emails/{id}/body handler (on-demand body fetch + AES decrypt)

Update routes/email.rs list response (drop body, add has_body)

Wire body worker + new endpoint in main.rs

Update frontend Emails.tsx to fetch body on click

Run cargo check and tsc to verify everything compiles




File	Change
init.sql	Added idx_emails_pending_body partial index for body_encrypted = ''
backend/src/email/sync.rs	Replaced fetch_email_detail with fetch_headers_only using format=metadata. Inserts rows with body_encrypted='' sentinel.
backend/src/email/body_worker.rs	NEW — background worker, 40 concurrent fetches per account, 200/account/iteration, idle 5s when nothing pending
backend/src/email/handler.rs	Added GET /api/emails/{id}/body — auth + ownership → return cached AES-decrypted body, or fetch from Gmail on-demand and persist
backend/src/email/mod.rs	Exposed body_worker
backend/src/routes/email.rs	Dropped body_encrypted/body_iv from list, added has_body and gmail_id
backend/src/main.rs	Registered get_email_body + start_body_worker(pool)
frontend/src/emails/Emails.tsx	openEmail now fetches /api/emails/:id/body, shows "Loading…" state, then runs WAYVE_SECURE_V1 RSA decrypt as before
What you need to do before this works
Wipe existing email bodies (you said wipe and re-sync):


UPDATE emails SET body_encrypted = '', body_iv = '';
UPDATE email_accounts SET last_sync = NULL;
The first line marks every existing email as "needs body fetched" so the body worker picks them up. The second forces the header sync to re-walk all message IDs (cheap; metadata-only).

Apply the new partial index — running init.sql is idempotent thanks to IF NOT EXISTS, but psql -f init.sql may complain about earlier non-IF-NOT-EXISTS lines. Just run the new index manually:


CREATE INDEX IF NOT EXISTS idx_emails_pending_body
ON emails (account_id, id) WHERE body_encrypted = '';
Restart the backend so the body worker spawns and the new route registers.

After that, click any email — you'll get either an instant decrypt (if the worker already filled it) or a brief "Loading…" while the on-demand fetch runs. Background worker chews through the rest at ~40 concurrent fetches per account.