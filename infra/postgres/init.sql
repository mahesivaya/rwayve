CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    public_key TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Profile fields. Idempotent: safe to re-run on an existing DB.
ALTER TABLE users ADD COLUMN IF NOT EXISTS first_name TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_name TEXT;

-- Google signup support: password is NULL for users who registered via OAuth.
ALTER TABLE users ALTER COLUMN password DROP NOT NULL;
ALTER TABLE users ADD COLUMN IF NOT EXISTS auth_provider TEXT NOT NULL DEFAULT 'local';

-- Password reset tokens. Single-use, 30-minute lifetime.
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_id
    ON password_reset_tokens(user_id);



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
ALTER TABLE meetings ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'wayve';
ALTER TABLE meetings ADD COLUMN IF NOT EXISTS google_event_id TEXT;
ALTER TABLE meetings ADD COLUMN IF NOT EXISTS account_id INTEGER;
CREATE UNIQUE INDEX IF NOT EXISTS meetings_google_event_uniq
  ON meetings(user_id, google_event_id) WHERE google_event_id IS NOT NULL;

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

-- Notes
CREATE TABLE IF NOT EXISTS notes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    title TEXT,
    content TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);


-- 🔥 INDEXES

CREATE INDEX IF NOT EXISTS idx_messages_conversation
ON messages (sender_id, receiver_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_messages_reverse
ON messages (receiver_id, sender_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_messages_unread
ON messages (receiver_id, status);

CREATE INDEX IF NOT EXISTS idx_emails_account_created
ON emails (account_id, created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_emails_pending_body
ON emails (account_id, id) WHERE body_encrypted = '';

CREATE INDEX IF NOT EXISTS idx_meetings_user_date
ON meetings (user_id, date, start_time);

CREATE INDEX IF NOT EXISTS idx_meeting_participants_meeting_id
ON meeting_participants(meeting_id);

CREATE INDEX IF NOT EXISTS idx_email_accounts_user_id
ON email_accounts(user_id);

CREATE UNIQUE INDEX IF NOT EXISTS unique_user_email_idx
ON email_accounts (user_id, LOWER(email));

CREATE INDEX IF NOT EXISTS idx_files_user
ON files (user_id, created_at DESC);