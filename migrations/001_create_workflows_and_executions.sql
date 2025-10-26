-- Migration: Create workflows and executions tables
-- Version: 001
-- Description: Core workflow management and execution tracking tables

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Workflows table
CREATE TABLE IF NOT EXISTS workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    definition JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by VARCHAR(255),
    tags TEXT[],
    is_active BOOLEAN DEFAULT TRUE,
    UNIQUE(name, version)
);

CREATE INDEX idx_workflows_name ON workflows(name);
CREATE INDEX idx_workflows_tags ON workflows USING GIN(tags);
CREATE INDEX idx_workflows_created_at ON workflows(created_at DESC);
CREATE INDEX idx_workflows_is_active ON workflows(is_active);

-- Executions table
CREATE TABLE IF NOT EXISTS executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID REFERENCES workflows(id),
    workflow_version VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL CHECK (status IN ('queued', 'running', 'completed', 'failed', 'cancelled', 'paused')),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    triggered_by VARCHAR(255),
    trigger_type VARCHAR(50),
    input_params JSONB,
    result JSONB,
    error TEXT,
    retry_count INT DEFAULT 0,
    parent_execution_id UUID REFERENCES executions(id)
);

CREATE INDEX idx_executions_workflow_id ON executions(workflow_id);
CREATE INDEX idx_executions_status ON executions(status);
CREATE INDEX idx_executions_started_at ON executions(started_at DESC);
CREATE INDEX idx_executions_created_at ON executions(created_at DESC);

-- Task executions table
CREATE TABLE IF NOT EXISTS task_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID REFERENCES executions(id) ON DELETE CASCADE,
    task_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL CHECK (status IN ('pending', 'running', 'completed', 'failed', 'skipped')),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    output TEXT,
    error TEXT,
    retry_count INT DEFAULT 0,
    parent_task_id UUID REFERENCES task_executions(id)
);

CREATE INDEX idx_task_executions_execution_id ON task_executions(execution_id);
CREATE INDEX idx_task_executions_status ON task_executions(status);
CREATE INDEX idx_task_executions_started_at ON task_executions(started_at DESC);

-- Execution logs table
CREATE TABLE IF NOT EXISTS execution_logs (
    id BIGSERIAL PRIMARY KEY,
    execution_id UUID REFERENCES executions(id) ON DELETE CASCADE,
    task_execution_id UUID REFERENCES task_executions(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    level VARCHAR(20) CHECK (level IN ('debug', 'info', 'warn', 'error')),
    message TEXT,
    metadata JSONB
);

CREATE INDEX idx_execution_logs_execution_id ON execution_logs(execution_id, timestamp DESC);
CREATE INDEX idx_execution_logs_task_execution_id ON execution_logs(task_execution_id, timestamp DESC);
CREATE INDEX idx_execution_logs_level ON execution_logs(level);

-- Checkpoints table
CREATE TABLE IF NOT EXISTS checkpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID REFERENCES executions(id) ON DELETE CASCADE,
    checkpoint_name VARCHAR(255),
    state JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_checkpoints_execution_id ON checkpoints(execution_id, created_at DESC);
CREATE INDEX idx_checkpoints_name ON checkpoints(execution_id, checkpoint_name);

-- Execution queue table
CREATE TABLE IF NOT EXISTS execution_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID REFERENCES workflows(id),
    execution_id UUID REFERENCES executions(id),
    priority INT DEFAULT 0,
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    scheduled_at TIMESTAMPTZ,
    claimed_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    worker_id VARCHAR(255),
    attempts INT DEFAULT 0,
    max_retries INT DEFAULT 3,
    last_error TEXT
);

CREATE INDEX idx_execution_queue_status ON execution_queue(status);
CREATE INDEX idx_execution_queue_priority ON execution_queue(priority DESC, created_at ASC);
CREATE INDEX idx_execution_queue_scheduled_at ON execution_queue(scheduled_at) WHERE status = 'pending';
CREATE INDEX idx_execution_queue_worker_id ON execution_queue(worker_id);
CREATE INDEX idx_execution_queue_updated_at ON execution_queue(updated_at) WHERE status = 'processing';

-- Update trigger for workflows
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_workflows_updated_at BEFORE UPDATE ON workflows
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_execution_queue_updated_at BEFORE UPDATE ON execution_queue
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
