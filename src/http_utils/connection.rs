// libs
use failure;
use http::Response;
use std::{
    net::TcpStream,
    process::exit,
    sync::{Arc, Mutex},
};
// local
use super::{
    constants::*,
    errors::*,
    formatting::get_origin_addr,
    request::{get_parsed_request, write_req_to_origin},
    response::{read_res_from_origin, write_response_to_client},
};
use crate::cache_utils::cache::{CacheWriteLock, HTTPCache};

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
    proxy_origin_stream: &mut TcpStream,
) -> Result<Response<Vec<u8>>> {
    // 2.b) write to origin
    // TODO: propagate error to client http response
    if let Err(err) = write_req_to_origin(proxy_origin_stream, &parsed_req) {
        return Err(fmt_error(
            RequestError::ConnectionError(err),
            "Error writing to origin:",
        ));
    };

    // 3) Read from origin
    let res_from_origin = read_res_from_origin(proxy_origin_stream).or_else(|err| Err(err))?;

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
pub fn handle_client_proxy_connection(
    mut client_proxy_connection: TcpStream,
    cache: Arc<HTTPCache>,
) -> Result<()> {
    ////////////////////////////////////////////
    // 1) parse http request

    // TODO: propagate error to client http response
    let parsed_req = get_parsed_request(&mut client_proxy_connection)?;

    // 1) parse http request
    ////////////////////////////////////////////

    let mut proxy_origin_stream = match TcpStream::connect(&get_origin_addr()) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Error: Please re-start the origin server {}", err);
            exit(1);
        }
    };

    ////////////////////////////////////////////
    // 2) check cache
    // return early if we have an entry in the cache
    let lock_r = cache.lock_read();
    let query_key = String::from_utf8(parsed_req.body().to_vec()).unwrap();

    match lock_r.get(&query_key) {
        Some(entry_mutex) => {
            let entry = entry_mutex.lock().expect("Err: getting res from mutex (2)");
            write_response_to_client(&mut client_proxy_connection, entry)?;
            return Ok(());
        }
        None => {
            drop(lock_r);
            // Insert
            let lock_w = cache.lock_write();
            fetch_new_response(parsed_req, &mut proxy_origin_stream, lock_w)?;

            return Ok(());
        }
    }
}

/// If the cache didnt return a value-
///     1) query the external source (fwd to origin first)
///     2) add to cache
///     3) send the http response with payload back to the client
fn fetch_new_response<'a>(
    parsed_client_req: http::Request<Vec<u8>>,
    proxy_origin_stream: &mut TcpStream,
    mut lock_w: CacheWriteLock,
) -> Result<()> {
    // TODO: propagate error to http response
    println!("cache miss... making request to origin... ");
    let res_from_origin =
        forward_request_and_return_response(&parsed_client_req, proxy_origin_stream)?;
    println!(
        "forwarded request, got response: {:?} ",
        String::from_utf8_lossy(res_from_origin.body())
    );

    // add to cache
    let query_key = String::from_utf8(parsed_client_req.body().to_vec())
        .unwrap()
        .clone();

    // TODO: either write to client after `insert` or return mutex
    new_insert(&mut lock_w, query_key, res_from_origin);
    // let res_mutex = new_insert(&mut lock_w, query_key, res_from_origin);

    Ok(())
}

fn new_insert(lock_w: &mut CacheWriteLock, key: String, entry: Response<Vec<u8>>) {
    lock_w.guard.entry(key).or_insert_with(|| Mutex::new(entry));
}
