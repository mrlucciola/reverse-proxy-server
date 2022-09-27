use failure;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    // string::ParseError,
};
// use tokio;
// local
mod http_utils;
use http_utils::connection::new_endpoint_str;

type Result<T> = std::result::Result<T, failure::Error>;

// #[derive(Debug)]
// pub struct RequestLine {
//     method: Option<String>,
//     path: Option<String>,
// }
// impl RequestLine {
//     fn method(&self) -> String {
//         if let Some(method) = &self.method {
//             method.to_string()
//         } else {
//             String::from("")
//         }
//     }
//     fn path(&self) -> String {
//         if let Some(path) = &self.path {
//             path.to_string()
//         } else {
//             String::from("")
//         }
//     }
//     fn get_resource_id(&self) -> String {
//         let path = self.path();
//         let path_tokens: Vec<String> = path.split("/").map(|line| line.parse().unwrap()).collect();
//         path_tokens[path_tokens.len() - 1].clone()
//     }
// }

// impl FromStr for RequestLine {
//     type Err = ParseError;
//     fn from_str(msg: &str) -> std::result::Result<Self, Self::Err> {
//         let mut msg_tokens = msg.split_ascii_whitespace();
//         let method = match msg_tokens.next() {
//             Some(token) => Some(String::from(token)),
//             None => None,
//         };
//         let path = match msg_tokens.next() {
//             Some(token) => Some(String::from(token)),
//             None => None,
//         };

//         Ok(Self { method, path })
//     }
// }

// use serde_json
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
/// TODO: url must be validated
/// TODO: url must be supported by already-implemented structs
fn call_api(url: String) -> Result<Vec<Payload>> {
    let res = match reqwest::blocking::get(&url) {
        Ok(r) => r,
        Err(err) => {
            println!("no clue: {:?}", err);
            if err.is_builder() {};
            return Err(failure::err_msg(err));
        }
    };
    let body = res.text()?;
    // reqwest::Error::is_builder()
    let json = serde_json::from_str::<Vec<Payload>>(&body)?;

    Ok(json)
}

fn build_res_to_proxy(
    http_status_code: u16,
    http_status_text: String,
    res_status_str: String,
) -> String {
    let line1 = format!("HTTP/1.1 {http_status_code} {http_status_text}");
    let line2 = format!("Content-Type: text/html");
    let line3 = format!("Content-Length:{}", res_status_str.len());
    let line4 = format!("{res_status_str}");

    format!("{}\n{}\n{}\n\n{}", line1, line2, line3, line4)
}

fn handle_connection(
    proxy_origin_stream: &mut TcpStream,
    parsed_req: http::Request<Vec<u8>>,
    json: Vec<Payload>,
) -> Result<()> {
    println!("\n\nin handle conn\n\n");
    // let parsed_req
    let cond_invalid_method = parsed_req.method() != http::Method::GET;
    if cond_invalid_method {
        eprintln!("Please use GET request");
        let res_str = build_res_to_proxy(400, "Invalid".to_string(), "Invalid request".to_string());

        proxy_origin_stream.write(res_str.as_bytes())?;
        return Err(failure::err_msg("Please use GET request"));
    }
    println!("parsed_req: {:?}", parsed_req);
    let res_str = build_res_to_proxy(200, "OK".to_string(), "Successful".to_string());

    let stringy = serde_json::to_string(&json)?;
    proxy_origin_stream.write(res_str.as_bytes())?;

    Ok(())
}

fn main() -> Result<()> {
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
        let mut connection = connection.unwrap();

        // init the buffer
        let mut buffer = [0; 2_usize.pow(9)];

        // read to the buffer
        // connection.read(&mut buffer).unwrap();

        // let parsed_req_result = get_parsed_request(&mut connection);
        // let mut in_buffer = [0_u8; MAX_HEADERS_SIZE];
        let mut bytes_read = 0;

        loop {
            // check for new bytes
            let new_bytes = connection.read(&mut buffer[bytes_read..])?;
            bytes_read += new_bytes;

            // init headers
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut req = httparse::Request::new(&mut headers);

            let parsed = req.parse(&buffer)?;
            // check if the request is incomplete (partial)
            if parsed.is_partial() {
                return Err(failure::format_err!("Error: Incomplete request"));
                // return Err(Box<dyn >);
                // return Err((""));
            };
            // build proper `request` body
            let mut new_req = http::Request::builder();
            for header in req.headers {
                new_req = new_req.header(header.name, header.value);
            }
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

            // return Ok(new_req);
            // let xxx = new_req.into_body().into_iter();
            let body = new_req.body().to_vec();
            let url = String::from_utf8(body).unwrap();

            // for xyz in xxx {
            //     println!("str: {:?}", String::from_utf8_lossy(&[xyz]));
            // }
            println!("\n IN ORI: parsed_req: {:?}\n\n", url);
            let json = call_api(url)?;

            /////////////////////////////////////////
            // send back to proxy

            // println!("body = {:?}", body);

            // fn build_res(parsed_req: http::Request<Vec<u8>>) -> String {
            //     let cond_invalid_req = parsed_req.method() != "GET";
            //     "".to_string()
            // }

            println!("almost at handle 0 ");
            let parsed_req = new_req;
            println!("almost at handle");
            // let html_res_str = build_res(parsed_req);
            handle_connection(&mut connection, parsed_req, json)?;

            // write to stream
            // connection.write(html_res_str.as_bytes()).unwrap();
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
    Ok(())
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
