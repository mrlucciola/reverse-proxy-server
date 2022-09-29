// imports
use http::Response;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
// local
pub use crate::http_utils::{constants::*, errors::Result};

/// Bytes array
pub type ResBody = Vec<u8>;
pub type MapValue = Response<Vec<u8>>;
pub type Cache = HashMap<String, Mutex<MapValue>>;

#[derive(Debug, Default)]
/// An instance of a thread-safe cache for the proxy server.
///
/// type is:
/// HTTPCache = Arc<RwLock<Cache>>\
/// Cache = HashMap<String, Mutex<MapValue>>\
/// MapValue = Response<Vec<u8>>
///
/// Arc<RwLock<HashMap<String, Mutex<Response< Vec<u8\> \>\>\>\>\>
pub struct HTTPCache(Arc<RwLock<Cache>>);
/// Instance of read lock for the cache
pub struct CacheReadLock<'a> {
    pub guard: RwLockReadGuard<'a, Cache>,
}
/// Instance of a write lock for the cache
pub struct CacheWriteLock<'a> {
    pub guard: RwLockWriteGuard<'a, Cache>,
}
impl<'a> CacheReadLock<'a> {
    /// Get entry from the hashmap (cache)
    pub fn get(&self, key: &String) -> Option<&Mutex<MapValue>> {
        self.guard.get(key).map(|s| s.clone())
    }
}
impl<'a> CacheWriteLock<'a> {
    pub fn insert(
        &'a mut self,
        key: &'a String,
        entry: MapValue,
    ) -> &Mutex<http::Response<Vec<u8>>> {
        let xxx = self
            .guard
            .entry(key.to_string())
            .or_insert_with(|| Mutex::new(entry));
        xxx
    }
}

impl HTTPCache {
    /// Create a new instance of HTTPCache
    /// type Arc<RwLock<HashMap<String, Mutex<Response<Vec<u8\>\>\>\>\>\>
    pub fn new() -> Self {
        let new_instance = Cache::new();
        let new_instance = RwLock::new(new_instance);
        let new_instance: Arc<RwLock<HashMap<String, Mutex<Response<Vec<u8>>>>>> =
            Arc::new(new_instance);

        Self(new_instance)
    }
    // pub fn clone(&self) -> Self {
    //     self.clone()
    // }
    /// Initialize the lock for writing
    pub fn lock_write(&self) -> CacheWriteLock {
        CacheWriteLock {
            guard: self.0.write().expect("Poisoned write lock (RwLock)"),
        }
    }
    /// Initialize the lock for reading
    pub fn lock_read(&self) -> CacheReadLock {
        CacheReadLock {
            guard: self.0.read().expect("Poisoned read lock (RwLock)"),
        }
    }

    // Wrapper - add an entry to the cache (hashmap)
    // TODO: add validation to key/url in caller and pass in url instead
    // pub fn add_entry(&self, req: &Request<Vec<u8>>, res: MapValue) -> Result<()> {
    //     Ok(())
    // }
}
