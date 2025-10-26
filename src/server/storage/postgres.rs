// PostgreSQL storage backend implementation

#![allow(unused_variables)]

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use chrono::{DateTime, Utc};
#[cfg(feature = "server")]
use sqlx::{PgPool, Row};
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use super::traits::*;
#[cfg(feature = "server")]
use crate::dsl::schema::DSLWorkflow;

#[cfg(feature = "server")]
pub struct PostgresStorage {
    pool: PgPool,
}

#[cfg(feature = "server")]
impl PostgresStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl WorkflowStorage for PostgresStorage {
    async fn store_workflow(
        &self,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<Uuid> {
        let id = metadata.id;
        let definition_json = serde_json::to_value(workflow)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let created_by_uuid = metadata
            .created_by
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok());

        sqlx::query(
            r#"
            INSERT INTO workflows (
                id, name, version, description, definition,
                created_by, tags, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE
            SET
                definition = $5,
                description = $4,
                is_active = $8,
                updated_at = $10
            "#,
        )
        .bind(id)
        .bind(&metadata.name)
        .bind(&metadata.version)
        .bind(&metadata.description)
        .bind(definition_json)
        .bind(created_by_uuid)
        .bind(&metadata.tags)
        .bind(metadata.is_active)
        .bind(metadata.created_at)
        .bind(metadata.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    async fn get_workflow(&self, id: Uuid) -> Result<Option<(DSLWorkflow, WorkflowMetadata)>> {
        let row = sqlx::query(
            r#"
            SELECT
                id, name, version, description, definition,
                created_at, updated_at, created_by, tags, is_active
            FROM workflows
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let workflow: DSLWorkflow = serde_json::from_value(row.get("definition"))
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            let metadata = WorkflowMetadata {
                id: row.get("id"),
                name: row.get("name"),
                version: row.get("version"),
                description: row.get("description"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row
                    .try_get::<Option<Uuid>, _>("created_by")
                    .ok()
                    .flatten()
                    .map(|u| u.to_string()),
                tags: row
                    .get::<Option<Vec<String>>, _>("tags")
                    .unwrap_or_default(),
                is_active: row.get("is_active"),
            };

            Ok(Some((workflow, metadata)))
        } else {
            Ok(None)
        }
    }

    async fn update_workflow(
        &self,
        id: Uuid,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<()> {
        let definition_json = serde_json::to_value(workflow)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        let result = sqlx::query(
            r#"
            UPDATE workflows
            SET
                definition = $2,
                description = $3,
                is_active = $4,
                tags = $5,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(definition_json)
        .bind(&metadata.description)
        .bind(metadata.is_active)
        .bind(&metadata.tags)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("Workflow {} not found", id)));
        }

        Ok(())
    }

    async fn delete_workflow(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM workflows WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("Workflow {} not found", id)));
        }

        Ok(())
    }

    async fn list_workflows(
        &self,
        filter: &WorkflowFilter,
    ) -> Result<Vec<(DSLWorkflow, WorkflowMetadata)>> {
        let limit = filter.limit.unwrap_or(100) as i64;
        let offset = filter.offset.unwrap_or(0) as i64;

        let mut query_str = String::from(
            r#"
            SELECT
                id, name, version, description, definition,
                created_at, updated_at, created_by, tags, is_active
            FROM workflows
            WHERE 1=1
            "#,
        );

        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();

        if let Some(ref name) = filter.name {
            query_str.push_str(&format!(" AND name ILIKE ${}", params.len() + 1));
            params.push(Box::new(format!("%{}%", name)));
        }

        if let Some(ref created_by) = filter.created_by {
            if let Ok(uuid) = Uuid::parse_str(created_by) {
                query_str.push_str(&format!(" AND created_by = ${}", params.len() + 1));
                params.push(Box::new(uuid));
            }
        }

        if let Some(is_active) = filter.is_active {
            query_str.push_str(&format!(" AND is_active = ${}", params.len() + 1));
            params.push(Box::new(is_active));
        }

        query_str.push_str(" ORDER BY created_at DESC");
        query_str.push_str(&format!(
            " LIMIT ${} OFFSET ${}",
            params.len() + 1,
            params.len() + 2
        ));

        // For simplicity, use a raw query
        let rows = sqlx::query(&query_str)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();

        for row in rows {
            let workflow: DSLWorkflow = serde_json::from_value(row.get("definition"))
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            let metadata = WorkflowMetadata {
                id: row.get("id"),
                name: row.get("name"),
                version: row.get("version"),
                description: row.get("description"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row
                    .try_get::<Option<Uuid>, _>("created_by")
                    .ok()
                    .flatten()
                    .map(|u| u.to_string()),
                tags: row
                    .get::<Option<Vec<String>>, _>("tags")
                    .unwrap_or_default(),
                is_active: row.get("is_active"),
            };

            results.push((workflow, metadata));
        }

        Ok(results)
    }

    async fn get_workflow_version(
        &self,
        id: Uuid,
        version: &str,
    ) -> Result<Option<(DSLWorkflow, WorkflowMetadata)>> {
        let row = sqlx::query(
            r#"
            SELECT
                id, name, version, description, definition,
                created_at, updated_at, created_by, tags, is_active
            FROM workflows
            WHERE id = $1 AND version = $2
            "#,
        )
        .bind(id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let workflow: DSLWorkflow = serde_json::from_value(row.get("definition"))
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            let metadata = WorkflowMetadata {
                id: row.get("id"),
                name: row.get("name"),
                version: row.get("version"),
                description: row.get("description"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row
                    .try_get::<Option<Uuid>, _>("created_by")
                    .ok()
                    .flatten()
                    .map(|u| u.to_string()),
                tags: row
                    .get::<Option<Vec<String>>, _>("tags")
                    .unwrap_or_default(),
                is_active: row.get("is_active"),
            };

            Ok(Some((workflow, metadata)))
        } else {
            Ok(None)
        }
    }

    async fn store_workflow_version(
        &self,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<()> {
        // In PostgreSQL, versions are just separate rows with different version strings
        self.store_workflow(workflow, metadata).await?;
        Ok(())
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl ExecutionStorage for PostgresStorage {
    async fn store_execution(&self, execution: &Execution) -> Result<Uuid> {
        let id = execution.id;
        let status_str = match execution.status {
            ExecutionStatus::Queued => "queued",
            ExecutionStatus::Running => "running",
            ExecutionStatus::Completed => "completed",
            ExecutionStatus::Failed => "failed",
            ExecutionStatus::Cancelled => "cancelled",
            ExecutionStatus::Paused => "paused",
        };

        sqlx::query(
            r#"
            INSERT INTO executions (
                id, workflow_id, workflow_version, status,
                started_at, completed_at, created_at,
                triggered_by, trigger_type, input_params,
                result, error, retry_count, parent_execution_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (id) DO UPDATE
            SET
                status = $4,
                started_at = $5,
                completed_at = $6,
                result = $11,
                error = $12,
                retry_count = $13
            "#,
        )
        .bind(id)
        .bind(execution.workflow_id)
        .bind(&execution.workflow_version)
        .bind(status_str)
        .bind(execution.started_at)
        .bind(execution.completed_at)
        .bind(execution.created_at)
        .bind(
            execution
                .triggered_by
                .as_ref()
                .and_then(|s| Uuid::parse_str(s).ok()),
        )
        .bind(&execution.trigger_type)
        .bind(&execution.input_params)
        .bind(&execution.result)
        .bind(&execution.error)
        .bind(execution.retry_count as i32)
        .bind(execution.parent_execution_id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    async fn get_execution(&self, id: Uuid) -> Result<Option<Execution>> {
        let row = sqlx::query(
            r#"
            SELECT
                id, workflow_id, workflow_version, status,
                started_at, completed_at, created_at,
                triggered_by, trigger_type, input_params,
                result, error, retry_count, parent_execution_id
            FROM executions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "queued" => ExecutionStatus::Queued,
                "running" => ExecutionStatus::Running,
                "completed" => ExecutionStatus::Completed,
                "failed" => ExecutionStatus::Failed,
                "cancelled" => ExecutionStatus::Cancelled,
                "paused" => ExecutionStatus::Paused,
                _ => ExecutionStatus::Queued,
            };

            Ok(Some(Execution {
                id: row.get("id"),
                workflow_id: row.get("workflow_id"),
                workflow_version: row.get("workflow_version"),
                status,
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                created_at: row.get("created_at"),
                triggered_by: row
                    .try_get::<Option<Uuid>, _>("triggered_by")
                    .ok()
                    .flatten()
                    .map(|u| u.to_string()),
                trigger_type: row.get("trigger_type"),
                input_params: row.get("input_params"),
                result: row.get("result"),
                error: row.get("error"),
                retry_count: row.get::<i32, _>("retry_count") as u32,
                parent_execution_id: row.get("parent_execution_id"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_execution(&self, _id: Uuid, execution: &Execution) -> Result<()> {
        self.store_execution(execution).await?;
        Ok(())
    }

    async fn list_executions(&self, filter: &ExecutionFilter) -> Result<Vec<Execution>> {
        let limit = filter.limit.unwrap_or(100) as i64;
        let offset = filter.offset.unwrap_or(0) as i64;

        let mut query_str = String::from(
            r#"
            SELECT
                id, workflow_id, workflow_version, status,
                started_at, completed_at, created_at,
                triggered_by, trigger_type, input_params,
                result, error, retry_count, parent_execution_id
            FROM executions
            WHERE 1=1
            "#,
        );

        if let Some(workflow_id) = filter.workflow_id {
            query_str.push_str(&format!(" AND workflow_id = '{}'", workflow_id));
        }

        if let Some(ref status) = filter.status {
            let status_str = match status {
                ExecutionStatus::Queued => "queued",
                ExecutionStatus::Running => "running",
                ExecutionStatus::Completed => "completed",
                ExecutionStatus::Failed => "failed",
                ExecutionStatus::Cancelled => "cancelled",
                ExecutionStatus::Paused => "paused",
            };
            query_str.push_str(&format!(" AND status = '{}'", status_str));
        }

        query_str.push_str(" ORDER BY created_at DESC");
        query_str.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let rows = sqlx::query(&query_str)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();

        for row in rows {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "queued" => ExecutionStatus::Queued,
                "running" => ExecutionStatus::Running,
                "completed" => ExecutionStatus::Completed,
                "failed" => ExecutionStatus::Failed,
                "cancelled" => ExecutionStatus::Cancelled,
                "paused" => ExecutionStatus::Paused,
                _ => ExecutionStatus::Queued,
            };

            results.push(Execution {
                id: row.get("id"),
                workflow_id: row.get("workflow_id"),
                workflow_version: row.get("workflow_version"),
                status,
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                created_at: row.get("created_at"),
                triggered_by: row
                    .try_get::<Option<Uuid>, _>("triggered_by")
                    .ok()
                    .flatten()
                    .map(|u| u.to_string()),
                trigger_type: row.get("trigger_type"),
                input_params: row.get("input_params"),
                result: row.get("result"),
                error: row.get("error"),
                retry_count: row.get::<i32, _>("retry_count") as u32,
                parent_execution_id: row.get("parent_execution_id"),
            });
        }

        Ok(results)
    }

    async fn store_execution_log(&self, log: &ExecutionLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO execution_logs (
                execution_id, task_execution_id, timestamp,
                level, message, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(log.execution_id)
        .bind(log.task_execution_id)
        .bind(log.timestamp)
        .bind(&log.level)
        .bind(&log.message)
        .bind(&log.metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_execution_logs(
        &self,
        execution_id: Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<ExecutionLog>> {
        let limit = limit.unwrap_or(1000) as i64;

        let rows = sqlx::query(
            r#"
            SELECT id, execution_id, task_execution_id, timestamp, level, message, metadata
            FROM execution_logs
            WHERE execution_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
        )
        .bind(execution_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let logs = rows
            .into_iter()
            .map(|row| ExecutionLog {
                id: Some(row.get("id")),
                execution_id: row.get("execution_id"),
                task_execution_id: row.get("task_execution_id"),
                timestamp: row.get("timestamp"),
                level: row.get("level"),
                message: row.get("message"),
                metadata: row.get("metadata"),
            })
            .collect();

        Ok(logs)
    }

    async fn delete_execution(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM executions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!(
                "Execution {} not found",
                id
            )));
        }

        Ok(())
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl CheckpointStorage for PostgresStorage {
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<Uuid> {
        let id = checkpoint.id;

        sqlx::query(
            r#"
            INSERT INTO checkpoints (id, execution_id, checkpoint_name, state, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (execution_id, checkpoint_name) DO UPDATE
            SET state = $4, created_at = $5
            "#,
        )
        .bind(id)
        .bind(checkpoint.execution_id)
        .bind(&checkpoint.checkpoint_name)
        .bind(&checkpoint.state)
        .bind(checkpoint.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    async fn get_checkpoint(&self, execution_id: Uuid, name: &str) -> Result<Option<Checkpoint>> {
        let row = sqlx::query(
            r#"
            SELECT id, execution_id, checkpoint_name, state, created_at
            FROM checkpoints
            WHERE execution_id = $1 AND checkpoint_name = $2
            "#,
        )
        .bind(execution_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(Checkpoint {
                id: row.get("id"),
                execution_id: row.get("execution_id"),
                checkpoint_name: row.get("checkpoint_name"),
                state: row.get("state"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_checkpoints(&self, execution_id: Uuid) -> Result<Vec<Checkpoint>> {
        let rows = sqlx::query(
            r#"
            SELECT id, execution_id, checkpoint_name, state, created_at
            FROM checkpoints
            WHERE execution_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(execution_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let checkpoints = rows
            .into_iter()
            .map(|row| Checkpoint {
                id: row.get("id"),
                execution_id: row.get("execution_id"),
                checkpoint_name: row.get("checkpoint_name"),
                state: row.get("state"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(checkpoints)
    }

    async fn delete_checkpoint(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM checkpoints WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!(
                "Checkpoint {} not found",
                id
            )));
        }

        Ok(())
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl ScheduleStorage for PostgresStorage {
    async fn store_schedule(&self, schedule: &Schedule) -> Result<Uuid> {
        sqlx::query(
            r#"
            INSERT INTO schedules (
                id, workflow_id, cron_expression, timezone, is_active,
                input_params, created_at, updated_at, created_by,
                last_run_at, next_run_at, description
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(schedule.id)
        .bind(schedule.workflow_id)
        .bind(&schedule.cron_expression)
        .bind(&schedule.timezone)
        .bind(schedule.is_active)
        .bind(&schedule.input_params)
        .bind(schedule.created_at)
        .bind(schedule.updated_at)
        .bind(&schedule.created_by)
        .bind(schedule.last_run_at)
        .bind(schedule.next_run_at)
        .bind(&schedule.description)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(schedule.id)
    }

    async fn get_schedule(&self, id: Uuid) -> Result<Option<Schedule>> {
        let row = sqlx::query(
            r#"
            SELECT id, workflow_id, cron_expression, timezone, is_active,
                   input_params, created_at, updated_at, created_by,
                   last_run_at, next_run_at, description
            FROM schedules
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(Schedule {
                id: row.get("id"),
                workflow_id: row.get("workflow_id"),
                cron_expression: row.get("cron_expression"),
                timezone: row.get("timezone"),
                is_active: row.get("is_active"),
                input_params: row.get("input_params"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
                last_run_at: row.get("last_run_at"),
                next_run_at: row.get("next_run_at"),
                description: row.get("description"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_schedule(&self, id: Uuid, schedule: &Schedule) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE schedules SET
                workflow_id = $2,
                cron_expression = $3,
                timezone = $4,
                is_active = $5,
                input_params = $6,
                updated_at = $7,
                last_run_at = $8,
                next_run_at = $9,
                description = $10
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(schedule.workflow_id)
        .bind(&schedule.cron_expression)
        .bind(&schedule.timezone)
        .bind(schedule.is_active)
        .bind(&schedule.input_params)
        .bind(schedule.updated_at)
        .bind(schedule.last_run_at)
        .bind(schedule.next_run_at)
        .bind(&schedule.description)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("Schedule {} not found", id)));
        }

        Ok(())
    }

    async fn delete_schedule(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM schedules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("Schedule {} not found", id)));
        }

        Ok(())
    }

    async fn list_schedules(&self, filter: &ScheduleFilter) -> Result<Vec<Schedule>> {
        let mut query = String::from(
            r#"
            SELECT id, workflow_id, cron_expression, timezone, is_active,
                   input_params, created_at, updated_at, created_by,
                   last_run_at, next_run_at, description
            FROM schedules
            WHERE 1=1
            "#,
        );

        if filter.workflow_id.is_some() {
            query.push_str(" AND workflow_id = $1");
        }
        if filter.is_active.is_some() {
            query.push_str(if filter.workflow_id.is_some() {
                " AND is_active = $2"
            } else {
                " AND is_active = $1"
            });
        }
        if filter.created_by.is_some() {
            let idx = [filter.workflow_id.is_some(), filter.is_active.is_some()]
                .iter()
                .filter(|&&x| x)
                .count()
                + 1;
            query.push_str(&format!(" AND created_by = ${}", idx));
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            let idx = [
                filter.workflow_id.is_some(),
                filter.is_active.is_some(),
                filter.created_by.is_some(),
            ]
            .iter()
            .filter(|&&x| x)
            .count()
                + 1;
            query.push_str(&format!(" LIMIT ${}", idx));
        }
        if let Some(offset) = filter.offset {
            let idx = [
                filter.workflow_id.is_some(),
                filter.is_active.is_some(),
                filter.created_by.is_some(),
                filter.limit.is_some(),
            ]
            .iter()
            .filter(|&&x| x)
            .count()
                + 1;
            query.push_str(&format!(" OFFSET ${}", idx));
        }

        let mut q = sqlx::query(&query);

        if let Some(workflow_id) = filter.workflow_id {
            q = q.bind(workflow_id);
        }
        if let Some(is_active) = filter.is_active {
            q = q.bind(is_active);
        }
        if let Some(ref created_by) = filter.created_by {
            q = q.bind(created_by);
        }
        if let Some(limit) = filter.limit {
            q = q.bind(limit as i64);
        }
        if let Some(offset) = filter.offset {
            q = q.bind(offset as i64);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let schedules = rows
            .into_iter()
            .map(|row| Schedule {
                id: row.get("id"),
                workflow_id: row.get("workflow_id"),
                cron_expression: row.get("cron_expression"),
                timezone: row.get("timezone"),
                is_active: row.get("is_active"),
                input_params: row.get("input_params"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
                last_run_at: row.get("last_run_at"),
                next_run_at: row.get("next_run_at"),
                description: row.get("description"),
            })
            .collect();

        Ok(schedules)
    }

    async fn get_due_schedules(&self, before: DateTime<Utc>) -> Result<Vec<Schedule>> {
        let rows = sqlx::query(
            r#"
            SELECT id, workflow_id, cron_expression, timezone, is_active,
                   input_params, created_at, updated_at, created_by,
                   last_run_at, next_run_at, description
            FROM schedules
            WHERE is_active = TRUE
              AND (next_run_at IS NULL OR next_run_at <= $1)
            ORDER BY next_run_at ASC NULLS FIRST
            "#,
        )
        .bind(before)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let schedules = rows
            .into_iter()
            .map(|row| Schedule {
                id: row.get("id"),
                workflow_id: row.get("workflow_id"),
                cron_expression: row.get("cron_expression"),
                timezone: row.get("timezone"),
                is_active: row.get("is_active"),
                input_params: row.get("input_params"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
                last_run_at: row.get("last_run_at"),
                next_run_at: row.get("next_run_at"),
                description: row.get("description"),
            })
            .collect();

        Ok(schedules)
    }

    async fn store_schedule_run(&self, run: &ScheduleRun) -> Result<Uuid> {
        let status_str = match run.status {
            ScheduleRunStatus::Scheduled => "scheduled",
            ScheduleRunStatus::Running => "running",
            ScheduleRunStatus::Completed => "completed",
            ScheduleRunStatus::Failed => "failed",
            ScheduleRunStatus::Skipped => "skipped",
        };

        sqlx::query(
            r#"
            INSERT INTO schedule_runs (
                id, schedule_id, execution_id, scheduled_for,
                started_at, status, error, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(run.id)
        .bind(run.schedule_id)
        .bind(run.execution_id)
        .bind(run.scheduled_for)
        .bind(run.started_at)
        .bind(status_str)
        .bind(&run.error)
        .bind(run.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(run.id)
    }

    async fn get_schedule_runs(
        &self,
        schedule_id: Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<ScheduleRun>> {
        let limit_val = limit.unwrap_or(100) as i64;

        let rows = sqlx::query(
            r#"
            SELECT id, schedule_id, execution_id, scheduled_for,
                   started_at, status, error, created_at
            FROM schedule_runs
            WHERE schedule_id = $1
            ORDER BY scheduled_for DESC
            LIMIT $2
            "#,
        )
        .bind(schedule_id)
        .bind(limit_val)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let runs = rows
            .into_iter()
            .map(|row| {
                let status_str: String = row.get("status");
                let status = match status_str.as_str() {
                    "scheduled" => ScheduleRunStatus::Scheduled,
                    "running" => ScheduleRunStatus::Running,
                    "completed" => ScheduleRunStatus::Completed,
                    "failed" => ScheduleRunStatus::Failed,
                    "skipped" => ScheduleRunStatus::Skipped,
                    _ => ScheduleRunStatus::Failed,
                };

                ScheduleRun {
                    id: row.get("id"),
                    schedule_id: row.get("schedule_id"),
                    execution_id: row.get("execution_id"),
                    scheduled_for: row.get("scheduled_for"),
                    started_at: row.get("started_at"),
                    status,
                    error: row.get("error"),
                    created_at: row.get("created_at"),
                }
            })
            .collect();

        Ok(runs)
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl OrganizationStorage for PostgresStorage {
    async fn store_organization(&self, organization: &Organization) -> Result<Uuid> {
        sqlx::query(
            r#"
            INSERT INTO organizations (
                id, name, slug, description, logo_url, plan,
                settings, created_at, updated_at, is_active
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(organization.id)
        .bind(&organization.name)
        .bind(&organization.slug)
        .bind(&organization.description)
        .bind(&organization.logo_url)
        .bind(&organization.plan)
        .bind(&organization.settings)
        .bind(organization.created_at)
        .bind(organization.updated_at)
        .bind(organization.is_active)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(organization.id)
    }

    async fn get_organization(&self, id: Uuid) -> Result<Option<Organization>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, slug, description, logo_url, plan,
                   settings, created_at, updated_at, is_active
            FROM organizations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(Organization {
                id: row.get("id"),
                name: row.get("name"),
                slug: row.get("slug"),
                description: row.get("description"),
                logo_url: row.get("logo_url"),
                plan: row.get("plan"),
                settings: row.get("settings"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_organization_by_slug(&self, slug: &str) -> Result<Option<Organization>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, slug, description, logo_url, plan,
                   settings, created_at, updated_at, is_active
            FROM organizations
            WHERE slug = $1
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(Organization {
                id: row.get("id"),
                name: row.get("name"),
                slug: row.get("slug"),
                description: row.get("description"),
                logo_url: row.get("logo_url"),
                plan: row.get("plan"),
                settings: row.get("settings"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_organization(&self, id: Uuid, organization: &Organization) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE organizations SET
                name = $2,
                slug = $3,
                description = $4,
                logo_url = $5,
                plan = $6,
                settings = $7,
                updated_at = $8,
                is_active = $9
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&organization.name)
        .bind(&organization.slug)
        .bind(&organization.description)
        .bind(&organization.logo_url)
        .bind(&organization.plan)
        .bind(&organization.settings)
        .bind(organization.updated_at)
        .bind(organization.is_active)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!(
                "Organization {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn delete_organization(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM organizations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!(
                "Organization {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn list_organizations(&self, filter: &OrganizationFilter) -> Result<Vec<Organization>> {
        let mut query = String::from(
            r#"
            SELECT id, name, slug, description, logo_url, plan,
                   settings, created_at, updated_at, is_active
            FROM organizations
            WHERE 1=1
            "#,
        );

        if filter.name.is_some() {
            query.push_str(" AND name ILIKE $1");
        }
        if filter.slug.is_some() {
            let idx = if filter.name.is_some() { 2 } else { 1 };
            query.push_str(&format!(" AND slug = ${}", idx));
        }
        if filter.is_active.is_some() {
            let idx = [filter.name.is_some(), filter.slug.is_some()]
                .iter()
                .filter(|&&x| x)
                .count()
                + 1;
            query.push_str(&format!(" AND is_active = ${}", idx));
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            let idx = [
                filter.name.is_some(),
                filter.slug.is_some(),
                filter.is_active.is_some(),
            ]
            .iter()
            .filter(|&&x| x)
            .count()
                + 1;
            query.push_str(&format!(" LIMIT ${}", idx));
        }
        if let Some(offset) = filter.offset {
            let idx = [
                filter.name.is_some(),
                filter.slug.is_some(),
                filter.is_active.is_some(),
                filter.limit.is_some(),
            ]
            .iter()
            .filter(|&&x| x)
            .count()
                + 1;
            query.push_str(&format!(" OFFSET ${}", idx));
        }

        let mut q = sqlx::query(&query);

        if let Some(ref name) = filter.name {
            q = q.bind(format!("%{}%", name));
        }
        if let Some(ref slug) = filter.slug {
            q = q.bind(slug);
        }
        if let Some(is_active) = filter.is_active {
            q = q.bind(is_active);
        }
        if let Some(limit) = filter.limit {
            q = q.bind(limit as i64);
        }
        if let Some(offset) = filter.offset {
            q = q.bind(offset as i64);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let organizations = rows
            .into_iter()
            .map(|row| Organization {
                id: row.get("id"),
                name: row.get("name"),
                slug: row.get("slug"),
                description: row.get("description"),
                logo_url: row.get("logo_url"),
                plan: row.get("plan"),
                settings: row.get("settings"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                is_active: row.get("is_active"),
            })
            .collect();

        Ok(organizations)
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl TeamStorage for PostgresStorage {
    async fn store_team(&self, team: &Team) -> Result<Uuid> {
        sqlx::query(
            r#"
            INSERT INTO teams (
                id, organization_id, name, description,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(team.id)
        .bind(team.organization_id)
        .bind(&team.name)
        .bind(&team.description)
        .bind(team.created_at)
        .bind(team.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(team.id)
    }

    async fn get_team(&self, id: Uuid) -> Result<Option<Team>> {
        let row = sqlx::query(
            r#"
            SELECT id, organization_id, name, description,
                   created_at, updated_at
            FROM teams
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(Team {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                name: row.get("name"),
                description: row.get("description"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_team(&self, id: Uuid, team: &Team) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE teams SET
                name = $2,
                description = $3,
                updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&team.name)
        .bind(&team.description)
        .bind(team.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("Team {} not found", id)));
        }

        Ok(())
    }

    async fn delete_team(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM teams WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("Team {} not found", id)));
        }

        Ok(())
    }

    async fn list_teams(&self, filter: &TeamFilter) -> Result<Vec<Team>> {
        let mut query = String::from(
            r#"
            SELECT id, organization_id, name, description,
                   created_at, updated_at
            FROM teams
            WHERE 1=1
            "#,
        );

        if filter.organization_id.is_some() {
            query.push_str(" AND organization_id = $1");
        }
        if filter.name.is_some() {
            let idx = if filter.organization_id.is_some() {
                2
            } else {
                1
            };
            query.push_str(&format!(" AND name ILIKE ${}", idx));
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            let idx = [filter.organization_id.is_some(), filter.name.is_some()]
                .iter()
                .filter(|&&x| x)
                .count()
                + 1;
            query.push_str(&format!(" LIMIT ${}", idx));
        }
        if let Some(offset) = filter.offset {
            let idx = [
                filter.organization_id.is_some(),
                filter.name.is_some(),
                filter.limit.is_some(),
            ]
            .iter()
            .filter(|&&x| x)
            .count()
                + 1;
            query.push_str(&format!(" OFFSET ${}", idx));
        }

        let mut q = sqlx::query(&query);

        if let Some(organization_id) = filter.organization_id {
            q = q.bind(organization_id);
        }
        if let Some(ref name) = filter.name {
            q = q.bind(format!("%{}%", name));
        }
        if let Some(limit) = filter.limit {
            q = q.bind(limit as i64);
        }
        if let Some(offset) = filter.offset {
            q = q.bind(offset as i64);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let teams = rows
            .into_iter()
            .map(|row| Team {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                name: row.get("name"),
                description: row.get("description"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(teams)
    }

    async fn add_team_member(&self, member: &TeamMember) -> Result<Uuid> {
        sqlx::query(
            r#"
            INSERT INTO team_members (
                id, team_id, user_id, role, added_at, added_by
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(member.id)
        .bind(member.team_id)
        .bind(member.user_id)
        .bind(&member.role)
        .bind(member.added_at)
        .bind(member.added_by)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(member.id)
    }

    async fn remove_team_member(&self, team_id: Uuid, user_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM team_members WHERE team_id = $1 AND user_id = $2")
            .bind(team_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound("Team member not found".to_string()));
        }

        Ok(())
    }

    async fn get_team_members(&self, team_id: Uuid) -> Result<Vec<TeamMember>> {
        let rows = sqlx::query(
            r#"
            SELECT id, team_id, user_id, role, added_at, added_by
            FROM team_members
            WHERE team_id = $1
            ORDER BY added_at DESC
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let members = rows
            .into_iter()
            .map(|row| TeamMember {
                id: row.get("id"),
                team_id: row.get("team_id"),
                user_id: row.get("user_id"),
                role: row.get("role"),
                added_at: row.get("added_at"),
                added_by: row.get("added_by"),
            })
            .collect();

        Ok(members)
    }

    async fn get_user_teams(&self, user_id: Uuid) -> Result<Vec<Team>> {
        let rows = sqlx::query(
            r#"
            SELECT t.id, t.organization_id, t.name, t.description,
                   t.created_at, t.updated_at
            FROM teams t
            INNER JOIN team_members tm ON t.id = tm.team_id
            WHERE tm.user_id = $1
            ORDER BY t.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let teams = rows
            .into_iter()
            .map(|row| Team {
                id: row.get("id"),
                organization_id: row.get("organization_id"),
                name: row.get("name"),
                description: row.get("description"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(teams)
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl ApiKeyStorage for PostgresStorage {
    async fn store_api_key(&self, api_key: &ApiKey) -> Result<Uuid> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (
                id, user_id, key_hash, key_prefix, name, description,
                scopes, created_at, expires_at, last_used_at, is_active
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(api_key.id)
        .bind(api_key.user_id)
        .bind(&api_key.key_hash)
        .bind(&api_key.key_prefix)
        .bind(&api_key.name)
        .bind(&api_key.description)
        .bind(&api_key.scopes)
        .bind(api_key.created_at)
        .bind(api_key.expires_at)
        .bind(api_key.last_used_at)
        .bind(api_key.is_active)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(api_key.id)
    }

    async fn get_api_key(&self, id: Uuid) -> Result<Option<ApiKey>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, key_hash, key_prefix, name, description,
                   scopes, created_at, expires_at, last_used_at, is_active
            FROM api_keys WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(ApiKey {
                id: row.get("id"),
                user_id: row.get("user_id"),
                key_hash: row.get("key_hash"),
                key_prefix: row.get("key_prefix"),
                name: row.get("name"),
                description: row.get("description"),
                scopes: row.get("scopes"),
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
                last_used_at: row.get("last_used_at"),
                is_active: row.get("is_active"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, key_hash, key_prefix, name, description,
                   scopes, created_at, expires_at, last_used_at, is_active
            FROM api_keys WHERE key_hash = $1 AND is_active = TRUE
            "#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(ApiKey {
                id: row.get("id"),
                user_id: row.get("user_id"),
                key_hash: row.get("key_hash"),
                key_prefix: row.get("key_prefix"),
                name: row.get("name"),
                description: row.get("description"),
                scopes: row.get("scopes"),
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
                last_used_at: row.get("last_used_at"),
                is_active: row.get("is_active"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_api_key(&self, id: Uuid, api_key: &ApiKey) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE api_keys SET
                name = $2, description = $3, scopes = $4,
                expires_at = $5, is_active = $6
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&api_key.name)
        .bind(&api_key.description)
        .bind(&api_key.scopes)
        .bind(api_key.expires_at)
        .bind(api_key.is_active)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("API key {} not found", id)));
        }

        Ok(())
    }

    async fn revoke_api_key(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("UPDATE api_keys SET is_active = FALSE WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("API key {} not found", id)));
        }

        Ok(())
    }

    async fn delete_api_key(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(format!("API key {} not found", id)));
        }

        Ok(())
    }

    async fn list_api_keys(&self, filter: &ApiKeyFilter) -> Result<Vec<ApiKey>> {
        let mut query = String::from(
            r#"
            SELECT id, user_id, key_hash, key_prefix, name, description,
                   scopes, created_at, expires_at, last_used_at, is_active
            FROM api_keys WHERE 1=1
            "#,
        );

        if filter.user_id.is_some() {
            query.push_str(" AND user_id = $1");
        }
        if filter.is_active.is_some() {
            let idx = if filter.user_id.is_some() { 2 } else { 1 };
            query.push_str(&format!(" AND is_active = ${}", idx));
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            let idx = [filter.user_id.is_some(), filter.is_active.is_some()]
                .iter()
                .filter(|&&x| x)
                .count()
                + 1;
            query.push_str(&format!(" LIMIT ${}", idx));
        }

        let mut q = sqlx::query(&query);

        if let Some(user_id) = filter.user_id {
            q = q.bind(user_id);
        }
        if let Some(is_active) = filter.is_active {
            q = q.bind(is_active);
        }
        if let Some(limit) = filter.limit {
            q = q.bind(limit as i64);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let keys = rows
            .into_iter()
            .map(|row| ApiKey {
                id: row.get("id"),
                user_id: row.get("user_id"),
                key_hash: row.get("key_hash"),
                key_prefix: row.get("key_prefix"),
                name: row.get("name"),
                description: row.get("description"),
                scopes: row.get("scopes"),
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
                last_used_at: row.get("last_used_at"),
                is_active: row.get("is_active"),
            })
            .collect();

        Ok(keys)
    }

    async fn update_last_used(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
