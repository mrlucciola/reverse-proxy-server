// libs
use failure::{self, Fail};
use http::Response;
use httparse;
use serde::{Deserialize, Serialize};
use std::{io::Read, net::TcpStream, sync::Mutex};
// local
pub use super::{
    connection::{check_body_len, write_to_stream},
    constants::*,
    errors::{fmt_error, ResponseError, Result},
};

#[derive(Deserialize, Serialize, Debug)]
pub struct Payload {
    pub id: String, // "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f 1b60a8ce26f",
    pub height: u32, // 0,
    pub version: u32, // 1,
    pub timestamp: u32, // 1231006505,
    pub tx_count: u32, // 1,
    pub size: u32,  // 285,
    pub weight: u32, // 816,
    pub merkle_root: String, // "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
    pub previousblockhash: Option<String>, // null,
    pub mediantime: u32, // 1231006505,
    pub nonce: u32, // 2083236893,
    pub bits: u32,  // 486604799,
    pub difficulty: u32, // 1,
}

fn check_for_complete_request(res_status: httparse::Status<usize>) -> Option<usize> {
    if let httparse::Status::Complete(res_len) = res_status {
        Some(res_len)
    } else {
        None
    }
}

fn parse_res_from_origin(buffer: &[u8]) -> Result<Option<(http::Response<Vec<u8>>, usize)>> {
    // init headers
    let mut headers = [httparse::EMPTY_HEADER; AMT_MAX_HEADERS];
    let mut res_init = httparse::Response::new(&mut headers);

    // parse the response into res_init, get status
    // TODO: propagate error to client http response
    let res_status = res_init
        .parse(buffer)
        .or_else(|err| Err(fmt_error(ResponseError::MalformedResponse(err), "")))?;

    // if this is a complete request, build and return response
    // TODO: propagate error to client http response
    let res_len = match check_for_complete_request(res_status) {
        Some(len) => len,
        None => return Err(failure::err_msg("Buffer overflow")),
    };

    // init the response builder
    let mut res = http::Response::builder()
        .status(res_init.code.unwrap())
        .version(http::Version::HTTP_11);

    // add headers to the response builder
    for header in res_init.headers {
        res = res.header(header.name, header.value);
    }

    // init the response body
    let res: Response<Vec<u8>> = res.body(Vec::new()).unwrap();

    Ok(Some((res, res_len)))
}

/// For Proxy: read the response from origin
pub fn read_res_from_origin(proxy_origin_stream: &mut TcpStream) -> Result<Response<Vec<u8>>> {
    // init response buffer
    let mut res_buffer = [0_u8; 2_usize.pow(10) * 8]; // 8 kb buffer
    let mut bytes_read = 0;

    loop {
        // read incoming stream and write bytes into the buffer
        // TODO: propagate error to client http response
        let new_bytes = proxy_origin_stream
            .read(&mut res_buffer[bytes_read..])
            .or_else(|err| {
                Err(fmt_error(
                    ResponseError::ConnectionError(err),
                    "Error reading new byes:",
                ))
            })?;

        // handle incomplete response
        if new_bytes == 0 {
            break;
        }

        bytes_read += new_bytes;
    }

    // check for valid response
    // TODO: propagate error to client http response
    let parsed_res_option = parse_res_from_origin(&res_buffer[..bytes_read])?;
    if let None = parsed_res_option {
        return Err(failure::err_msg("Incomplete response - returned none"));
    }

    let (mut parsed_res, headers_len) = parsed_res_option.unwrap();

    // return the remainder of the response body (without the headers)
    parsed_res
        .body_mut()
        .extend_from_slice(&res_buffer[headers_len..bytes_read]);

    return Ok(parsed_res);
}

/// Build the response object to send to the client
/// Response value is mutex because it could be coming from the cache
/// Lock drop happens when scope closes
/// TODO: consider- unwrapping mutex before the fxn call
pub fn write_response_to_client<'b>(
    stream: &mut TcpStream,
    res: &Mutex<Response<Vec<u8>>>,
) -> Result<()> {
    let res = res.lock().expect("Poisoned mutex: writing to client");
    let status_str = format!(
        "{:?} {} {}",
        res.version(),
        res.status().as_str(),
        res.status().canonical_reason().unwrap_or("")
    );
    write_to_stream(stream, status_str, res.headers(), res.body())?;

    Ok(())
}

/// PLACEHOLDER - Does not do anything
///
/// Function takes the returned error, initiates builds and sends the response
pub fn write_error_res(err: &failure::Error, stream: &mut TcpStream, err_status: u16) {
    // failure::err_msg(format!("{err:?}"));
    // let err = failure::Error::from(err);

    ////////////////////////////////////////////////////
    // create the response (below)
    let res = Response::new("");
    let builder = Response::builder()
        .status(err_status)
        .version(http::Version::HTTP_11);
    // create the response (above)
    ////////////////////////////////////////////////////

    let status_str = String::from("");
    let status_str = format!(
        "{:?} {} {}",
        res.version(),
        res.status().as_str(),
        res.status().canonical_reason().unwrap_or("")
    );

    // write_to_stream(stream, status_str, header_map, http_body);
}
