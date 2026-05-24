-- Hash Rust session tokens so database values cannot be replayed as bearer tokens.

ALTER TABLE personal_access_tokens
    ADD COLUMN IF NOT EXISTS token_prefix VARCHAR(32) NULL,
    ADD COLUMN IF NOT EXISTS token_hash VARCHAR(64) NULL;

-- Existing rows stored bearer tokens directly. Invalidate them explicitly so old
-- database token values cannot continue to authenticate after this migration.
UPDATE personal_access_tokens
SET token = CONCAT('invalidated:', id),
    token_prefix = NULL,
    token_hash = NULL,
    updated_at = CURRENT_TIMESTAMP
WHERE token_hash IS NULL
  AND token NOT LIKE 'invalidated:%';

CREATE UNIQUE INDEX IF NOT EXISTS uq_personal_access_tokens_token_hash
    ON personal_access_tokens (token_hash)
    WHERE token_hash IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_personal_access_tokens_token_prefix
    ON personal_access_tokens (token_prefix)
    WHERE token_prefix IS NOT NULL;
