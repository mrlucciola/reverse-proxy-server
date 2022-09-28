// imports
use failure;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};
// local
pub mod http_utils;
use http_utils::connection::new_endpoint_str;

type Result<T> = std::result::Result<T, failure::Error>;

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

/// Get the payload from the endpoint
/// convert response to http response
/// TODO: url must be validated
/// TODO: url must be supported by already-implemented structs
// fn call_api(url: String) -> Result<Vec<Payload>> {
fn call_api(url: String) -> Result<http::Response<Vec<u8>>> {
    let res = reqwest::blocking::get(&url)?;

    // return to `http` lib response
    let mut new_res = http::Response::builder()
        .status(&res.status())
        .version(http::Version::HTTP_11);
    for (header_name, header_value) in res.headers() {
        new_res = new_res.header(header_name, header_value);
    }

    let res_body = res.text()?;

    let res_body_json = serde_json::from_str::<Vec<Payload>>(&res_body)?;
    let res_body_json_str = serde_json::to_string::<Vec<Payload>>(&res_body_json)?;
    let res_body_json_u8 = res_body_json_str.as_bytes().to_vec();

    let new_res = new_res.body(res_body_json_u8).unwrap();

    Ok(new_res)
}

fn write_res_to_proxy_stream_from_origin(
    proxy_origin_stream: &mut TcpStream,
    res: http::Response<Vec<u8>>,
) -> Result<()> {
    let data_to_forward = format!(
        "{:?} {} {}",
        res.version(),
        res.status().as_str(),
        res.status().canonical_reason().unwrap_or("")
    );
    proxy_origin_stream.write(&data_to_forward.into_bytes())?;
    proxy_origin_stream.write(b"\r\n")?;

    for (header_name, header_value) in res.headers() {
        proxy_origin_stream.write(&format!("{}: ", header_name).as_bytes())?;
        proxy_origin_stream.write(header_value.as_bytes())?;
        proxy_origin_stream.write(b"\r\n")?;
    }
    proxy_origin_stream.write(b"\r\n")?;

    if res.body().len() > 0 {
        proxy_origin_stream.write(res.body())?;
    }

    Ok(())
}

fn main() {
    const ORIG_PORT: u16 = 8080;
    const ORIG_ADDR: &str = "127.0.0.1";
    let origin_endpoint = new_endpoint_str(ORIG_ADDR, ORIG_PORT);

    // create listener
    let listener = TcpListener::bind(origin_endpoint).unwrap();
    println!(
        "(ORIGIN) Listening at endpoint: {}",
        listener.local_addr().unwrap()
    );

    // check listener for incoming connections/http requests
    for connection in listener.incoming() {
        let mut proxy_origin_stream = connection.unwrap();

        ///////////////////////////////////////////////////
        // HANDLE INCOMING CONNECTION (request) FROM PROXY
        // init the buffer
        let mut buffer = [0; 2_usize.pow(9)];
        let mut bytes_read = 0;

        loop {
            // check for new bytes
            // TODO: propagate error + code to http response
            let new_bytes = match proxy_origin_stream.read(&mut buffer[bytes_read..]) {
                Ok(nb) => nb,
                Err(err) => {
                    eprintln!("{:?}", err);
                    break;
                }
            };
            bytes_read += new_bytes;

            // init req headers
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut req = httparse::Request::new(&mut headers);

            // read to buffer
            // TODO: propagate error + code to http response
            let parsed = match req.parse(&buffer) {
                Ok(p) => p,
                Err(err) => {
                    eprintln!("Error parsing: {:?}", err);
                    continue;
                }
            };

            // check if the request is incomplete (partial)
            if parsed.is_partial() {
                eprintln!("{}", failure::format_err!("Warning: Partial request"));
            };

            /////////////////////////////////////////
            // build http `request`
            let mut new_req = http::Request::builder();
            for header in req.headers {
                new_req = new_req.header(header.name, header.value);
            }

            // add to the request
            let mut new_req = new_req
                .method(req.method.unwrap())
                .uri(req.path.unwrap())
                .version(http::Version::HTTP_11)
                .body(Vec::new())
                .unwrap();

            // add data to the body
            new_req
                .body_mut()
                .extend_from_slice(&buffer[parsed.unwrap()..bytes_read]);

            let body = new_req.body().to_vec();

            // TODO: validate the body - must be only URL
            let url = String::from_utf8(body).unwrap();

            // build http `request`
            /////////////////////////////////////////

            /////////////////////////////////////////
            // call external api, get json response; build response body

            // TODO: propagate error + code to http response
            let res_with_json = match call_api(url) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("Error calling api or json: {}", failure::err_msg(err));
                    break;
                }
            };
            // call external api, get json response; build response body
            /////////////////////////////////////////

            /////////////////////////////////////////
            // send back to proxy

            // TODO: propagate error + code to http response
            if let Err(e) =
                write_res_to_proxy_stream_from_origin(&mut proxy_origin_stream, res_with_json)
            {
                eprintln!("Error writing to stream: {}", e);
                break;
            }
            // send back to proxy
            /////////////////////////////////////////

            break;
        }
    }
}
