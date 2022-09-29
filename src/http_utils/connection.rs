// libs
use failure;
use http::Response;
use std::{
    collections::HashMap,
    net::TcpStream,
    process::exit,
    sync::{Arc, Mutex, RwLockWriteGuard},
};
// local
use super::{
    constants::*,
    errors::*,
    formatting::get_origin_addr,
    request::{get_parsed_request, write_req_to_origin},
    response::{read_res_from_origin, write_response_to_client},
};
use crate::cache_utils::cache::HTTPCache;

pub fn check_body_len(header_map: &http::HeaderMap) -> Result<usize> {
    let header_value = header_map.get("content-length");
    if header_value.is_none() {
        return Ok(0);
    };

    let content_body_len = header_map
        .get("content-length")
        .unwrap()
        .to_str()
        .or(Err(failure::err_msg(format!(
            "{:?}",
            ConnectionError::InvalidContentLength
        ))))?
        .parse::<usize>()
        .or(Err(failure::err_msg(format!(
            "{:?}",
            ConnectionError::InvalidContentLength
        ))))?;

    if content_body_len > SIZE_MAX_BODY {
        return Err(failure::err_msg(format!(
            "{:?}",
            ConnectionError::BodySizeTooLarge
        )));
    }
    Ok(content_body_len)
}

/// Attempt to write to origin
pub fn forward_request_and_return_response(
    parsed_req: &http::Request<Vec<u8>>,
) -> Result<Response<Vec<u8>>> {
    let mut proxy_origin_stream = TcpStream::connect(&get_origin_addr())?;
    // 2.b) write to origin
    // TODO: propagate error to client http response
    if let Err(err) = write_req_to_origin(&mut proxy_origin_stream, &parsed_req) {
        return Err(fmt_error(
            RequestError::ConnectionError(err),
            "Error writing to origin:",
        ));
    };

    // 3) Read from origin
    let res_from_origin = read_res_from_origin(&mut proxy_origin_stream).or_else(|err| Err(err))?;

    // 3.a) check response, proceed if 200 error code
    if res_from_origin.status().as_u16() != 200 {
        return Err(failure::err_msg(format!(
            "Unsuccessful response: {}",
            res_from_origin.status().as_u16()
        )));
    }

    Ok(res_from_origin)
}

/// Handle the tcp connection between client and proxy
///
/// 1) forward request to origin
/// 2) receive response from origin
/// 3) write back to client
///
/// Handle error responses to client here
pub fn handle_client_proxy_connection<'a>(
    mut client_proxy_connection: TcpStream,
    cache: &'a Arc<HTTPCache>,
) -> Result<()> {
    ////////////////////////////////////////////
    // 1) parse http request

    // TODO: propagate error to client http response
    let parsed_req = get_parsed_request(&mut client_proxy_connection)?;

    // 1) parse http request
    ////////////////////////////////////////////

    ////////////////////////////////////////////
    // 2) check cache

    // return early if we have an entry in the cache
    let query_key = String::from_utf8(parsed_req.body().to_vec()).unwrap();

    let lock_r = cache.lock_read();
    match lock_r.get(&query_key) {
        Some(entry_mutex) => {
            // let entry = entry_mutex
            //     .lock()
            //     .expect("Poisoned mutex: after getting cache entry");
            write_response_to_client(&mut client_proxy_connection, entry_mutex)?;
            drop(lock_r);
        }
        None => {
            // If the cache didnt return a value-
            //     0) drop the read lock
            //     1) query the external source (fwd to origin first)
            //     2) add to cache
            //     3) send the http response with payload back to the client
            drop(lock_r);

            // TODO: propagate error to http response
            println!("cache miss... making request to origin... ");
            let res_from_origin = forward_request_and_return_response(&parsed_req)?;

            let mut lock_w = cache.lock_write();
            let entry_mutex = insert_entry_to_cache(parsed_req, res_from_origin, &mut lock_w.guard);

            // Insert
            // let mut lock_w = cache.lock_write();
            // let res_mutex = fetch_new_response(parsed_req, &mut lock_w.guard)?;

            // let entry = entry_mutex
            //     .lock()
            //     .expect("Poisoned mutex: after fetching response");

            write_response_to_client(&mut client_proxy_connection, entry_mutex)?;
        }
    };

    Ok(())
}

fn insert_entry_to_cache<'a>(
    parsed_client_req: http::Request<Vec<u8>>,
    res_from_origin: Response<Vec<u8>>,
    lock_guard: &'a mut RwLockWriteGuard<HashMap<String, Mutex<Response<Vec<u8>>>>>,
) -> &'a Mutex<Response<Vec<u8>>> {
    let query_key = String::from_utf8(parsed_client_req.body().to_vec())
        .unwrap()
        .clone();

    let entry_mutex = new_insert(lock_guard, query_key, res_from_origin);
    entry_mutex
}

// fn new_insert(lock_w: &mut CacheWriteLock, key: String, entry: Response<Vec<u8>>) -> &mut Mutex<Response<Vec<u8>>> {
// fn new_insert<'a>(lock_w: &'a mut CacheWriteLock<'a>, key: String, entry: Response<Vec<u8>>) -> &mut Mutex<Response<Vec<u8>>> {
fn new_insert<'a>(
    // lock_guard: &'a mut CacheWriteLock<'a>,
    lock_guard: &'a mut RwLockWriteGuard<HashMap<String, Mutex<Response<Vec<u8>>>>>,
    key: String,
    entry: Response<Vec<u8>>,
) -> &'a mut Mutex<Response<Vec<u8>>> {
    // ) -> &'a mut Response<Vec<u8>> {
    let map_entry = lock_guard.entry(key);
    let inserted_value = map_entry.or_insert_with(|| Mutex::new(entry));
    // let xxx = inserted_value.get_mut().unwrap();
    inserted_value
}
