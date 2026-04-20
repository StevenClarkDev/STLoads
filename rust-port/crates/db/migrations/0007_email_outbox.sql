CREATE TABLE IF NOT EXISTS email_outbox (
    id BIGSERIAL PRIMARY KEY,
    template_name VARCHAR(96) NOT NULL,
    to_email VARCHAR(255) NOT NULL,
    to_name VARCHAR(255) NULL,
    subject VARCHAR(255) NOT NULL,
    html_body TEXT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 8,
    last_error TEXT NULL,
    next_attempt_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    locked_at TIMESTAMP NULL,
    sent_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_email_outbox_due
    ON email_outbox (status, next_attempt_at, id);

CREATE INDEX IF NOT EXISTS idx_email_outbox_recipient
    ON email_outbox (to_email, created_at);

