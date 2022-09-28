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

// fn build_res_to_proxy(
//     http_status_code: u16,
//     http_status_text: String,
//     res_status_str: String,
// ) -> String {
//     let line1 = format!("HTTP/1.1 {http_status_code} {http_status_text}");
//     let line2 = format!("Content-Type: text/html");
//     let line3 = format!("Content-Length:{}", res_status_str.len());
//     let line4 = format!("{res_status_str}");

//     format!("{}\n{}\n{}\n\n{}", line1, line2, line3, line4)
// }

// fn handle_connection(
//     proxy_origin_stream: &mut TcpStream,
//     parsed_req: http::Request<Vec<u8>>,
//     json: Vec<Payload>,
// ) -> Result<()> {
//     println!("\n\nin handle conn\n");
//     let cond_invalid_method = parsed_req.method() != http::Method::GET;
//     if cond_invalid_method {
//         eprintln!("Please use GET request");
//         let res_str = build_res_to_proxy(400, "Invalid".to_string(), "Invalid request".to_string());

//         proxy_origin_stream.write(res_str.as_bytes())?;
//         return Err(failure::err_msg("Please use GET request"));
//     }
//     println!("parsed_req: {:?}", parsed_req);
//     let res_str = build_res_to_proxy(200, "OK".to_string(), "Successful".to_string());
//     println!("res_str: {res_str}");

//     println!("converting json to str...");
//     let json_str = serde_json::to_string(&json)?;
//     println!("json_str: \n{json_str}\n");
//     // add json to the response body

//     // write to the stream
//     proxy_origin_stream.write(res_str.as_bytes())?;

//     Ok(())
// }

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

        // read to the buffer
        // proxy_origin_stream.read(&mut buffer).unwrap();

        // let parsed_req_result = get_parsed_request(&mut proxy_origin_stream);
        // let mut in_buffer = [0_u8; MAX_HEADERS_SIZE];
        let mut bytes_read = 0;

        loop {
            // check for new bytes
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

            println!("After init request object");
            let parsed = match req.parse(&buffer) {
                Ok(p) => p,
                Err(err) => {
                    eprintln!("Error parsing: {:?}", err);
                    continue;
                }
            };
            println!("After parse check");
            // check if the request is incomplete (partial)
            if parsed.is_partial() {
                eprintln!("{}", failure::format_err!("Warning: Partial request"));
            };
            println!("After partial check");
            // build proper `request` body
            let mut new_req = http::Request::builder();
            for header in req.headers {
                new_req = new_req.header(header.name, header.value);
            }
            println!("After new req builder");
            // build the request
            let mut new_req = new_req
                .method(req.method.unwrap())
                .uri(req.path.unwrap())
                .version(http::Version::HTTP_11)
                .body(Vec::new())
                .unwrap();

            println!("After new request object");
            // add data to the body
            new_req
                .body_mut()
                .extend_from_slice(&buffer[parsed.unwrap()..bytes_read]);

            println!("After adding data to request body");

            let body = new_req.body().to_vec();
            let url = String::from_utf8(body).unwrap();
            println!("After url parsed: {}", url);

            /////////////////////////////////////////
            // call external api, get json response; build response body
            let res_with_json = match call_api(url) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("here: {}", failure::err_msg(err));
                    break;
                }
            };
            println!(
                "After res_with_json: body: {} content: {:?}",
                &res_with_json.body().len(),
                &res_with_json.headers().get("content-length").unwrap()
            );
            // http::Response<Vec<Payload>>
            // call external api, get json response; build response body
            /////////////////////////////////////////

            /////////////////////////////////////////
            // send back to proxy
            if let Err(e) =
                write_res_to_proxy_stream_from_origin(&mut proxy_origin_stream, res_with_json)
            {
                eprintln!("Error writing to stream: {}", e);
                break;
            } else {
                println!("Writing to proxy - success")
            }
            println!("After writing to proxy");
            // send back to proxy
            /////////////////////////////////////////

            break;
        }
        // if let Err(err) = parsed_req_result {
        //     eprintln!("Error parsing request: {:?}", err);
        //     return;
        // }
        // let parsed_req = parsed_req_result.unwrap();
        // println!(
        //     "\n IN ORI: parsed_req: {:?}\n\n",
        //     String::from_utf8(parsed_req.body().to_vec())
        // );
        // parsed_req.body()

        // request lines
        // let req_line = "";
        // let str_request_line = if let Some(line) = str::from_utf8(&buffer).unwrap().lines().next() {
        //     line
        // } else {
        //     println!("Error parsing request line");
        //     req_line
        // };
        // let req_line = RequestLine::from_str(str_request_line).unwrap();

        // // build the logic to build responses from requests
        // let html_res_str = build_response(req_line,);

        // println!("res to send: {:?}", html_res_str);

        // connection.write(html_res_str.as_bytes()).unwrap();
    }
}

// fn build_response(req_line: RequestLine, body: Vec<Payload>) -> String {
//     let html_res_str: String;
//     let status: String;

//     println!("len is {}", req_line.get_resource_id().len());

//     let cond_invalid_req = req_line.method() != "GET" || !req_line.path().starts_with("/status");
//     if cond_invalid_req {
//         status = format!("Not found");
//         html_res_str = format!(
//             "{}\n{}\nContent-Length:{}\n\n{}",
//             "HTTP/1.1 404 Not Found\n",
//             "Content-Type: text/html",
//             status.len(),
//             status
//         );
//     } else {
//         status = format!(
//             "{} {}: Exists\n",
//             "Status for item #",
//             req_line.get_resource_id()
//         );

//         html_res_str = format!(
//             "{} {} {}\n\n{}",
//             "HTTP/1.1 200 OK\nContent-Type:",
//             "text/html\nContent-Length:",
//             status.len(),
//             status
//         );
//     }
//     html_res_str
// }
