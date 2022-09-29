// libs
use http::{HeaderMap, Response};
use std::{
    collections::HashMap,
    io::Write,
    net::TcpStream,
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
        .or(Err(fmt_error(ConnectionError::InvalidContentLength, "")))?
        .parse::<usize>()
        .or(Err(fmt_error(ConnectionError::InvalidContentLength, "")))?;

    if content_body_len > SIZE_MAX_BODY {
        return Err(fmt_error(ConnectionError::BodySizeTooLarge, ""));
    }

    Ok(content_body_len)
}

/// Forward the client request to origin, and return response back to client
///
/// 1) Attempt to write to origin
/// 2) Validate and format the response from [destination > origin > proxy]
pub fn forward_request_and_return_response(
    parsed_req: &http::Request<Vec<u8>>,
) -> Result<Response<Vec<u8>>> {
    let mut proxy_origin_stream = TcpStream::connect(&get_origin_addr())?;
    // 1) write to origin
    // TODO: propagate error to client http response
    if let Err(err) = write_req_to_origin(&mut proxy_origin_stream, &parsed_req) {
        return Err(fmt_error(
            RequestError::ConnectionError(err),
            "Writing to origin:",
        ));
    };

    // 2.a) Read the response from origin
    let res_from_origin = read_res_from_origin(&mut proxy_origin_stream).or_else(|err| Err(err))?;

    // 2.b) validate response, proceed if 200 error code
    let response_status = res_from_origin.status().as_u16();
    if response_status != 200 {
        return Err(fmt_error(
            ResponseError::IncorrectResponse,
            &response_status.to_string(),
        ));
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

            // Insert
            let mut lock_w = cache.lock_write();

            let entry_mutex = CacheWriteLock::insert_req(&mut lock_w, parsed_req, res_from_origin);

            write_response_to_client(&mut client_proxy_connection, entry_mutex)?;
        }
    };

    Ok(())
}

/// Takes a http response object and writes to a stream
///
/// Abstraction that reduces reused code
/// No return
pub fn write_to_stream(
    stream: &mut TcpStream,
    status_str: String,
    header_map: &HeaderMap,
    http_body: &Vec<u8>,
) -> Result<()> {
    //res
    stream.write(&status_str.into_bytes())?;
    stream.write(b"\r\n")?;

    // TODO: propagate error to http response
    // write header to stream
    for (header_name, header_value) in header_map {
        stream.write(&format!("{}: ", header_name).as_bytes())?;
        stream.write(header_value.as_bytes())?;
        stream.write(b"\r\n")?;
    }
    stream.write(b"\r\n")?;

    // write body to stream
    if http_body.len() > 0 {
        stream.write(http_body)?;
    }

    Ok(())
}
