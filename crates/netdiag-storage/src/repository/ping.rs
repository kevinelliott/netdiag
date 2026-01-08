//! Ping results repository.

use crate::error::StorageResult;
use crate::models::{QueryOptions, StoredPingResult};
use chrono::Utc;
use netdiag_types::diagnostics::PingStats;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Repository for ping results.
#[derive(Clone)]
pub struct PingRepository {
    pool: SqlitePool,
}

impl PingRepository {
    /// Create a new ping repository.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Save ping stats.
    pub async fn save(
        &self,
        stats: &PingStats,
        session_id: Option<Uuid>,
    ) -> StorageResult<StoredPingResult> {
        let result = StoredPingResult {
            id: Uuid::new_v4(),
            session_id,
            target: stats.target.to_string(),
            resolved_ip: Some(stats.target.to_string()),
            transmitted: stats.transmitted,
            received: stats.received,
            loss_percent: stats.loss_percent,
            min_rtt_ms: stats.min_rtt.map(|d| d.as_secs_f64() * 1000.0),
            avg_rtt_ms: stats.avg_rtt.map(|d| d.as_secs_f64() * 1000.0),
            max_rtt_ms: stats.max_rtt.map(|d| d.as_secs_f64() * 1000.0),
            stddev_ms: stats.stddev_rtt.map(|d| d.as_secs_f64() * 1000.0),
            quality: Some(format!("{:?}", stats.quality_rating())),
            created_at: Utc::now(),
            raw_data: serde_json::to_value(stats).ok(),
        };

        sqlx::query(
            r#"
            INSERT INTO ping_results (
                id, session_id, target, resolved_ip, transmitted, received,
                loss_percent, min_rtt_ms, avg_rtt_ms, max_rtt_ms, stddev_ms,
                quality, created_at, raw_data
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(result.id.to_string())
        .bind(result.session_id.map(|id| id.to_string()))
        .bind(&result.target)
        .bind(&result.resolved_ip)
        .bind(result.transmitted as i64)
        .bind(result.received as i64)
        .bind(result.loss_percent)
        .bind(result.min_rtt_ms)
        .bind(result.avg_rtt_ms)
        .bind(result.max_rtt_ms)
        .bind(result.stddev_ms)
        .bind(&result.quality)
        .bind(result.created_at.to_rfc3339())
        .bind(result.raw_data.as_ref().map(|v| v.to_string()))
        .execute(&self.pool)
        .await?;

        Ok(result)
    }

    /// Get ping result by ID.
    pub async fn get(&self, id: Uuid) -> StorageResult<Option<StoredPingResult>> {
        let row: Option<PingRow> = sqlx::query_as(
            r#"
            SELECT id, session_id, target, resolved_ip, transmitted, received,
                   loss_percent, min_rtt_ms, avg_rtt_ms, max_rtt_ms, stddev_ms,
                   quality, created_at, raw_data
            FROM ping_results
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    /// List ping results with options.
    pub async fn list(&self, options: &QueryOptions) -> StorageResult<Vec<StoredPingResult>> {
        let limit = options.limit.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);

        let mut query = String::from(
            r#"
            SELECT id, session_id, target, resolved_ip, transmitted, received,
                   loss_percent, min_rtt_ms, avg_rtt_ms, max_rtt_ms, stddev_ms,
                   quality, created_at, raw_data
            FROM ping_results
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

        let mut q = sqlx::query_as::<_, PingRow>(&query);

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

        let rows: Vec<PingRow> = q.fetch_all(&self.pool).await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Get average stats for a target.
    pub async fn get_average_stats(
        &self,
        target: &str,
        limit: i64,
    ) -> StorageResult<Option<PingAverages>> {
        let row: Option<(f64, f64, f64, f64, f64)> = sqlx::query_as(
            r#"
            SELECT
                AVG(avg_rtt_ms) as avg_rtt,
                AVG(min_rtt_ms) as avg_min,
                AVG(max_rtt_ms) as avg_max,
                AVG(loss_percent) as avg_loss,
                COUNT(*) as count
            FROM (
                SELECT avg_rtt_ms, min_rtt_ms, max_rtt_ms, loss_percent
                FROM ping_results
                WHERE target = ?
                ORDER BY created_at DESC
                LIMIT ?
            )
            "#,
        )
        .bind(target)
        .bind(limit)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(avg_rtt, avg_min, avg_max, avg_loss, count)| PingAverages {
            avg_rtt_ms: avg_rtt,
            avg_min_rtt_ms: avg_min,
            avg_max_rtt_ms: avg_max,
            avg_loss_percent: avg_loss,
            sample_count: count as u64,
        }))
    }

    /// Delete ping result.
    pub async fn delete(&self, id: Uuid) -> StorageResult<()> {
        sqlx::query("DELETE FROM ping_results WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

/// Average ping statistics.
#[derive(Debug, Clone)]
pub struct PingAverages {
    /// Average RTT in milliseconds
    pub avg_rtt_ms: f64,
    /// Average minimum RTT
    pub avg_min_rtt_ms: f64,
    /// Average maximum RTT
    pub avg_max_rtt_ms: f64,
    /// Average packet loss
    pub avg_loss_percent: f64,
    /// Number of samples
    pub sample_count: u64,
}

/// Internal row type for query.
#[derive(sqlx::FromRow)]
struct PingRow {
    id: String,
    session_id: Option<String>,
    target: String,
    resolved_ip: Option<String>,
    transmitted: i64,
    received: i64,
    loss_percent: f64,
    min_rtt_ms: Option<f64>,
    avg_rtt_ms: Option<f64>,
    max_rtt_ms: Option<f64>,
    stddev_ms: Option<f64>,
    quality: Option<String>,
    created_at: String,
    raw_data: Option<String>,
}

impl From<PingRow> for StoredPingResult {
    fn from(row: PingRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or(Uuid::nil()),
            session_id: row.session_id.and_then(|s| Uuid::parse_str(&s).ok()),
            target: row.target,
            resolved_ip: row.resolved_ip,
            transmitted: row.transmitted as u32,
            received: row.received as u32,
            loss_percent: row.loss_percent,
            min_rtt_ms: row.min_rtt_ms,
            avg_rtt_ms: row.avg_rtt_ms,
            max_rtt_ms: row.max_rtt_ms,
            stddev_ms: row.stddev_ms,
            quality: row.quality,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .map(|t| t.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            raw_data: row.raw_data.and_then(|s| serde_json::from_str(&s).ok()),
        }
    }
}
