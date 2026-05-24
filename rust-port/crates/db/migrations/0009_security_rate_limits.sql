-- Shared security throttles for rate limits and account lockouts.

CREATE TABLE IF NOT EXISTS security_rate_limits (
    key VARCHAR(255) PRIMARY KEY,
    counter INT NOT NULL DEFAULT 0,
    window_started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    locked_until TIMESTAMP NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_security_rate_limits_expires_at
    ON security_rate_limits (expires_at);

CREATE INDEX IF NOT EXISTS idx_security_rate_limits_locked_until
    ON security_rate_limits (locked_until)
    WHERE locked_until IS NOT NULL;
