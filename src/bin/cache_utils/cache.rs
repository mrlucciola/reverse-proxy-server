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
/// Response<Vec< u8>>
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
    guard: RwLockReadGuard<'a, Cache>,
}
/// Instance of a write lock for the cache
pub struct CacheWriteLock<'a> {
    guard: RwLockWriteGuard<'a, Cache>,
}
impl<'a> CacheReadLock<'a> {
    /// Get entry from the hashmap (cache)
    pub fn get(&self, key: &String) -> Option<&Mutex<MapValue>> {
        self.guard.get(key).map(|s| s.clone())
    }
}
impl<'a> CacheWriteLock<'a> {
    pub fn insert(&self, key: &String, entry: MapValue) -> &mut Mutex<http::Response<Vec<u8>>> {
        // map.entry(key).or_insert_with(|| Mutex::new(item));
        self.guard
            .entry(key.to_string())
            .or_insert_with(|| Mutex::new(entry))
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
    pub fn clone(&self) -> Self {
        self.clone()
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

    pub fn get_cached_response(&self, req: &Request<Vec<u8>>) -> Option<MapValue> {
        // get url (key) from the incoming request
        let query_key = String::from_utf8(req.body().to_vec()).unwrap();

        // drops at fxn end
        let lock = self.to_owned().lock_read();

        // check if value exists
        match lock.get(&query_key) {
            Some(entry) => Some(entry.into_inner().expect("Poisoned cache")),
            None => None,
        }
    }
    pub fn add_entry_to_cache(
        &self,
        req: &Request<Vec<u8>>,
        entry: MapValue,
    ) -> Result<&mut Mutex<Response<Vec<u8>>>> {
        // get url (key) from the incoming request
        let query_key = String::from_utf8(req.body().to_vec()).unwrap();
        // get the write lock
        let lock = self.lock_write();
        // add to the cache
        let entry_mutex = lock.insert(&query_key, entry);

        Ok(entry_mutex)
    }

    // pub fn remove(&self, key: &String) {
    //     // let write_lock = self.0.write().unwrap();
    //     // let map = *write_lock;
    //     // map.remove(key);

    //     // drop(write_lock);
    // }

    // self.store.lock().unwrap().get(key).unwrap().clone()
    // pub fn read(&self) -> RwLockReadGuard<'_, HashMap<String, String>> {
    //     self.a.read().unwrap()
    // }

    /// Retrieve value from hashmap, given a URL key
    /// Keeping this here in case we use it for another method
    // fn get(&self, key: &String) -> Option<&Mutex<MapValue>> {
    //     // get lock
    //     let map = self.0.read().expect("Poisoned");

    //     // return map.get(key).map(|s| s.clone());
    // }

    /// Retrieve value from hashmap (cache), given a URL key
    /// Wrapper for the `get` method, in case we need to add extra use-case-specific logic
    // fn get_cached_response_old(&self, req: &Request<Vec<u8>>) -> Option<&Mutex<MapValue>> {
    //     // get url (key) from the incoming request
    //     let query_key = String::from_utf8(req.body().to_vec()).unwrap();

    //     // let xxx = self;
    //     // let map = self.(&query_key);

    //     // return none if !exist, else return `result_from_cache`
    //     // match self.get(&query_key) {
    //     //     Some(entry) => Some(entry),
    //     //     None => None,
    //     // }
    // }

    /// Add an entry to the cache
    /// update if already exists
    pub fn insert(&self, key: String, item: MapValue) {
        // write lock on the inner mutex, rather than the outer rwlock
        let map = self.0.write().expect("ErrorInsert: Poisoned mutex");

        map.entry(key).or_insert_with(|| Mutex::new(item));
    }

    /// Wrapper - add an entry to the cache (hashmap)
    /// TODO: add validation to key/url in caller and pass in url instead
    pub fn add_entry(&self, req: &Request<Vec<u8>>, res: MapValue) -> Result<()> {
        let url_key = String::from_utf8(req.body().to_vec())?;

        self.insert(url_key, res);

        Ok(())
    }
}
