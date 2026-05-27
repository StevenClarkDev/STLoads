ALTER TABLE loads
    ADD COLUMN IF NOT EXISTS lifecycle_status VARCHAR(32) NOT NULL DEFAULT 'draft',
    ADD COLUMN IF NOT EXISTS revision_number INT NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS cloned_from_load_id BIGINT NULL,
    ADD COLUMN IF NOT EXISTS is_template BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS template_name VARCHAR(255) NULL,
    ADD COLUMN IF NOT EXISTS published_at TIMESTAMP(6) NULL,
    ADD COLUMN IF NOT EXISTS revised_at TIMESTAMP(6) NULL,
    ADD COLUMN IF NOT EXISTS cancelled_at TIMESTAMP(6) NULL,
    ADD COLUMN IF NOT EXISTS archived_at TIMESTAMP(6) NULL,
    ADD COLUMN IF NOT EXISTS lifecycle_reason TEXT NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_loads_lifecycle_status_enterprise'
    ) THEN
        ALTER TABLE loads
            ADD CONSTRAINT chk_loads_lifecycle_status_enterprise
            CHECK (lifecycle_status IN ('draft', 'published', 'revised', 'cancelled', 'archived'));
    END IF;

    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'fk_loads_cloned_from_load_id'
    ) THEN
        ALTER TABLE loads
            ADD CONSTRAINT fk_loads_cloned_from_load_id
            FOREIGN KEY (cloned_from_load_id) REFERENCES loads(id) ON DELETE SET NULL;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_loads_lifecycle_status
    ON loads (organization_id, lifecycle_status, updated_at)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_loads_templates
    ON loads (organization_id, is_template, template_name)
    WHERE is_template = TRUE AND deleted_at IS NULL;
