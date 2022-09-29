// imports
use failure;
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};
// local
use tcp_proxy::http_utils::{
    connection::write_to_stream,
    constants::*,
    errors,
    formatting::{get_origin_addr, Result},
    response::{write_error_res, Payload},
};

/// Get the payload from the endpoint
/// convert response to http response
/// TODO: url must be validated
/// TODO: url must be supported by already-implemented structs
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

    // validate
    let res_body_json = serde_json::from_str::<Vec<Payload>>(&res_body)?;
    let res_body_json_str = serde_json::to_string::<Vec<Payload>>(&res_body_json)?;
    let res_body_json_u8 = res_body_json_str.as_bytes().to_vec();

    let new_res = new_res.body(res_body_json_u8).unwrap();

    Ok(new_res)
}

// TODO: rename to `write_res_to_proxy_from_origin`
/// Write the response received from `destination` to `proxy`
///
/// Takes the dest. response object and tcp stream
/// No return value
fn write_res_to_proxy_from_origin(
    proxy_origin_stream: &mut TcpStream,
    res: http::Response<Vec<u8>>,
) -> Result<()> {
    let status_str = format!(
        "{:?} {} {}",
        res.version(),
        res.status().as_str(),
        res.status().canonical_reason().unwrap_or("")
    );
    write_to_stream(proxy_origin_stream, status_str, res.headers(), res.body())?;

    Ok(())
}

fn main() {
    // create listener
    let listener = TcpListener::bind(get_origin_addr()).unwrap();
    println!("Listening at: {}", listener.local_addr().unwrap());

    // check listener for incoming connections/http requests
    for connection in listener.incoming() {
        let mut proxy_origin_stream = connection.unwrap();

        ///////////////////////////////////////////////////
        // HANDLE INCOMING CONNECTION (request) FROM PROXY
        // init the buffer
        let mut buffer = [0; SIZE_MAX_HEADERS];
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
            let mut headers = [httparse::EMPTY_HEADER; AMT_MAX_HEADERS];
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
            if let Err(e) = write_res_to_proxy_from_origin(&mut proxy_origin_stream, res_with_json)
            {
                write_error_res(&e, &mut proxy_origin_stream, 400);
                break;
            }
            // send back to proxy
            /////////////////////////////////////////

            break;
        }
    }
}
