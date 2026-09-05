//! An in-memory session store.
//!
//! 1. A session opens with an expiry of `now + lease`.
//! 2. `expire_all` removes every session whose expiry is at or before `now`.
//! 3. Time is an input: every operation takes `now`, so a test passes a constant.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A store-generated session identifier.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct SessionId(u64);

impl SessionId {
    /// The raw identifier, for a log line or a wire message.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Store configuration.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// How long a session stays valid after it opens.
    pub lease: Duration,
}

/// A session known to the store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    id: SessionId,
    opened_at: Instant,
    expires_at: Instant,
}

impl Session {
    #[must_use]
    pub const fn id(&self) -> SessionId {
        self.id
    }

    #[must_use]
    pub const fn opened_at(&self) -> Instant {
        self.opened_at
    }

    #[must_use]
    pub const fn expires_at(&self) -> Instant {
        self.expires_at
    }

    /// A session is expired from the instant its expiry is reached.
    #[must_use]
    pub fn is_expired(&self, now: Instant) -> bool {
        now >= self.expires_at
    }
}

/// The store.
#[derive(Debug)]
pub struct Store {
    config: Config,
    sessions: HashMap<SessionId, Session>,
    next_id: u64,
}

impl Store {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
            next_id: 1,
        }
    }

    /// Opens a session that expires at `now + lease`.
    pub fn open(&mut self, now: Instant) -> SessionId {
        let id = SessionId(self.next_id);
        self.next_id += 1;
        let session = Session {
            id,
            opened_at: now,
            expires_at: now + self.config.lease,
        };
        self.sessions.insert(id, session);
        id
    }

    #[must_use]
    pub fn get(&self, id: SessionId) -> Option<&Session> {
        self.sessions.get(&id)
    }

    /// Removes every expired session and returns how many were removed.
    pub fn expire_all(&mut self, now: Instant) -> usize {
        let before = self.sessions.len();
        self.sessions.retain(|_, session| !session.is_expired(now));
        before - self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEASE: Duration = Duration::from_mins(1);

    fn store() -> Store {
        Store::new(Config { lease: LEASE })
    }

    #[test]
    fn test_open_sets_expiry_to_now_plus_lease() {
        let mut store = store();
        // t0 is an origin; every instant below is an offset from it.
        let t0 = Instant::now();
        let id = store.open(t0);
        let session = store.get(id).expect("the session was just opened");
        // expiry = t0 + 60s; one second before it the session is live
        assert_eq!(session.expires_at(), t0 + LEASE);
        assert!(!session.is_expired(t0 + Duration::from_secs(59)));
        assert!(session.is_expired(t0 + LEASE));
    }

    #[test]
    fn test_expire_all_removes_only_expired_sessions() {
        let mut store = store();
        let t0 = Instant::now();
        let early = store.open(t0);
        let late = store.open(t0 + Duration::from_secs(30));
        // at t0 + 60s: early (expires t0 + 60s) is expired, late (expires t0 + 90s) is not
        let removed = store.expire_all(t0 + LEASE);
        assert_eq!(removed, 1, "one of two sessions expired, removed {removed}");
        assert!(store.get(early).is_none());
        assert!(store.get(late).is_some());
    }
}
