use crate::core::traits::ArcRwLock;
use std::sync::{Arc, RwLock};

/// Shared state container which consists of `Arc` and `RwLock`,
/// and implements the `ArcRwLock` trait. This allows for many
/// shared read-only references to the state, and only one mutable
/// reference to the state.
pub struct SharedState<T>(Arc<RwLock<T>>);

impl<T> Clone for SharedState<T> {
    fn clone(&self) -> Self {
        SharedState(self.0.clone())
    }
}

impl<T> ArcRwLock<T> for SharedState<T>
where
    T: Send + Sync,
{
    fn new(value: T) -> Self {
        SharedState(Arc::new(RwLock::new(value)))
    }

    fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.0.read().unwrap())
    }

    fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.0.write().unwrap())
    }
}
