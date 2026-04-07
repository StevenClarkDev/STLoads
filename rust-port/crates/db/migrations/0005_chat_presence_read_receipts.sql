-- Conversation read receipts and presence tracking for the Rust chat cutover.
-- These tables stay additive so we can diff them cleanly against the legacy Laravel schema.

CREATE TABLE IF NOT EXISTS conversation_reads (
    conversation_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    last_read_message_id BIGINT NULL,
    last_read_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (conversation_id, user_id),
    CONSTRAINT fk_conversation_reads_conversation_id FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    CONSTRAINT fk_conversation_reads_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_conversation_reads_last_read_message_id FOREIGN KEY (last_read_message_id) REFERENCES messages(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_conversation_reads_user_id ON conversation_reads (user_id);
CREATE INDEX IF NOT EXISTS idx_conversation_reads_last_read_message_id ON conversation_reads (last_read_message_id);

CREATE TABLE IF NOT EXISTS conversation_presence (
    conversation_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    state VARCHAR(32) NOT NULL DEFAULT 'online',
    last_seen_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (conversation_id, user_id),
    CONSTRAINT fk_conversation_presence_conversation_id FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    CONSTRAINT fk_conversation_presence_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_conversation_presence_user_id ON conversation_presence (user_id);
CREATE INDEX IF NOT EXISTS idx_conversation_presence_last_seen_at ON conversation_presence (last_seen_at);
