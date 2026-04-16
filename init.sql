CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_accounts (
    id SERIAL PRIMARY KEY,
    
    email TEXT NOT NULL,
    user_id INTEGER NOT NULL,

    access_token TEXT,
    refresh_token TEXT,
    token_expiry TIMESTAMP,

    is_active BOOLEAN DEFAULT TRUE,
    last_sync BIGINT,

    created_at TIMESTAMP DEFAULT NOW(),

    -- 🔐 Constraints
    CONSTRAINT fk_user_accounts
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE

);


CREATE TABLE IF NOT EXISTS meetings (
    id SERIAL PRIMARY KEY,
    user_id INTEGER,
    title TEXT NOT NULL,
    date DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    user_id INTEGER NOT NULL,

    CONSTRAINT fk_user_meetings
    FOREIGN KEY (user_id)
    REFERENCES users(id)
    ON DELETE CASCADE
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

DO $$ BEGIN
    CREATE TYPE message_status AS ENUM ('sent', 'delivered', 'read');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

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
    user_id INTEGER NOT NULL,

    name TEXT NOT NULL,
    file_type TEXT,
    file_path TEXT NOT NULL,
    size BIGINT DEFAULT 0,

    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    is_deleted BOOLEAN DEFAULT FALSE,

    -- Foreign key constraint
    CONSTRAINT fk_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
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

CREATE INDEX idx_files_user_id ON files(user_id);

CREATE INDEX idx_email_accounts_user_id
ON email_accounts(user_id);

CREATE UNIQUE INDEX unique_user_email_idx
ON email_accounts (user_id, LOWER(email));
