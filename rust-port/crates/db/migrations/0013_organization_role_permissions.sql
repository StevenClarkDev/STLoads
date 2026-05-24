-- Organization-role permission matrix for enterprise RBAC.

INSERT INTO organization_roles (key, label, description, privileged)
VALUES
    ('support', 'Support', 'Internal or delegated support user with controlled support tooling access.', TRUE),
    ('integration_admin', 'Integration Admin', 'Organization integration administrator for TMS, API, and webhook operations.', TRUE)
ON CONFLICT (key) DO UPDATE SET
    label = EXCLUDED.label,
    description = EXCLUDED.description,
    privileged = EXCLUDED.privileged;

CREATE TABLE IF NOT EXISTS organization_role_permissions (
    role_key VARCHAR(64) NOT NULL,
    permission_key VARCHAR(100) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (role_key, permission_key),
    CONSTRAINT fk_organization_role_permissions_role_key FOREIGN KEY (role_key) REFERENCES organization_roles(key) ON DELETE CASCADE
);

INSERT INTO organization_role_permissions (role_key, permission_key)
VALUES
    ('owner', 'access_admin_portal'),
    ('owner', 'manage_users'),
    ('owner', 'manage_roles'),
    ('owner', 'manage_master_data'),
    ('owner', 'manage_loads'),
    ('owner', 'manage_dispatch_desk'),
    ('owner', 'manage_marketplace'),
    ('owner', 'manage_tracking'),
    ('owner', 'manage_payments'),
    ('owner', 'manage_tms_operations'),
    ('admin', 'access_admin_portal'),
    ('admin', 'manage_users'),
    ('admin', 'manage_roles'),
    ('admin', 'manage_master_data'),
    ('admin', 'manage_loads'),
    ('admin', 'manage_dispatch_desk'),
    ('admin', 'manage_marketplace'),
    ('admin', 'manage_tracking'),
    ('admin', 'manage_payments'),
    ('admin', 'manage_tms_operations'),
    ('finance', 'access_admin_portal'),
    ('finance', 'manage_payments'),
    ('operator', 'manage_loads'),
    ('operator', 'manage_dispatch_desk'),
    ('operator', 'manage_marketplace'),
    ('operator', 'manage_tracking'),
    ('operator', 'manage_tms_operations'),
    ('support', 'access_admin_portal'),
    ('support', 'manage_users'),
    ('integration_admin', 'access_admin_portal'),
    ('integration_admin', 'manage_tms_operations')
ON CONFLICT (role_key, permission_key) DO NOTHING;
