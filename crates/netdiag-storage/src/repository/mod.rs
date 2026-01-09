//! Repository implementations for data access.

mod dns;
mod ping;
mod session;
mod traceroute;

pub use dns::DnsRepository;
pub use ping::PingRepository;
pub use session::SessionRepository;
pub use traceroute::TracerouteRepository;

use crate::database::Database;

/// Combined repository for all data access.
pub struct Repository {
    /// Ping results repository
    pub ping: PingRepository,
    /// Session repository
    pub session: SessionRepository,
    /// Traceroute results repository
    pub traceroute: TracerouteRepository,
    /// DNS results repository
    pub dns: DnsRepository,
}

impl Repository {
    /// Create a new repository with the given database.
    pub fn new(db: &Database) -> Self {
        Self {
            ping: PingRepository::new(db.pool().clone()),
            session: SessionRepository::new(db.pool().clone()),
            traceroute: TracerouteRepository::new(db.pool().clone()),
            dns: DnsRepository::new(db.pool().clone()),
        }
    }
}
