// libs
use http::Request;
use httparse;
use std::{
    io::{Read, Write},
    net::TcpStream,
};
// local
pub use super::{
    constants::*,
    errors::{fmt_error, RequestError, Result},
};

/// This function forwards the incoming request to the `origin`.
///
/// TODO: propagate error to http response
/// Fxn receives a stream to the `origin` from `proxy`, and a `Request` parsed by `http` crate
pub fn write_req_to_origin(
    proxy_origin_stream: &mut TcpStream,
    parsed_req: &Request<Vec<u8>>,
) -> Result<()> {
    // build the message to send
    let data_to_forward = format!(
        "{} {} {:?}",
        parsed_req.method(),
        parsed_req.uri(),
        parsed_req.version()
    );

    // TODO: propagate error to http response
    // write to stream
    proxy_origin_stream.write(&data_to_forward.into_bytes())?;
    proxy_origin_stream.write(b"\r\n")?;

    // write header to stream
    for (header_name, header_value) in parsed_req.headers() {
        proxy_origin_stream.write(&format!("{}: ", header_name).as_bytes())?;
        proxy_origin_stream.write(header_value.as_bytes())?;
        proxy_origin_stream.write(b"\r\n")?;
    }
    proxy_origin_stream.write(b"\r\n")?;

    // write body to stream
    if parsed_req.body().len() > 0 {
        proxy_origin_stream.write(parsed_req.body())?;
    }

    Ok(())
}

pub fn get_parsed_request(stream: &mut TcpStream) -> Result<Request<Vec<u8>>> {
    // init request buffer
    let mut in_buffer = [0_u8; SIZE_MAX_HEADERS];
    let mut bytes_read = 0;

    loop {
        // Read bytes from the connection into the buffer, starting at position bytes_read
        // TODO: propagate error to client http response
        let new_bytes = stream
            .read(&mut in_buffer[bytes_read..])
            .or_else(|err| Err(fmt_error(err, "RequestError::ConnectionError")))?;

        // handle incomplete request
        if new_bytes == 0 {
            return Err(fmt_error(RequestError::IncompleteRequest(bytes_read), ""));
        }
        bytes_read += new_bytes;

        // init header buffer
        let mut headers = [httparse::EMPTY_HEADER; AMT_MAX_HEADERS];
        let mut req = httparse::Request::new(&mut headers);

        if let Ok(parsed) = req.parse(&in_buffer) {
            // 1.a) check request, proceed if GET requets
            // TODO: handle error if no origin
            // TODO: propagate error to client http response

            if req.method.unwrap() != http::Method::GET {
                return Err(failure::err_msg(format!(
                    "Error::RequestMethod- please use GET.  Submitted: {}",
                    req.method.unwrap()
                )));
            }

            // check if the request is incomplete (partial)
            // TODO: propagate error to client http response
            if parsed.is_partial() {
                return Err(fmt_error(
                    RequestError::IncompleteRequest(parsed.unwrap()),
                    "",
                ));
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
                .extend_from_slice(&in_buffer[parsed.unwrap()..bytes_read]);

            return Ok(new_req);
        }
    }
}
