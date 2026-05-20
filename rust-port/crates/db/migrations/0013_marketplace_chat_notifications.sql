-- P11 marketplace chat, notification preference, and message context hardening.

CREATE TABLE IF NOT EXISTS marketplace_notification_preferences (
    tenant_id VARCHAR(100) NOT NULL,
    user_id BIGINT NOT NULL,
    email_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    critical_email_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    realtime_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (tenant_id, user_id),
    CONSTRAINT fk_marketplace_notification_preferences_tenant_id FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    CONSTRAINT fk_marketplace_notification_preferences_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_marketplace_notification_preferences_user
    ON marketplace_notification_preferences (user_id, tenant_id);

CREATE INDEX IF NOT EXISTS idx_messages_meta_event_type
    ON messages ((meta->>'event_type'))
    WHERE meta IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_messages_meta_reference
    ON messages ((meta->>'reference_type'), (meta->>'reference_id'))
    WHERE meta IS NOT NULL;
