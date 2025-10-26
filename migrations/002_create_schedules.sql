-- Migration: Create schedules table
-- Version: 002
-- Description: Cron-based workflow scheduling

-- Schedules table
CREATE TABLE IF NOT EXISTS schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID REFERENCES workflows(id) ON DELETE CASCADE,
    cron_expression VARCHAR(255) NOT NULL,
    timezone VARCHAR(100) DEFAULT 'UTC',
    is_active BOOLEAN DEFAULT TRUE,
    input_params JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    created_by VARCHAR(255),
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    description TEXT
);

CREATE INDEX idx_schedules_workflow_id ON schedules(workflow_id);
CREATE INDEX idx_schedules_next_run_at ON schedules(next_run_at) WHERE is_active = TRUE;
CREATE INDEX idx_schedules_is_active ON schedules(is_active);

-- Schedule execution history
CREATE TABLE IF NOT EXISTS schedule_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id UUID REFERENCES schedules(id) ON DELETE CASCADE,
    execution_id UUID REFERENCES executions(id),
    scheduled_for TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    status VARCHAR(50) CHECK (status IN ('scheduled', 'running', 'completed', 'failed', 'skipped')),
    error TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_schedule_runs_schedule_id ON schedule_runs(schedule_id, scheduled_for DESC);
CREATE INDEX idx_schedule_runs_status ON schedule_runs(status);

-- Update trigger for schedules
CREATE TRIGGER update_schedules_updated_at BEFORE UPDATE ON schedules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
