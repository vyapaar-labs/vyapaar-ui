//! In-memory, WASM-safe subscription stream.
//!
//! A push-based broadcast: [`EventStream::publish`] appends to a log and
//! notifies every live subscriber in order. There is no `tokio`, no OS
//! thread, and no blocking receive — exactly what the WASM Strategy Studio
//! needs, and a deterministic enough seam for the TUI and headless CLI.

use std::cell::RefCell;
use std::rc::Rc;

use crate::events::ClientEvent;

/// A subscriber handle. Dropping it unsubscribes the associated receiver.
#[derive(Debug)]
pub struct Subscription {
    stream: Rc<RefCell<StreamInner>>,
    id: u64,
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.stream
            .borrow_mut()
            .subscribers
            .retain(|s| s.id != self.id);
    }
}

impl Subscription {
    /// Read events published since the last call to this method (or since
    /// subscription, on the first call).
    pub fn drain(&self) -> Vec<ClientEvent> {
        self.stream.borrow_mut().drain(self.id)
    }

    /// The number of events not yet drained by this subscriber.
    pub fn pending(&self) -> usize {
        self.stream.borrow().pending(self.id)
    }
}

#[derive(Debug, Default)]
struct StreamInner {
    /// Monotonic event counter (used to compute per-subscriber deltas).
    counter: u64,
    /// Every event ever published to this stream.
    log: Vec<(u64, ClientEvent)>,
    /// Live subscribers.
    subscribers: Vec<Subscriber>,
}

#[derive(Debug)]
struct Subscriber {
    id: u64,
    /// Last event counter this subscriber has seen.
    seen: u64,
}

impl StreamInner {
    fn pending(&self, id: u64) -> usize {
        let seen = self
            .subscribers
            .iter()
            .find(|s| s.id == id)
            .map_or(self.counter, |s| s.seen);
        self.log.iter().filter(|(c, _)| *c > seen).count()
    }

    fn drain(&mut self, id: u64) -> Vec<ClientEvent> {
        let Some(sub) = self.subscribers.iter_mut().find(|s| s.id == id) else {
            return Vec::new();
        };
        let seen = sub.seen;
        let events = self
            .log
            .iter()
            .filter(|(c, _)| *c > seen)
            .map(|(_, e)| e.clone())
            .collect::<Vec<_>>();
        sub.seen = self.counter;
        events
    }
}

/// A broadcast event stream.
#[derive(Debug, Default)]
pub struct EventStream {
    inner: Rc<RefCell<StreamInner>>,
    next_id: u64,
}

impl EventStream {
    /// Create a new empty stream.
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to the stream. Delivery order matches publish order.
    pub fn subscribe(&mut self) -> Subscription {
        let id = self.next_id;
        self.next_id += 1;
        let seen = self.counter();
        self.inner
            .borrow_mut()
            .subscribers
            .push(Subscriber { id, seen });
        Subscription {
            stream: Rc::clone(&self.inner),
            id,
        }
    }

    /// Publish an event to all subscribers and append to the log.
    pub fn publish(&mut self, event: ClientEvent) {
        let mut inner = self.inner.borrow_mut();
        inner.counter += 1;
        let counter = inner.counter;
        inner.log.push((counter, event));
        // Bound the log growth: keep the last 4096 events.
        if inner.log.len() > 4096 {
            let excess = inner.log.len() - 4096;
            inner.log.drain(..excess);
        }
    }

    /// Total number of events ever published to this stream.
    pub fn published(&self) -> u64 {
        self.counter()
    }

    /// Number of live subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.inner.borrow().subscribers.len()
    }

    fn counter(&self) -> u64 {
        self.inner.borrow().counter
    }
}
