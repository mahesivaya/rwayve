CREATE TABLE IF NOT EXISTS email_accounts (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    access_token TEXT,
    refresh_token TEXT,
    token_expiry TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE,
    last_sync BIGINT
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
    account_id INTEGER,
    subject TEXT,
    sender TEXT,
    receiver TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    body_encrypted TEXT NOT NULL,
    body_iv TEXT NOT NULL,
    UNIQUE(account_id, gmail_id)
);


CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS messages (
    id SERIAL PRIMARY KEY,
    sender_id INT,
    receiver_id INT,
    content_encrypted TEXT, 
    content_iv TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

ALTER TABLE emails ADD COLUMN body_encrypted TEXT;
ALTER TABLE emails ADD COLUMN body_iv TEXT;