use std::sync::{Arc, RwLock};

/// A shared state trait which consists of `Arc` and `RwLock`.
/// This allows for many shared read-only references to the state,
/// and only one mutable reference to the state.
pub trait ArcRwLock<T>: Clone + Send + Sync {
    /// Create a new shared state.
    fn new(value: T) -> Self;

    /// Read the shared state.
    fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R;

    /// Write the shared state.
    fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R;
}
