// Database migration runner

#[cfg(feature = "server")]
use sqlx::{PgPool, Row};
#[cfg(feature = "server")]
use std::path::Path;
#[cfg(feature = "server")]
use tokio::fs;

#[cfg(feature = "server")]
#[derive(Debug)]
pub struct Migration {
    pub version: i32,
    pub name: String,
    pub sql: String,
}

#[cfg(feature = "server")]
pub struct MigrationRunner {
    pool: PgPool,
}

#[cfg(feature = "server")]
impl MigrationRunner {
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    /// Create migrations table if it doesn't exist
    pub async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get list of applied migrations
    pub async fn get_applied_migrations(&self) -> Result<Vec<i32>, Box<dyn std::error::Error>> {
        let rows = sqlx::query("SELECT version FROM _migrations ORDER BY version")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|row| row.get("version")).collect())
    }

    /// Load migrations from directory
    pub async fn load_migrations<P: AsRef<Path>>(
        &self,
        migrations_dir: P,
    ) -> Result<Vec<Migration>, Box<dyn std::error::Error>> {
        let mut migrations = Vec::new();
        let mut entries = fs::read_dir(migrations_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                let filename = path.file_name().unwrap().to_string_lossy();

                // Parse filename: 001_initial_schema.sql -> version=1, name=initial_schema
                if let Some((version_str, name_part)) = filename.split_once('_') {
                    if let Ok(version) = version_str.parse::<i32>() {
                        let name = name_part.trim_end_matches(".sql").to_string();
                        let sql = fs::read_to_string(&path).await?;

                        migrations.push(Migration { version, name, sql });
                    }
                }
            }
        }

        migrations.sort_by_key(|m| m.version);
        Ok(migrations)
    }

    /// Run pending migrations
    pub async fn migrate_up(
        &self,
        migrations: Vec<Migration>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let applied = self.get_applied_migrations().await?;
        let applied_set: std::collections::HashSet<i32> = applied.into_iter().collect();

        for migration in migrations {
            if applied_set.contains(&migration.version) {
                println!(
                    "  ⏭  Migration {} already applied: {}",
                    migration.version, migration.name
                );
                continue;
            }

            println!(
                "  ⚙  Applying migration {}: {}",
                migration.version, migration.name
            );

            // Begin transaction
            let mut tx = self.pool.begin().await?;

            // Execute migration SQL
            sqlx::query(&migration.sql)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to execute migration {}: {}", migration.version, e))?;

            // Record migration
            sqlx::query(
                "INSERT INTO _migrations (version, name, applied_at) VALUES ($1, $2, NOW())",
            )
            .bind(migration.version)
            .bind(&migration.name)
            .execute(&mut *tx)
            .await?;

            // Commit transaction
            tx.commit().await?;

            println!("  ✓  Migration {} applied successfully", migration.version);
        }

        Ok(())
    }

    /// Rollback last migration
    pub async fn migrate_down(
        &self,
        migrations: Vec<Migration>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let applied = self.get_applied_migrations().await?;

        if applied.is_empty() {
            println!("  ℹ  No migrations to rollback");
            return Ok(());
        }

        let last_version = applied.last().unwrap();

        // Find migration to rollback
        let migration = migrations.iter().find(|m| m.version == *last_version);

        if let Some(migration) = migration {
            println!(
                "  ⚙  Rolling back migration {}: {}",
                migration.version, migration.name
            );
            println!("  ⚠  Note: Down migrations are not yet supported");
            println!("  ℹ  To rollback, manually drop tables and remove entry from _migrations");

            // For now, just remove from migrations table
            // In a full implementation, we'd need separate down.sql files
            sqlx::query("DELETE FROM _migrations WHERE version = $1")
                .bind(migration.version)
                .execute(&self.pool)
                .await?;

            println!("  ✓  Migration {} marked as rolled back", migration.version);
        } else {
            println!(
                "  ⚠  Migration {} not found in migrations directory",
                last_version
            );
        }

        Ok(())
    }

    /// Show migration status
    pub async fn status(
        &self,
        migrations: Vec<Migration>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let applied = self.get_applied_migrations().await?;
        let applied_set: std::collections::HashSet<i32> = applied.into_iter().collect();

        println!("\n  Migration Status:");
        println!("  {}", "─".repeat(60));

        for migration in migrations {
            let status = if applied_set.contains(&migration.version) {
                "✓ Applied"
            } else {
                "✗ Pending"
            };

            println!(
                "  {} | {:3} | {}",
                status, migration.version, migration.name
            );
        }

        println!("  {}", "─".repeat(60));
        println!();

        Ok(())
    }

    pub async fn close(self) {
        self.pool.close().await;
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_migration_runner() {
        // This test requires a running PostgreSQL instance
        // Run with: cargo test --features server -- --ignored
    }
}
