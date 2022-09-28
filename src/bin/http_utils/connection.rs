// libs
use failure;
use http::Response;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    process::exit,
    sync::Mutex, borrow::BorrowMut,
};
// local
use crate::cache_utils::cache::HTTPCache;

pub use super::constants::*;
use super::errors::*;
use super::request::{get_parsed_request, write_req_to_origin};
use super::response::{read_res_from_origin, write_response_to_client};

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

pub fn new_endpoint_str(addr: &str, port: u16) -> String {
    let addr_parsed = IpAddr::V4(addr.parse::<Ipv4Addr>().unwrap());
    let endpoint = SocketAddr::new(addr_parsed, port);

    endpoint.to_string()
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
        )); // prev- RequestError::ConnectionError(err)
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
    client_proxy_connection: &mut TcpStream,
    origin_endpoint: String,
    cache: HTTPCache,
    // cache: Arc<RwLock<HTTPCache>>,
) -> Result<()> {
    ////////////////////////////////////////////
    // 1) parse http request

    // TODO: propagate error to client http response
    let parsed_req = get_parsed_request(client_proxy_connection)?;

    // 1) parse http request
    ////////////////////////////////////////////

    ////////////////////////////////////////////

    ////////////////////////////////////////////
    // check cache

    // TODO: Propagate error to client http response
    let res_from_origin = check_cache(
        // TODO: do we need ref?
        &cache,
        parsed_req,
        origin_endpoint.clone(),
    )?;

    // check cache
    ////////////////////////////////////////////

    ////////////////////////////////////////////
    // 2) Write to origin

    // 4) respond to client
    // TODO: propagate error to client http response
    // TODO: log timestamps for client, proxy, & origin requests and responses
    write_response_to_client(client_proxy_connection, res_from_origin)?;

    Ok(())
}

/// We need to return mutex because mutex is being stored in cache
/// TODO: consider - insert new entries to cache AFTER writing to client
fn check_cache(
    cache: &HTTPCache, // prev: &Arc<RwLock<HTTPCache>>,
    // cache_map: RwLockReadGuard<HTTPCache>,
    parsed_client_req: http::Request<Vec<u8>>,
    origin_endpoint: String,
) -> Result<&mut http::Response<Vec<u8>>> {
    // ) -> Result<&mut Mutex<http::Response<Vec<u8>>>> {
    println!("7.1) opening stream: {}", true);
    let mut proxy_origin_stream = match TcpStream::connect(origin_endpoint) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Error: Please re-start the origin server {}", err);
            exit(1);
        }
    };

    // return early if we have a `live` response in the cache
    println!("7.2) checking/getting cached response: {}", true);
    if let Some(mut cached_res) = cache.get_cached_response(&parsed_client_req) {
        println!("7.2.a) success: got cached response");
        return Ok(cached_res.borrow_mut());
        // return Ok(&mut Mutex::new(cached_res));
    };

    // TODO: propagate error to http response
    println!("7.3) cache miss... making request to origin... ");
    let res_from_origin =
        forward_request_and_return_response(&parsed_client_req, &mut proxy_origin_stream)?;
    println!(
        "7.4) forwarded request, got response: {:?} ",
        String::from_utf8_lossy(res_from_origin.body())
    );

    println!(
        "7.5) start: add entry to cache: res status: {}",
        res_from_origin.status().as_u16()
    );
    // add to cache
    let lock = cache.lock_write();
    let query_key = String::from_utf8(parsed_client_req.body().to_vec()).unwrap();
    let res_from_origin_mutex = lock.insert(&query_key, res_from_origin);
    let res_orig = res_from_origin_mutex.get_mut().expect("Poisoned");
    // drop(lock);
    // let res_from_origin_mutex = cache.add_entry_to_cache(&parsed_client_req, res_from_origin)?;
    // let new_res = res_from_origin_mutex.into_inner().expect("poisoned");
    println!("7.7) complete: add entry to cache: {}", true);
    // println!("7.8) dereferencing res mutex");
    // let res_from_origin = res_from_origin_mutex.into_inner()?;
    println!("7.9) complete: dereferenced response mutex");
    Ok(res_orig)
}
