//! Session repository.

use crate::error::StorageResult;
use crate::models::{DiagnosticSession, QueryOptions, SessionStatus, SessionType};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Repository for diagnostic sessions.
#[derive(Clone)]
pub struct SessionRepository {
    pool: SqlitePool,
}

impl SessionRepository {
    /// Create a new session repository.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new diagnostic session.
    pub async fn create(&self, session_type: SessionType) -> StorageResult<DiagnosticSession> {
        let session = DiagnosticSession {
            id: Uuid::new_v4(),
            started_at: Utc::now(),
            ended_at: None,
            session_type,
            status: SessionStatus::Running,
            summary: None,
            metadata: None,
        };

        sqlx::query(
            r#"
            INSERT INTO diagnostic_sessions (id, started_at, ended_at, session_type, status, summary, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(session.id.to_string())
        .bind(session.started_at.to_rfc3339())
        .bind(session.ended_at.map(|t| t.to_rfc3339()))
        .bind(session.session_type.as_str())
        .bind(session.status.as_str())
        .bind(&session.summary)
        .bind(session.metadata.as_ref().map(|m| m.to_string()))
        .execute(&self.pool)
        .await?;

        Ok(session)
    }

    /// Get a session by ID.
    pub async fn get(&self, id: Uuid) -> StorageResult<Option<DiagnosticSession>> {
        let row: Option<(
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            r#"
                SELECT id, started_at, ended_at, session_type, status, summary, metadata
                FROM diagnostic_sessions
                WHERE id = ?
                "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((id, started_at, ended_at, session_type, status, summary, metadata)) => {
                Ok(Some(DiagnosticSession {
                    id: Uuid::parse_str(&id).unwrap_or(Uuid::nil()),
                    started_at: chrono::DateTime::parse_from_rfc3339(&started_at)
                        .map(|t| t.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    ended_at: ended_at.and_then(|t| {
                        chrono::DateTime::parse_from_rfc3339(&t)
                            .map(|t| t.with_timezone(&Utc))
                            .ok()
                    }),
                    session_type: session_type.parse().unwrap_or(SessionType::Manual),
                    status: status.parse().unwrap_or(SessionStatus::Running),
                    summary,
                    metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
                }))
            }
            None => Ok(None),
        }
    }

    /// Complete a session.
    pub async fn complete(
        &self,
        id: Uuid,
        status: SessionStatus,
        summary: Option<String>,
    ) -> StorageResult<()> {
        sqlx::query(
            r#"
            UPDATE diagnostic_sessions
            SET ended_at = ?, status = ?, summary = ?
            WHERE id = ?
            "#,
        )
        .bind(Utc::now().to_rfc3339())
        .bind(status.as_str())
        .bind(summary)
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List sessions with options.
    pub async fn list(&self, options: &QueryOptions) -> StorageResult<Vec<DiagnosticSession>> {
        let limit = options.limit.unwrap_or(100);
        let offset = options.offset.unwrap_or(0);

        let rows: Vec<(
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
            Option<String>,
        )> = sqlx::query_as(
            r#"
                SELECT id, started_at, ended_at, session_type, status, summary, metadata
                FROM diagnostic_sessions
                ORDER BY started_at DESC
                LIMIT ? OFFSET ?
                "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let sessions: Vec<DiagnosticSession> = rows
            .into_iter()
            .filter_map(
                |(id, started_at, ended_at, session_type, status, summary, metadata)| {
                    Some(DiagnosticSession {
                        id: Uuid::parse_str(&id).ok()?,
                        started_at: chrono::DateTime::parse_from_rfc3339(&started_at)
                            .map(|t| t.with_timezone(&Utc))
                            .ok()?,
                        ended_at: ended_at.and_then(|t| {
                            chrono::DateTime::parse_from_rfc3339(&t)
                                .map(|t| t.with_timezone(&Utc))
                                .ok()
                        }),
                        session_type: session_type.parse().ok()?,
                        status: status.parse().ok()?,
                        summary,
                        metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
                    })
                },
            )
            .collect();

        Ok(sessions)
    }

    /// Delete a session and all related data.
    pub async fn delete(&self, id: Uuid) -> StorageResult<()> {
        let id_str = id.to_string();

        // Delete related data first
        sqlx::query("DELETE FROM ping_results WHERE session_id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM traceroute_results WHERE session_id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM speed_results WHERE session_id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM interface_snapshots WHERE session_id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM dns_results WHERE session_id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        // Delete session
        sqlx::query("DELETE FROM diagnostic_sessions WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
