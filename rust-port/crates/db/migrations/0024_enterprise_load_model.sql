ALTER TABLE loads
    ADD COLUMN IF NOT EXISTS freight_mode VARCHAR(32) NOT NULL DEFAULT 'FTL',
    ADD COLUMN IF NOT EXISTS visibility VARCHAR(32) NOT NULL DEFAULT 'public',
    ADD COLUMN IF NOT EXISTS service_level VARCHAR(64) NULL,
    ADD COLUMN IF NOT EXISTS customer_reference VARCHAR(128) NULL,
    ADD COLUMN IF NOT EXISTS po_number VARCHAR(128) NULL,
    ADD COLUMN IF NOT EXISTS pickup_appointment_ref VARCHAR(128) NULL,
    ADD COLUMN IF NOT EXISTS delivery_appointment_ref VARCHAR(128) NULL,
    ADD COLUMN IF NOT EXISTS facility_contact_name VARCHAR(255) NULL,
    ADD COLUMN IF NOT EXISTS facility_contact_phone VARCHAR(64) NULL,
    ADD COLUMN IF NOT EXISTS facility_contact_email VARCHAR(255) NULL,
    ADD COLUMN IF NOT EXISTS appointment_window_start TIMESTAMP(6) NULL,
    ADD COLUMN IF NOT EXISTS appointment_window_end TIMESTAMP(6) NULL,
    ADD COLUMN IF NOT EXISTS accessorial_flags JSONB NULL,
    ADD COLUMN IF NOT EXISTS temperature_data JSONB NULL,
    ADD COLUMN IF NOT EXISTS container_data JSONB NULL,
    ADD COLUMN IF NOT EXISTS securement_data JSONB NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_loads_visibility_enterprise'
    ) THEN
        ALTER TABLE loads
            ADD CONSTRAINT chk_loads_visibility_enterprise
            CHECK (visibility IN ('public', 'private', 'contract', 'internal'));
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_loads_org_visibility_status
    ON loads (organization_id, visibility, status)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_loads_customer_reference
    ON loads (customer_reference)
    WHERE customer_reference IS NOT NULL AND deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_loads_accessorial_flags_gin
    ON loads USING GIN (accessorial_flags)
    WHERE accessorial_flags IS NOT NULL;
