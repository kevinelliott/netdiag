//! Traceroute results repository.

use crate::error::StorageResult;
use crate::models::{QueryOptions, StoredTracerouteResult};
use chrono::Utc;
use netdiag_types::diagnostics::TracerouteResult;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Repository for traceroute results.
#[derive(Clone)]
pub struct TracerouteRepository {
    pool: SqlitePool,
}

impl TracerouteRepository {
    /// Create a new traceroute repository.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Save traceroute result.
    pub async fn save(
        &self,
        result: &TracerouteResult,
        session_id: Option<Uuid>,
    ) -> StorageResult<StoredTracerouteResult> {
        let stored = StoredTracerouteResult {
            id: Uuid::new_v4(),
            session_id,
            target: result.target.to_string(),
            resolved_ip: Some(result.target.to_string()),
            hop_count: result.hops.len() as u32,
            reached: result.reached,
            duration_ms: result.duration.as_millis() as u64,
            protocol: format!("{:?}", result.protocol),
            created_at: Utc::now(),
            hops_data: serde_json::to_value(&result.hops).unwrap_or(serde_json::Value::Null),
        };

        sqlx::query(
            r#"
            INSERT INTO traceroute_results (
                id, session_id, target, resolved_ip, hop_count, reached,
                duration_ms, protocol, created_at, hops_data
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(stored.id.to_string())
        .bind(stored.session_id.map(|id| id.to_string()))
        .bind(&stored.target)
        .bind(&stored.resolved_ip)
        .bind(stored.hop_count as i64)
        .bind(stored.reached)
        .bind(stored.duration_ms as i64)
        .bind(&stored.protocol)
        .bind(stored.created_at.to_rfc3339())
        .bind(stored.hops_data.to_string())
        .execute(&self.pool)
        .await?;

        Ok(stored)
    }

    /// Get traceroute result by ID.
    pub async fn get(&self, id: Uuid) -> StorageResult<Option<StoredTracerouteResult>> {
        let row: Option<TracerouteRow> = sqlx::query_as(
            r#"
            SELECT id, session_id, target, resolved_ip, hop_count, reached,
                   duration_ms, protocol, created_at, hops_data
            FROM traceroute_results
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    /// List traceroute results with options.
    pub async fn list(&self, options: &QueryOptions) -> StorageResult<Vec<StoredTracerouteResult>> {
        let limit = options.limit.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);

        let mut query = String::from(
            r#"
            SELECT id, session_id, target, resolved_ip, hop_count, reached,
                   duration_ms, protocol, created_at, hops_data
            FROM traceroute_results
            WHERE 1=1
            "#,
        );

        if options.target.is_some() {
            query.push_str(" AND target = ?");
        }
        if options.session_id.is_some() {
            query.push_str(" AND session_id = ?");
        }
        if options.from.is_some() {
            query.push_str(" AND created_at >= ?");
        }
        if options.to.is_some() {
            query.push_str(" AND created_at <= ?");
        }

        query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let mut q = sqlx::query_as::<_, TracerouteRow>(&query);

        if let Some(ref target) = options.target {
            q = q.bind(target);
        }
        if let Some(session_id) = options.session_id {
            q = q.bind(session_id.to_string());
        }
        if let Some(from) = options.from {
            q = q.bind(from.to_rfc3339());
        }
        if let Some(to) = options.to {
            q = q.bind(to.to_rfc3339());
        }

        q = q.bind(limit).bind(offset);

        let rows: Vec<TracerouteRow> = q.fetch_all(&self.pool).await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Delete traceroute result.
    pub async fn delete(&self, id: Uuid) -> StorageResult<()> {
        sqlx::query("DELETE FROM traceroute_results WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

/// Internal row type for query.
#[derive(sqlx::FromRow)]
struct TracerouteRow {
    id: String,
    session_id: Option<String>,
    target: String,
    resolved_ip: Option<String>,
    hop_count: i64,
    reached: bool,
    duration_ms: i64,
    protocol: String,
    created_at: String,
    hops_data: String,
}

impl From<TracerouteRow> for StoredTracerouteResult {
    fn from(row: TracerouteRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or(Uuid::nil()),
            session_id: row.session_id.and_then(|s| Uuid::parse_str(&s).ok()),
            target: row.target,
            resolved_ip: row.resolved_ip,
            hop_count: row.hop_count as u32,
            reached: row.reached,
            duration_ms: row.duration_ms as u64,
            protocol: row.protocol,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .map(|t| t.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            hops_data: serde_json::from_str(&row.hops_data).unwrap_or(serde_json::Value::Null),
        }
    }
}
