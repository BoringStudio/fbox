pub use std::collections::HashMap;
pub use std::sync::Arc;

pub use anyhow::Result;
pub use futures::future::BoxFuture;
pub use tokio::sync::Mutex;
pub use tokio::sync::RwLock;

pub use crate::settings::Settings;

pub type ArcMutex<T> = Arc<Mutex<T>>;
pub type ArcRwLock<T> = Arc<RwLock<T>>;
