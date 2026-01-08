//! DNS results repository.

use crate::error::StorageResult;
use crate::models::{QueryOptions, StoredDnsResult};
use chrono::Utc;
use netdiag_connectivity::DnsResult;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Repository for DNS results.
#[derive(Clone)]
pub struct DnsRepository {
    pool: SqlitePool,
}

impl DnsRepository {
    /// Create a new DNS repository.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Save DNS result.
    pub async fn save(
        &self,
        result: &DnsResult,
        session_id: Option<Uuid>,
    ) -> StorageResult<StoredDnsResult> {
        let stored = StoredDnsResult {
            id: Uuid::new_v4(),
            session_id,
            query: result.query.clone(),
            addresses: serde_json::to_value(
                &result
                    .addresses
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>(),
            )
            .unwrap_or(serde_json::Value::Null),
            duration_ms: result.duration.as_secs_f64() * 1000.0,
            success: result.success,
            error: result.error.clone(),
            created_at: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO dns_results (
                id, session_id, query, addresses, duration_ms, success, error, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(stored.id.to_string())
        .bind(stored.session_id.map(|id| id.to_string()))
        .bind(&stored.query)
        .bind(stored.addresses.to_string())
        .bind(stored.duration_ms)
        .bind(stored.success)
        .bind(&stored.error)
        .bind(stored.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(stored)
    }

    /// Get DNS result by ID.
    pub async fn get(&self, id: Uuid) -> StorageResult<Option<StoredDnsResult>> {
        let row: Option<DnsRow> = sqlx::query_as(
            r#"
            SELECT id, session_id, query, addresses, duration_ms, success, error, created_at
            FROM dns_results
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    /// List DNS results with options.
    pub async fn list(&self, options: &QueryOptions) -> StorageResult<Vec<StoredDnsResult>> {
        let limit = options.limit.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);

        let mut query = String::from(
            r#"
            SELECT id, session_id, query, addresses, duration_ms, success, error, created_at
            FROM dns_results
            WHERE 1=1
            "#,
        );

        if options.target.is_some() {
            query.push_str(" AND query = ?");
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

        let mut q = sqlx::query_as::<_, DnsRow>(&query);

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

        let rows: Vec<DnsRow> = q.fetch_all(&self.pool).await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Get average DNS resolution time for a query.
    pub async fn get_average_resolution_time(
        &self,
        query: &str,
        limit: i64,
    ) -> StorageResult<Option<f64>> {
        let row: Option<(f64,)> = sqlx::query_as(
            r#"
            SELECT AVG(duration_ms) as avg_duration
            FROM (
                SELECT duration_ms
                FROM dns_results
                WHERE query = ? AND success = 1
                ORDER BY created_at DESC
                LIMIT ?
            )
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(avg,)| avg))
    }

    /// Delete DNS result.
    pub async fn delete(&self, id: Uuid) -> StorageResult<()> {
        sqlx::query("DELETE FROM dns_results WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

/// Internal row type for query.
#[derive(sqlx::FromRow)]
struct DnsRow {
    id: String,
    session_id: Option<String>,
    query: String,
    addresses: String,
    duration_ms: f64,
    success: bool,
    error: Option<String>,
    created_at: String,
}

impl From<DnsRow> for StoredDnsResult {
    fn from(row: DnsRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or(Uuid::nil()),
            session_id: row.session_id.and_then(|s| Uuid::parse_str(&s).ok()),
            query: row.query,
            addresses: serde_json::from_str(&row.addresses).unwrap_or(serde_json::Value::Null),
            duration_ms: row.duration_ms,
            success: row.success,
            error: row.error,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .map(|t| t.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }
    }
}
