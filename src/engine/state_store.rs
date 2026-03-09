use crate::engine::TransferError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use tracing::{debug, info};

/// Persistent SQLite-backed store that tracks which artifacts have been transferred.
///
/// Used to implement idempotent, resumable migrations: before transferring an artifact
/// the engine queries `is_completed`; after a successful upload it calls `mark_completed`.
pub struct StateStore {
    pool: SqlitePool,
}

impl StateStore {
    /// Open (or create) the SQLite state database at `db_path` and run migrations.
    ///
    /// Accepts bare file paths (`".j2n/state.db"`), `sqlite:` URIs, and the special
    /// in-memory database `":memory:"`.  Parent directories are created if absent.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path or URI for the SQLite database file.
    ///
    /// # Errors
    ///
    /// Returns [`TransferError`] if the database cannot be opened or the schema
    /// migration fails.
    pub async fn new(db_path: &str) -> Result<Self, TransferError> {
        info!(path = %db_path, "Initializing StateStore");

        let path_without_scheme = match db_path {
            p if p.starts_with("sqlite://") => &p[9..],
            p if p.starts_with("sqlite:") => &p[7..],
            p => p,
        };

        if !path_without_scheme.is_empty() && path_without_scheme != ":memory:" {
            let parent = std::path::Path::new(path_without_scheme).parent();
            if let Some(p) = parent {
                if !p.as_os_str().is_empty() && !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
        }

        let options = SqliteConnectOptions::from_str(db_path)?.create_if_missing(true);

        let pool = SqlitePool::connect_with(options)
            .await
            .map_err(|e| TransferError::Config(format!("Failed to connect to SQLite: {}", e)))?;

        let store = Self { pool };
        store.migrate().await?;

        Ok(store)
    }

    async fn migrate(&self) -> Result<(), TransferError> {
        // Create metadata table if not exists
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS j2n_metadata (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TransferError::Config(format!("Failed to create metadata table: {}", e)))?;

        // Initialize version if missing
        sqlx::query(
            "INSERT OR IGNORE INTO j2n_metadata (key, value) VALUES ('schema_version', '1')",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            TransferError::Config(format!("Failed to initialize schema version: {}", e))
        })?;

        // Main transfer state table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS transfer_state (
                source_repo TEXT,
                path TEXT,
                target_repo TEXT,
                sha256 TEXT,
                size INTEGER,
                completed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (source_repo, path)
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TransferError::Config(format!("Migration failed: {}", e)))?;

        Ok(())
    }

    /// Check whether an artifact has already been successfully transferred.
    ///
    /// When `expected_sha256` is provided the stored hash must match; a record
    /// without a matching hash is treated as incomplete (e.g. a partial upload).
    ///
    /// # Arguments
    ///
    /// * `source_repo` - JFrog repository key.
    /// * `path` - Artifact path within the repository.
    /// * `expected_sha256` - Optional SHA-256 hash to verify integrity; may include
    ///   a `"sha256:"` prefix which is stripped before comparison.
    ///
    /// # Returns
    ///
    /// `true` if a record exists and the hash matches (or no hash was supplied).
    pub async fn is_completed(
        &self,
        source_repo: &str,
        path: &str,
        expected_sha256: Option<&str>,
    ) -> bool {
        use sqlx::Row;
        let result =
            sqlx::query("SELECT sha256 FROM transfer_state WHERE source_repo = ? AND path = ?")
                .bind(source_repo)
                .bind(path)
                .fetch_optional(&self.pool)
                .await;

        match result {
            Ok(Some(row)) => {
                let stored_sha256: String = row.get(0);
                if let Some(expected) = expected_sha256 {
                    let clean_expected = if expected.contains(':') {
                        expected.split(':').next_back().unwrap_or(expected)
                    } else {
                        expected
                    };
                    stored_sha256 == clean_expected
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    /// Record a successfully transferred artifact in the state database.
    ///
    /// Uses `INSERT OR REPLACE` so re-uploading the same artifact (e.g. after a
    /// hash mismatch) overwrites the previous record.
    ///
    /// # Arguments
    ///
    /// * `source_repo` - JFrog repository key.
    /// * `path` - Artifact path within the repository.
    /// * `target_repo` - Nexus repository name.
    /// * `sha256` - Hex-encoded SHA-256 digest of the transferred content.
    /// * `size` - Size in bytes.
    ///
    /// # Errors
    ///
    /// Returns [`TransferError`] if the database write fails.
    pub async fn mark_completed(
        &self,
        source_repo: &str,
        path: &str,
        target_repo: &str,
        sha256: &str,
        size: u64,
    ) -> Result<(), TransferError> {
        debug!(%path, "Marking artifact as completed in StateStore");

        sqlx::query(
            "INSERT OR REPLACE INTO transfer_state (source_repo, path, target_repo, sha256, size) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(source_repo)
        .bind(path)
        .bind(target_repo)
        .bind(sha256)
        .bind(size as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| TransferError::Config(format!("Failed to update state: {}", e)))?;

        Ok(())
    }

    /// Return aggregate statistics for all completed transfers.
    ///
    /// # Returns
    ///
    /// A tuple `(count, total_bytes)` where `count` is the number of completed
    /// artifacts and `total_bytes` is their combined size.
    ///
    /// # Errors
    ///
    /// Returns [`TransferError`] if the database query fails.
    pub async fn get_stats(&self) -> Result<(u64, u64), TransferError> {
        use sqlx::Row;
        let row = sqlx::query("SELECT COUNT(*), SUM(size) FROM transfer_state")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| TransferError::Config(format!("Failed to get stats: {}", e)))?;

        let count: i64 = row.get(0);
        let size: i64 = row.try_get(1).unwrap_or(0i64);

        Ok((count as u64, size as u64))
    }

    /// Retrieve all transfer records from the state database.
    ///
    /// Used by the audit report generator to produce a full CSV export.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of [`TransferRecord`]s ordered by insertion time (SQLite default).
    ///
    /// # Errors
    ///
    /// Returns [`TransferError`] if the database query fails.
    pub async fn get_all_records(&self) -> Result<Vec<TransferRecord>, TransferError> {
        use sqlx::Row;
        let rows = sqlx::query(
            "SELECT source_repo, path, target_repo, sha256, size, completed_at FROM transfer_state",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TransferError::Config(format!("Failed to fetch records: {}", e)))?;

        let mut records = Vec::new();
        for row in rows {
            records.push(TransferRecord {
                source_repo: row.get(0),
                path: row.get(1),
                target_repo: row.get(2),
                sha256: row.get(3),
                size: row.get::<i64, _>(4) as u64,
                completed_at: row.get(5),
            });
        }

        Ok(records)
    }
}

/// A single row from the `transfer_state` table, representing one completed transfer.
pub struct TransferRecord {
    pub source_repo: String,
    pub path: String,
    pub target_repo: String,
    pub sha256: String,
    pub size: u64,
    pub completed_at: String,
}
