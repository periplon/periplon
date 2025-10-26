-- Migration: Create resource ownership and sharing tables
-- Version: 006
-- Description: Resource-level permissions and sharing capabilities

-- Workflow ownership and permissions
CREATE TABLE IF NOT EXISTS workflow_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID REFERENCES workflows(id) ON DELETE CASCADE,
    subject_type VARCHAR(50) NOT NULL CHECK (subject_type IN ('user', 'team', 'organization', 'public')),
    subject_id UUID,
    permission_level VARCHAR(50) NOT NULL CHECK (permission_level IN ('owner', 'admin', 'write', 'read')),
    granted_by UUID REFERENCES users(id),
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_workflow_permissions_workflow_id ON workflow_permissions(workflow_id);
CREATE INDEX idx_workflow_permissions_subject ON workflow_permissions(subject_type, subject_id);
CREATE INDEX idx_workflow_permissions_expires_at ON workflow_permissions(expires_at) WHERE expires_at IS NOT NULL;

-- Execution ownership
CREATE TABLE IF NOT EXISTS execution_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID REFERENCES executions(id) ON DELETE CASCADE,
    subject_type VARCHAR(50) NOT NULL CHECK (subject_type IN ('user', 'team', 'organization')),
    subject_id UUID,
    permission_level VARCHAR(50) NOT NULL CHECK (permission_level IN ('owner', 'read')),
    granted_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_execution_permissions_execution_id ON execution_permissions(execution_id);
CREATE INDEX idx_execution_permissions_subject ON execution_permissions(subject_type, subject_id);

-- Workflow tags for categorization
CREATE TABLE IF NOT EXISTS workflow_tag_mappings (
    workflow_id UUID REFERENCES workflows(id) ON DELETE CASCADE,
    tag VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (workflow_id, tag)
);

CREATE INDEX idx_workflow_tag_mappings_tag ON workflow_tag_mappings(tag);

-- Workflow favorites
CREATE TABLE IF NOT EXISTS workflow_favorites (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    workflow_id UUID REFERENCES workflows(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, workflow_id)
);

CREATE INDEX idx_workflow_favorites_user_id ON workflow_favorites(user_id);

-- Resource usage tracking
CREATE TABLE IF NOT EXISTS resource_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    resource_type VARCHAR(100) NOT NULL,
    resource_id UUID,
    usage_date DATE NOT NULL,
    usage_count INT DEFAULT 0,
    compute_seconds BIGINT DEFAULT 0,
    storage_bytes BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, organization_id, resource_type, resource_id, usage_date)
);

CREATE INDEX idx_resource_usage_user_id ON resource_usage(user_id, usage_date DESC);
CREATE INDEX idx_resource_usage_organization_id ON resource_usage(organization_id, usage_date DESC);
CREATE INDEX idx_resource_usage_date ON resource_usage(usage_date DESC);

-- Add owner tracking to workflows table
ALTER TABLE workflows
ADD COLUMN IF NOT EXISTS owner_id UUID REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX idx_workflows_owner_id ON workflows(owner_id);

-- Add owner tracking to executions table
ALTER TABLE executions
ADD COLUMN IF NOT EXISTS owner_id UUID REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX idx_executions_owner_id ON executions(owner_id);

-- Function to automatically grant owner permission on workflow creation
CREATE OR REPLACE FUNCTION grant_workflow_owner_permission()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.owner_id IS NOT NULL THEN
        INSERT INTO workflow_permissions (workflow_id, subject_type, subject_id, permission_level, granted_by)
        VALUES (NEW.id, 'user', NEW.owner_id, 'owner', NEW.owner_id)
        ON CONFLICT DO NOTHING;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER workflow_owner_permission_trigger
AFTER INSERT ON workflows
FOR EACH ROW
EXECUTE FUNCTION grant_workflow_owner_permission();

-- Function to automatically grant owner permission on execution creation
CREATE OR REPLACE FUNCTION grant_execution_owner_permission()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.owner_id IS NOT NULL THEN
        INSERT INTO execution_permissions (execution_id, subject_type, subject_id, permission_level)
        VALUES (NEW.id, 'user', NEW.owner_id, 'owner')
        ON CONFLICT DO NOTHING;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER execution_owner_permission_trigger
AFTER INSERT ON executions
FOR EACH ROW
EXECUTE FUNCTION grant_execution_owner_permission();

-- View for checking user workflow permissions
CREATE OR REPLACE VIEW user_workflow_permissions AS
SELECT DISTINCT
    wp.workflow_id,
    u.id as user_id,
    CASE
        WHEN wp.permission_level = 'owner' THEN 4
        WHEN wp.permission_level = 'admin' THEN 3
        WHEN wp.permission_level = 'write' THEN 2
        WHEN wp.permission_level = 'read' THEN 1
        ELSE 0
    END as permission_level,
    wp.permission_level as permission_name
FROM workflow_permissions wp
JOIN users u ON (
    (wp.subject_type = 'user' AND wp.subject_id = u.id) OR
    (wp.subject_type = 'organization' AND wp.subject_id = u.organization_id) OR
    (wp.subject_type = 'team' AND wp.subject_id IN (
        SELECT team_id FROM team_members WHERE user_id = u.id
    )) OR
    (wp.subject_type = 'public')
)
WHERE (wp.expires_at IS NULL OR wp.expires_at > NOW());
