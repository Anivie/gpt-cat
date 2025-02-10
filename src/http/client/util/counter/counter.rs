use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU8};

use rand::Rng;

use crate::http::client::util::counter::SLEEP_RANGE;

/// The counter for concurrency control
pub(in crate::http) struct Counter {
    counter: AtomicU8,
    locked: AtomicBool,
    active: AtomicBool,
}

impl Default for Counter {
    fn default() -> Self {
        Self {
            counter: AtomicU8::new(0),
            locked: AtomicBool::new(false),
            active: AtomicBool::new(true),
        }
    }
}

impl Counter {
    pub(in crate::http) fn lock(&self) {
        let random = rand::rng().random_range(SLEEP_RANGE);
        self.counter.fetch_add(random, Relaxed);

        self.locked.store(true, Relaxed);
        self.active.store(false, Relaxed);
    }

    pub(in crate::http) fn unlock(&self) {
        self.locked.store(false, Relaxed);
    }

    pub(in crate::http) fn next_tick(&self) -> bool {
        !self.locked.load(Relaxed) && self.counter.load(Relaxed) > 0
    }

    pub(in crate::http) fn is_active(&self) -> bool {
        self.active.load(Relaxed) && !self.locked.load(Relaxed)
    }

    pub(in crate::http) fn tick(&self) {
        self.counter.fetch_sub(1, Relaxed);
        if self.counter.load(Relaxed) == 0 {
            self.active.store(true, Relaxed);
        }
    }
}
