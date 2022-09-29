// imports
use http::{Request, Response};
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
/// Arc<RwLock<HashMap<String, Mutex<Response<\Vec<u8\> \>\>\>\>\>
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
    /// Insert an entry into the cache
    pub fn insert(
        lock_guard: &'a mut RwLockWriteGuard<HashMap<String, Mutex<Response<Vec<u8>>>>>,
        key: String,
        entry: Response<Vec<u8>>,
    ) -> &'a mut Mutex<Response<Vec<u8>>> {
        let map_entry = lock_guard.entry(key);
        let inserted_value = map_entry.or_insert_with(|| Mutex::new(entry));

        inserted_value
    }

    /// Simple wrapper for Self::insert
    pub fn insert_req(
        lock: &'a mut CacheWriteLock,
        req: Request<Vec<u8>>,
        entry: Response<Vec<u8>>,
    ) -> &'a mut Mutex<Response<Vec<u8>>> {
        let key = String::from_utf8(req.body().to_vec()).unwrap().clone();

        // insert and return
        Self::insert(&mut lock.guard, key, entry)
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
    /// Simple wrapper for clone
    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
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
}
