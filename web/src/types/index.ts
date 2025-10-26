// API Types

export interface Workflow {
  id: string
  name: string
  version: string
  description?: string
  definition: WorkflowDefinition
  created_at: string
  updated_at: string
  created_by?: string
  tags: string[]
  is_active: boolean
}

export interface WorkflowDefinition {
  name: string
  version: string
  description?: string
  agents: Record<string, Agent>
  tasks: Record<string, Task>
  inputs?: Record<string, InputVariable>
  outputs?: Record<string, OutputVariable>
  secrets?: string[]
}

export interface Agent {
  description: string
  model?: string
  tools?: string[]
  permissions?: AgentPermissions
  inputs?: Record<string, InputVariable>
}

export interface AgentPermissions {
  mode?: string
  max_turns?: number
}

export interface Task {
  description: string
  agent: string
  depends_on?: string[]
  subtasks?: string[]
  output?: string
  inputs?: Record<string, any>
}

export interface InputVariable {
  type: string
  required?: boolean
  default?: any
  description?: string
}

export interface OutputVariable {
  source: {
    type: string
    path?: string
    task?: string
    key?: string
  }
}

export interface Execution {
  id: string
  workflow_id: string
  workflow_version: string
  status: ExecutionStatus
  started_at?: string
  completed_at?: string
  created_at: string
  triggered_by?: string
  trigger_type: string
  input_params?: any
  result?: any
  error?: string
  retry_count: number
  parent_execution_id?: string
}

export type ExecutionStatus =
  | 'queued'
  | 'running'
  | 'completed'
  | 'failed'
  | 'cancelled'
  | 'paused'

export interface ExecutionLog {
  id?: number
  execution_id: string
  task_execution_id?: string
  timestamp: string
  level: string
  message: string
  metadata?: any
}

export interface Schedule {
  id: string
  workflow_id: string
  cron_expression: string
  timezone: string
  is_active: boolean
  input_params?: any
  created_at: string
  updated_at: string
  created_by?: string
  last_run_at?: string
  next_run_at?: string
  description?: string
}

export interface Organization {
  id: string
  name: string
  slug: string
  description?: string
  logo_url?: string
  plan: string
  settings: any
  created_at: string
  updated_at: string
  is_active: boolean
}

export interface Team {
  id: string
  organization_id: string
  name: string
  description?: string
  created_at: string
  updated_at: string
}

export interface ApiKey {
  id: string
  key_prefix: string
  name?: string
  description?: string
  scopes: string[]
  created_at: string
  expires_at?: string
  last_used_at?: string
  is_active: boolean
}

export interface User {
  id: string
  username: string
  email: string
  full_name?: string
  is_active: boolean
  created_at: string
}

export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  access_token: string
  refresh_token: string
  user: User
}

export interface DashboardMetrics {
  total_workflows: number
  total_executions: number
  active_executions: number
  success_rate: number
  recent_executions: Execution[]
}
