//! The Rust blocks in `CRATES.md`, each inside the scaffolding it needs to compile.
//!
//! 1. Each block is a verbatim excerpt of this file. `evals/snippets.py` checks that.
//! 2. The items around a block are stubs: the smallest types and functions that let it compile
//!    and pass the check in `LINTS.md`.
//! 3. `#[rustfmt::skip]` keeps a block's one-line forms as `CRATES.md` prints them.
//! 4. `derive_more::Display` and `strum::Display` share a name, so each lives in its own module.
#![expect(
    clippy::missing_errors_doc,
    reason = "the excerpts omit doc comments; CRATES.md's prose carries the rule beside each one"
)]
#![expect(
    unreachable_pub,
    reason = "dynosaur emits a pub helper trait inside a private module; the lint reads macro output"
)]

use std::io;
use std::time::Duration;

use tokio::signal;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

/// The `derive_more` excerpt: an unconstrained newtype, so `From` inbound is valid.
pub mod ids {
    use derive_more::{Display, From, Into};

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] // std
    #[derive(Display, From, Into)] // derive_more
    pub struct JobId(u64);
}

/// The `strum` excerpt: a unit enum with its variant table and parser derived.
pub mod priority {
    use strum::{Display, EnumCount, EnumString, VariantArray};

    #[rustfmt::skip]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)] // std
    #[derive(VariantArray, EnumCount, EnumString, Display)] // strum
    #[strum(serialize_all = "lowercase")]
    pub enum Priority { Low, Normal, High }
    // Priority::VARIANTS, Priority::COUNT, "high".parse::<Priority>()
}

/// A worker the pool runs.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Worker(pub u32);

/// Runs one worker until its token is cancelled.
async fn run(worker: Worker, cancel: CancellationToken) {
    let Worker(_) = worker;
    cancel.cancelled().await;
}

/// The `tokio-util` excerpt: one token and one tracker for a set of workers.
#[rustfmt::skip]
pub async fn run_pool(workers: Vec<Worker>) -> io::Result<()> {
    let token = CancellationToken::new();
    let tracker = TaskTracker::new();
    for worker in workers { tracker.spawn(run(worker, token.child_token())); }
    signal::ctrl_c().await?;
    token.cancel();
    tracker.close();
    tracker.wait().await;
    Ok(())
}

/// A key in the store.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Key(pub String);

/// Why a put failed.
#[derive(Debug, thiserror::Error, Copy, Clone, PartialEq, Eq)]
#[error("the store is full")]
pub struct StoreError;

#[trait_variant::make(Send)]
#[dynosaur::dynosaur(DynStore = dyn(box) Store)]
pub trait Store {
    async fn put(&self, key: Key, value: Vec<u8>) -> Result<(), StoreError>;
}

/// A store that counts puts, to show the trait is implementable with a native `async fn`.
#[derive(Debug, Default)]
pub struct CountingStore {
    puts: std::sync::atomic::AtomicUsize,
}

impl CountingStore {
    /// How many puts the store accepted.
    #[must_use]
    pub fn puts(&self) -> usize {
        self.puts.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Store for CountingStore {
    async fn put(&self, _key: Key, _value: Vec<u8>) -> Result<(), StoreError> {
        self.puts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

/// Puts through the trait object, the dynamic path that boxes the returned future.
pub async fn put_boxed(store: &DynStore<'_>, key: Key, value: Vec<u8>) -> Result<(), StoreError> {
    store.put(key, value).await
}

/// A TCP port.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Port(pub u16);

/// The `bon` excerpt's config struct: one required member and one with a default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    port: Port,
    timeout: Duration,
}

#[bon::bon]
impl Server {
    #[builder]
    pub fn new(port: Port, #[builder(default = Duration::from_secs(5))] timeout: Duration) -> Self {
        Self { port, timeout }
    }
}

impl Server {
    /// The bound port.
    #[must_use]
    pub const fn port(&self) -> Port {
        self.port
    }

    /// The request timeout.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use strum::{EnumCount, VariantArray};

    use super::ids::JobId;
    use super::priority::Priority;
    use super::*;

    #[rstest]
    #[case::empty("", None)]
    #[case::high("high", Some(Priority::High))]
    fn test_parses_priority(#[case] raw: &str, #[case] expected: Option<Priority>) {
        assert_eq!(raw.parse::<Priority>().ok(), expected);
    }

    #[test]
    fn test_strum_derives_the_table_and_the_count() {
        assert_eq!(
            Priority::VARIANTS,
            &[Priority::Low, Priority::Normal, Priority::High]
        );
        assert_eq!(Priority::COUNT, 3);
        assert_eq!(Priority::High.to_string(), "high");
    }

    #[test]
    fn test_derive_more_gives_display_and_both_conversions() {
        let id = JobId::from(7);
        assert_eq!(id.to_string(), "7");
        assert_eq!(u64::from(id), 7);
    }

    #[tokio::test]
    async fn test_store_works_statically_and_through_the_trait_object() -> anyhow::Result<()> {
        let store = CountingStore::default();
        store.put(Key("a".to_owned()), vec![1]).await?;
        let boxed: Box<DynStore<'_>> = DynStore::new_box(&store);
        put_boxed(&boxed, Key("b".to_owned()), vec![2]).await?;
        assert_eq!(store.puts(), 2);
        Ok(())
    }

    #[test]
    fn test_bon_builder_applies_the_default_timeout() {
        let server = Server::builder().port(Port(8080)).build();
        assert_eq!(server.port(), Port(8080));
        assert_eq!(server.timeout(), Duration::from_secs(5));
    }
}
