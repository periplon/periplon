-- Migration: Create roles and permissions system
-- Version: 004
-- Description: RBAC (Role-Based Access Control) and ABAC (Attribute-Based Access Control)

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    is_system BOOLEAN DEFAULT FALSE,
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_roles_organization_id ON roles(organization_id);
CREATE INDEX idx_roles_is_system ON roles(is_system);

-- Permissions table
CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    resource_type VARCHAR(100),
    action VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_permissions_resource_type ON permissions(resource_type);
CREATE INDEX idx_permissions_action ON permissions(action);

-- Role permissions junction table
CREATE TABLE IF NOT EXISTS role_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
    UNIQUE(role_id, permission_id)
);

CREATE INDEX idx_role_permissions_role_id ON role_permissions(role_id);
CREATE INDEX idx_role_permissions_permission_id ON role_permissions(permission_id);

-- User roles junction table
CREATE TABLE IF NOT EXISTS user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE(user_id, role_id)
);

CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX idx_user_roles_expires_at ON user_roles(expires_at) WHERE expires_at IS NOT NULL;

-- Permission policies table (for ABAC)
CREATE TABLE IF NOT EXISTS permission_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    conditions JSONB NOT NULL,
    effect VARCHAR(20) NOT NULL CHECK (effect IN ('allow', 'deny')),
    priority INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_permission_policies_organization_id ON permission_policies(organization_id);
CREATE INDEX idx_permission_policies_is_active ON permission_policies(is_active);
CREATE INDEX idx_permission_policies_priority ON permission_policies(priority DESC);

-- Insert default system roles
INSERT INTO roles (name, description, is_system) VALUES
    ('admin', 'System administrator with full access', TRUE),
    ('user', 'Regular user with standard permissions', TRUE),
    ('viewer', 'Read-only access to resources', TRUE)
ON CONFLICT (name) DO NOTHING;

-- Insert default permissions
INSERT INTO permissions (name, description, resource_type, action) VALUES
    ('workflow.create', 'Create workflows', 'workflow', 'create'),
    ('workflow.read', 'View workflows', 'workflow', 'read'),
    ('workflow.update', 'Update workflows', 'workflow', 'update'),
    ('workflow.delete', 'Delete workflows', 'workflow', 'delete'),
    ('workflow.execute', 'Execute workflows', 'workflow', 'execute'),
    ('execution.read', 'View executions', 'execution', 'read'),
    ('execution.cancel', 'Cancel executions', 'execution', 'cancel'),
    ('user.create', 'Create users', 'user', 'create'),
    ('user.read', 'View users', 'user', 'read'),
    ('user.update', 'Update users', 'user', 'update'),
    ('user.delete', 'Delete users', 'user', 'delete'),
    ('organization.manage', 'Manage organization settings', 'organization', 'manage'),
    ('team.manage', 'Manage teams', 'team', 'manage')
ON CONFLICT (name) DO NOTHING;

-- Grant admin role all permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
ON CONFLICT DO NOTHING;

-- Grant user role standard permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'user'
AND p.name IN (
    'workflow.create', 'workflow.read', 'workflow.update', 'workflow.execute',
    'execution.read', 'execution.cancel'
)
ON CONFLICT DO NOTHING;

-- Grant viewer role read-only permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'viewer'
AND p.name IN ('workflow.read', 'execution.read')
ON CONFLICT DO NOTHING;

-- Update triggers
CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_permission_policies_updated_at BEFORE UPDATE ON permission_policies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
