// libs
use failure;
use http::Method;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    process::exit,
};
// local
use super::errors::*;
use super::request::{get_parsed_request, write_req_to_origin};
use super::response::{read_res_from_origin, write_response_to_client};

// Constants
pub const MAX_BODY_SIZE: usize = 10000000;

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

    if content_body_len > MAX_BODY_SIZE {
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
// fn new_endpoint_str(addr: &str, port: u16) -> String {
//     let addr_parsed = IpAddr::V4(addr.parse::<Ipv4Addr>().unwrap());
//     let endpoint = SocketAddr::new(addr_parsed, port);

//     endpoint.to_string()
// }

/// Handle the tcp connection between client and proxy
///
/// 1) forward request to origin
/// 2) receive response from origin
/// 3) write back to client
pub fn handle_connection(
    client_proxy_connection: &mut TcpStream,
    origin_endpoint: &String,
) -> Result<()> {
    // 1) parse http request
    let parsed_req = get_parsed_request(client_proxy_connection)?;

    // 1.a) check request, proceed if GET requets
    // TODO: handle error if no origin
    if parsed_req.method() != Method::GET {
        return Err(failure::err_msg(format!(
            "Error::RequestMethod- please use GET.  Submitted: {}",
            parsed_req.method()
        )));
    }

    // 2) Write to origin

    // 2.a) Open stream
    let mut proxy_origin_stream = match TcpStream::connect(origin_endpoint) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Error: Please re-start the origin server {}", err);
            exit(1);
        }
    };
    // 2.b) write to origin
    if let Err(err) = write_req_to_origin(&mut proxy_origin_stream, &parsed_req) {
        return Err(fmt_error(
            RequestError::ConnectionError(err),
            "Error writing to origin:",
        )); // prev- RequestError::ConnectionError(err)
    };

    // 3) Read from origin
    let res_from_origin = read_res_from_origin(&mut proxy_origin_stream).or_else(|err| Err(err))?;

    // 3.a) check response, proceed if 200 error code
    if res_from_origin.status().as_u16() != 200 {
        return Err(failure::err_msg(format!(
            "Invalid response: {}",
            res_from_origin.status().as_u16()
        )));
    }

    // 4) respond to client
    // TODO: log timestamps for client, proxy, & origin requests and responses
    write_response_to_client(client_proxy_connection, res_from_origin)?;

    Ok(())
}
