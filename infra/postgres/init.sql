CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    public_key TEXT,
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


CREATE TABLE IF NOT EXISTS emails (
    id SERIAL PRIMARY KEY,
    gmail_id TEXT NOT NULL,
    account_id INTEGER REFERENCES email_accounts(id) ON DELETE CASCADE,
    subject TEXT,
    sender TEXT,
    receiver TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    body_encrypted TEXT,
    body_iv TEXT,
    body_cached TEXT,
    body_cached_at TIMESTAMP,
    UNIQUE(account_id, gmail_id)
);




-- 1. Remove old wrong constraint (if exists)
ALTER TABLE email_accounts
ADD CONSTRAINT unique_user_email UNIQUE (user_id, email);



CREATE TABLE IF NOT EXISTS meetings (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    date DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    zoom_join_url TEXT,

    CONSTRAINT fk_user_meetings
    FOREIGN KEY (user_id)
    REFERENCES users(id)
    ON DELETE CASCADE
);

ALTER TABLE meetings ADD COLUMN IF NOT EXISTS zoom_join_url TEXT;

CREATE TABLE meeting_participants (
    id SERIAL PRIMARY KEY,
    meeting_id INT REFERENCES meetings(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    user_id INT NULL,   -- if exists in your system
    status TEXT DEFAULT 'pending'
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

CREATE INDEX idx_emails_pagination 
ON emails (account_id, created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_emails_account_created
ON emails (account_id, created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_emails_pending_body
ON emails (account_id, id) WHERE body_encrypted = '';

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

CREATE INDEX idx_meeting_participants_meeting_id
ON meeting_participants(meeting_id);

CREATE INDEX idx_accounts_user
ON email_accounts (user_id);

CREATE INDEX idx_emails_account 
ON emails (account_id);

CREATE UNIQUE INDEX idx_emails_unique 
ON emails (account_id, gmail_id);

CREATE INDEX idx_messages_reverse 
ON messages (receiver_id, sender_id, created_at DESC);

CREATE INDEX idx_messages_unread 
ON messages (receiver_id, status);

CREATE INDEX idx_files_user 
ON files (user_id, created_at DESC);