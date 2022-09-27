// libs
use std::{
    io::Write,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    process::exit,
};

// local
use super::request::{self, get_parsed_request, write_req_to_origin};
use super::response::{build_response, read_res_from_origin};

// Constants
pub const MAX_BODY_SIZE: usize = 10000000;

#[derive(Debug)]
pub enum Error {
    /// The Content-Length header is present, but does not contain a valid numeric value
    InvalidContentLength,
    /// The request body is bigger than MAX_BODY_SIZE
    BodySizeTooLarge,
    /// No headers in map
    EmptyHeaderValue,
}

pub fn check_body_len(header_map: &http::HeaderMap) -> Result<usize, Error> {
    let header_value = header_map.get("content-length");
    if header_value.is_none() {
        return Ok(0);
    };

    let content_body_len = header_map
        .get("content-length")
        .unwrap()
        .to_str()
        .or(Err(Error::InvalidContentLength))?
        .parse::<usize>()
        .or(Err(Error::InvalidContentLength))?;

    if content_body_len > MAX_BODY_SIZE {
        return Err(Error::BodySizeTooLarge);
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

pub fn handle_connection(
    mut client_proxy_connection: TcpStream,
    origin_endpoint: &String,
) -> Result<(), request::Error> {
    // 1) parse http request
    let parsed_req = get_parsed_request(&mut client_proxy_connection)?;
    let request_uri = parsed_req.uri();

    // handle error if no origin
    // let origin_endpoint = new_endpoint_str("127.0.0.1", 8080);
    let mut proxy_origin_stream = match TcpStream::connect(origin_endpoint) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Error: Please re-start the origin server {}", err);
            exit(1);
        }
    };

    if request_uri == "/" {
        // 2.a) write to origin
        if let Err(err) = write_req_to_origin(&mut proxy_origin_stream, &parsed_req) {
            eprintln!("Error writing to origin: {}", err);
            return Err(request::Error::ConnectionError(err));
        };
        // 2.b) read from origin
        let res_from_origin = read_res_from_origin(&mut proxy_origin_stream, &parsed_req)
            .or_else(|err| Err(request::Error::MiscError(err)))?;

        // 3) respond to client
        println!("res_from_origin: {:?}", res_from_origin);
        let res = build_response();

        client_proxy_connection.write_all(res.as_bytes()).unwrap();
        return Ok(());
    } else {
        println!("not found: {:?}", request_uri);
    }
    Ok(())
}

fn handle_connection_pt2(mut client_proxy_connection: TcpStream) -> Result<(), request::Error> {
    // let mut read_buffer_header: Vec<u8> = vec![0; MAX_HEADERS_SIZE];
    // let mut out_buffer: Vec<u8> = vec![0; 2_usize.pow(9)];

    // // 1) read client req
    // if let Err(err) = client_proxy_connection.read(&mut read_buffer_header) {
    //     return Err(request::Error::ConnectionError(err));
    //     // eprintln!("Error: incoming client-proxy connection: {}", err);
    // }

    // println!(
    //     "1) incoming req: {}",
    //     String::from_utf8_lossy(&read_buffer_header)
    // );

    // // init parser
    // let mut headers = [httparse::EMPTY_HEADER; MAX_NUM_HEADERS];
    // let mut req = httparse::Request::new(&mut headers);

    // let res = req
    //     .parse(&read_buffer_header)
    //     .or_else(|err| Err(request::Error::MalformedRequest(err)))?;

    // let mut req_vec: Request<Vec<_>>;
    // let headers_len: usize;
    // if let httparse::Status::Complete(len) = res {
    //     // get length of headers
    //     headers_len = len;

    //     // build the request
    //     let mut request_builder = http::Request::builder()
    //         .method(req.method.unwrap())
    //         .uri(req.path.unwrap())
    //         .version(http::Version::HTTP_11);

    //     // append headers
    //     for header in req.headers {
    //         request_builder = request_builder.header(header.name, header.value);
    //     }

    //     req_vec = request_builder.body(Vec::new()).unwrap();
    // } else {
    //     return Err(request::Error::MalformedRequest(httparse::Error::Status));
    // }

    // req_vec
    //     .body_mut()
    //     .extend_from_slice(&read_buffer_header[headers_len..read_buffer_header.len()]);

    // // previously we just completed the header
    // // body

    // fn handle_req_body(
    //     req_vec: Request<Vec<u8>>,
    //     client_proxy_connection: &mut TcpStream,
    //     in_buffer: &mut Vec<u8>,
    // ) -> Result<usize, request::Error> {
    //     let content_len = check_req_body_len(&req_vec)?;

    //     while req_vec.body().len() < content_len {
    //         // init the read buffer: 2^9=512 byte arr
    //         let mut read_buffer = vec![0_u8; std::cmp::min(2_usize.pow(9), content_len)];
    //         // copy bytes to buffer
    //         // bytes_read ==
    //         let bytes_read = client_proxy_connection
    //             .read(&mut read_buffer)
    //             .or_else(|err| Err(request::Error::ConnectionError(err)))?;
    //     }
    //     Ok(content_len)
    // }

    // // read_body(stream==client_conn, &mut request, content_length)?;
    // // read_body(client_proxy_connection, &mut req_vec, content_len)?;
    // handle_req_body(
    //     req_vec,
    //     &mut client_proxy_connection,
    //     &mut read_buffer_header,
    // );

    return Ok(());
    // return Ok(req_vec);
    // let res_x: Response<Vec<u8>>;
    // let is_in_cache = false;
    // if is_in_cache { // cache.contains_entry(&req)
    // } else {
    //     // fwd req & return res
    //     // res_x = Self::forward_request_and_return_response(&req, &mut host_conn);
    //     let req_line = format!("{} {} {:?}", req.method(), req.uri(), req.version());
    //     if let Err(err) = client_proxy_connection.write(&format_request_line(request).into_bytes())
    //     {
    //         println!("error");
    //     };
    //     client_proxy_connection.write(&['\r' as u8, '\n' as u8])?;
    //     /////////////
    //     // if let Err(err) = request::write_to_stream(&req, host_conn) {
    //     //     log::error!(
    //     //         "Failed to send request to host {:?}: {:?}",
    //     //         host_conn.peer_addr().unwrap().ip(),
    //     //         err
    //     //     );
    //     //     return response::make_http_error(http::StatusCode::BAD_GATEWAY);
    //     // }
    //     // add to cache
    //     // cache.add_entry(&req, &res);
    // }
    // let res = req
    //     .parse(&in_buffer)
    //     .or_else(|err| Err(request::Error::MalformedRequest(err)));
    // match res {
    //     Ok(x) => {
    //         println!("handled: {:#?}", x);
    //     }
    //     Err(e) => {
    //         println!("Oopsies: {:?}", e);
    //     }
    // }

    // // 2) write to origin
    // let _orig_fwd = proxy_origin_connection
    //     .write(&mut in_buffer)
    //     .expect("Error writing to origin");
    // println!("2) Forwarding req to origin");

    // // 3) read res from origin
    // let _orig_recv = proxy_origin_connection.read(&mut out_buffer).unwrap();
    // println!(
    //     "3) Received res from origin: {}",
    //     String::from_utf8_lossy(&out_buffer)
    // );

    // // 4) write res to client
    // let mut headers = [httparse::EMPTY_HEADER; MAX_NUM_HEADERS];
    // let mut resp = httparse::Response::new(&mut headers);
    // let res_result = resp
    //     .parse(&out_buffer)
    //     .or_else(|err| Err(http_utils::response::Error::MalformedResponse(err)));
    // let res;
    // match res_result {
    //     Ok(x) => {
    //         println!("parsed res: {:?}", x);
    //         res = x
    //     }
    //     Err(e) => println!("error: {:?}", e),
    // }
    // // client_proxy_connection.write(buf)
    // let _proxy_fwd = client_proxy_connection
    //     .write(&mut out_buffer)
    //     .expect("Error: writing to client");
    // println!("4) Forwarding res to client {}", _proxy_fwd);
    // println!(
    //     "out buf: \n\n{:?}\n\ndun\n",
    //     String::from_utf8_lossy(&out_buffer)
    // );
    // // stream.write(&['\r' as u8, '\n' as u8])?;
}
