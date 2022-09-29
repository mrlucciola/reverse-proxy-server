// imports
use std::sync::Arc;
// local
use super::cache::{HTTPCache, CACHE_TTL_SEC};

/// 1) Iterate through all entries in the cache HashMap
/// 1) Read timestamp value on the request (each entry is a request, key is URL)
/// 1) If greater than 30 seconds, delete entry from cache
/// 1) If at cache limit, remove oldest entry, (and insert current response in its place)
pub fn purge_expired_cache_entries(cache: Arc<HTTPCache>) {
    println!("\nPurging cache: ");
    let mut map_reader = cache.lock_write().guard;
    let init_map_size = map_reader.len();

    map_reader.retain(|_, entry_mutex| {
        // get a read lock for the entry
        let entry = entry_mutex
            .lock()
            .expect("Poisoned mutex: checking for outdated entries.");

        for (header_name, header_value) in entry.headers() {
            if header_name != "date" {
                continue;
            }
            if let Ok(timestamp) =
                chrono::DateTime::parse_from_rfc2822(header_value.to_str().unwrap())
            {
                let dt_now = chrono::Utc::now().timestamp();
                let dt_response = timestamp.timestamp();
                let diff = dt_now - dt_response;
                return diff < CACHE_TTL_SEC;
            };
        }
        true
    });

    println!("new map size {} - init: {init_map_size}", map_reader.len())
}
