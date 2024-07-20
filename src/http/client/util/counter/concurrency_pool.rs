use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use rayon::prelude::*;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

use crate::data::config::endpoint::Endpoint;
use crate::data::config::runtime_data::AccountVisitor;
use crate::http::client::util::counter::counter::Counter;

/// The safe pool that use to manage the concurrency, we can promise that the concurrency
/// will in the range of the concurrency_count
pub struct SafePool<T> {
    counter: Arc<Vec<Counter>>,
    sender: Sender<()>,
    inner: T,
}

/// The safe object that use to lock the concurrency
/// When the object is dropped, the concurrency will be unlocked
pub struct SafeObject<'a, T> {
    inner: T,
    counter: &'a Counter,
}

impl<T> Deref for SafeObject<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Drop for SafeObject<'_, T> {
    fn drop(&mut self) {
        self.counter.unlock();
    }
}

impl<T> SafePool<T> {
    pub fn new(inner: T, concurrency_count: u32) -> Self {
        let mut counter = Vec::new();
        for _ in 0..concurrency_count {
            counter.push(Counter::default());
        }
        let counter = Arc::new(counter);

        let sender = {
            let counter = counter.clone();
            let (sender, mut receiver) =
                tokio::sync::mpsc::channel::<()>(concurrency_count as usize);

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(1));

                loop {
                    interval.tick().await;

                    let tmp = counter
                        .par_iter()
                        .filter(|&counter| !counter.is_active())
                        .map(|counter| {
                            if counter.next_tick() {
                                counter.tick();
                            }
                        })
                        .count() as u32;

                    if tmp == 0 {
                        receiver.recv().await;
                        interval = tokio::time::interval(Duration::from_secs(1));
                    } else {
                        loop {
                            if let Err(_) = receiver.try_recv() {
                                break;
                            }
                        }
                    }
                }
            });

            sender
        };

        SafePool {
            counter,
            sender,
            inner,
        }
    }
}

pub trait VecSafePool {
    type Inner;

    fn to_vec_safe_pool(self, concurrency_count: u32) -> Vec<SafePool<Self::Inner>>;
}

impl<T> VecSafePool for Vec<T> {
    type Inner = T;

    fn to_vec_safe_pool(self, concurrency_count: u32) -> Vec<SafePool<Self::Inner>> {
        let mut result = Vec::new();
        for x in self {
            result.push(SafePool::new(x, concurrency_count));
        }
        result
    }
}

pub trait VecGettable {
    type Output;

    async fn get_safe_object(&self) -> SafeObject<&Self::Output>;
}

impl<T> VecGettable for Vec<SafePool<T>> {
    type Output = T;

    async fn get_safe_object(&self) -> SafeObject<&Self::Output> {
        //添加偏置条件，防止在并发情况下，每次都是第一个对象被选中
        let preference = if self.len() == 1 {
            0
        } else {
            rand::thread_rng().gen_range(0..self.len())
        };

        loop {
            for (index, safe_pool) in self.iter().enumerate() {
                if index != preference {
                    continue;
                }

                for counter in safe_pool.counter.iter() {
                    if !counter.is_active() {
                        continue;
                    }

                    counter.lock();
                    safe_pool.sender.send(()).await.unwrap();

                    return SafeObject {
                        inner: &safe_pool.inner,
                        counter,
                    };
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    }
}

impl SafePool<AccountVisitor> {
    pub fn get_endpoint(&self) -> &Endpoint {
        &self.inner.endpoint
    }
}
