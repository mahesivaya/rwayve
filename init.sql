CREATE TABLE IF NOT EXISTS email_accounts (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    access_token TEXT,
    refresh_token TEXT,
    token_expiry TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE,
    last_sync BIGINT
);

CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS meetings (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    date DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS emails (
    id SERIAL PRIMARY KEY,
    gmail_id TEXT NOT NULL,
    account_id INTEGER REFERENCES email_accounts(id) ON DELETE CASCADE,
    subject TEXT,
    sender TEXT,
    receiver TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    body_encrypted TEXT NOT NULL,
    body_iv TEXT NOT NULL,
    UNIQUE(account_id, gmail_id)
);

CREATE TYPE message_status AS ENUM ('sent', 'delivered', 'read');

CREATE TABLE IF NOT EXISTS messages (
    id SERIAL PRIMARY KEY,
    sender_id INT REFERENCES users(id) ON DELETE CASCADE,
    receiver_id INT REFERENCES users(id) ON DELETE CASCADE,
    content_encrypted TEXT,
    content_iv TEXT,
    status message_status DEFAULT 'sent',
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS files (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    size BIGINT DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW()
);




-- 🔥 INDEXES

CREATE INDEX idx_messages_conversation 
ON messages (sender_id, receiver_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_emails_account_created
ON emails (account_id, created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_messages_users
ON messages (sender_id, receiver_id);

CREATE INDEX IF NOT EXISTS idx_messages_id
ON messages (id ASC);

CREATE INDEX IF NOT EXISTS idx_meetings_date
ON meetings (date);

-- Emails pagination
CREATE INDEX IF NOT EXISTS idx_emails_account_created
ON emails (account_id, created_at DESC, id DESC);

-- Chat queries
CREATE INDEX IF NOT EXISTS idx_messages_users
ON messages (sender_id, receiver_id);

-- Meetings
CREATE INDEX IF NOT EXISTS idx_meetings_date
ON meetings (date);