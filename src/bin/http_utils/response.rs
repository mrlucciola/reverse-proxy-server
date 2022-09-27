// libs
use chrono::Utc;
use http::{Request, Response};
use std::{io::Read, net::TcpStream};
// local
use super::connection::{self, check_body_len};
// constants
pub const MAX_NUM_HEADERS: usize = 32;

#[derive(Debug)]
enum Error {
    /// Client hung up before sending a complete request
    IncompleteResponse,
    /// Client sent an invalid HTTP request. httparse::Error contains more details
    MalformedResponse(httparse::Error),
    /// The Content-Length header is present, but does not contain a valid numeric value
    InvalidContentLength,
    /// The Content-Length header does not match the size of the request body that was sent
    ContentLengthMismatch,
    /// The request body is bigger than MAX_BODY_SIZE
    ResponseBodyTooLarge,
    /// The request body is bigger than MAX_BODY_SIZE
    ResponseBodyError(connection::Error),
    /// Read response
    // ReadResponseError(Error),
    /// Encountered an I/O error when reading/writing a TcpStream
    ConnectionError(std::io::Error),
}

fn parse_res_from_origin(buffer: &[u8]) -> Result<Option<(http::Response<Vec<u8>>, usize)>, Error> {
    // init headers
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut res_init = httparse::Response::new(&mut headers);

    // parse the response into res_init, get status
    let res_status = res_init
        .parse(buffer)
        .or_else(|err| Err(Error::MalformedResponse(err)))?;

    // Ok(Some((resp, 3)))
    if let httparse::Status::Complete(len) = res_status {
        let mut res = http::Response::builder()
            .status(res_init.code.unwrap())
            .version(http::Version::HTTP_11);
        for header in res_init.headers {
            res = res.header(header.name, header.value);
        }
        let res = res.body(Vec::new()).unwrap();
        return Ok(Some((res, len)));
    }
    Ok(None)
}

/// Read the response from origin
pub fn read_res_from_origin(
    proxy_origin_stream: &mut TcpStream,
    parsed_req: &Request<Vec<u8>>,
) -> Result<Response<Vec<u8>>, Error> {
    // method should only be GET
    // &http::Method::GET;
    let res_from_origin =
        match read_res_from_origin_stream(proxy_origin_stream, &parsed_req.method()) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Error getting res from orig: {:?}", err);
                return Err(err);
            }
        };
    Ok(res_from_origin)
}

pub fn read_res_from_origin_stream(
    proxy_origin_stream: &mut TcpStream,
    request_method: &http::Method,
) -> Result<http::Response<Vec<u8>>, Error> {
    // let mut response = read_headers(stream)?;
    let mut res_buffer = [0_u8; 64];
    let mut bytes_read = 0;

    loop {
        // Read bytes from the connection into the buffer, starting at position bytes_read
        let new_bytes = proxy_origin_stream
            .read(&mut res_buffer[bytes_read..])
            .or_else(|err| Err(Error::ConnectionError(err)))?;

        // handle incomplete response
        if new_bytes == 0 {
            return Err(Error::IncompleteResponse);
        }
        bytes_read += new_bytes;

        // check for valid response
        let slice = &res_buffer[..bytes_read];
        if let Some((mut parsed_res, headers_len)) =
            parse_res_from_origin(&res_buffer[..bytes_read])?
        {
            // return the remainder of the response body (without the headers)
            parsed_res
                .body_mut()
                // .extend_from_slice(&res_buffer[headers_len..bytes_read]);
                .extend_from_slice(&res_buffer[headers_len..bytes_read]);

            // let xxx = parsed_res.extensions();

            // String::from_utf8_lossy(&read_buffer_header) str::from_utf8
            // A response may have a body as long as it is not responding to a HEAD request and as long as
            // the response status code is not 1xx, 204 (no content), or 304 (not modified).
            if !(request_method == http::Method::HEAD
                || parsed_res.status().as_u16() < 200
                || parsed_res.status() == http::StatusCode::NO_CONTENT
                || parsed_res.status() == http::StatusCode::NOT_MODIFIED)
            {
                read_res_body(proxy_origin_stream, &mut parsed_res)?;
            }
            return Ok(parsed_res);
        }
    }
}

pub fn read_res_body(
    stream: &mut TcpStream,
    response: &mut http::Response<Vec<u8>>,
) -> Result<(), Error> {
    // The response may or may not supply a Content-Length header. If it provides the header, then
    // we want to read that number of bytes; if it does not, we want to keep reading bytes until
    // the connection is closed.
    let content_len = match check_body_len(response.headers()) {
        Ok(item) => Ok(item),
        Err(err) => Err(Error::ResponseBodyError(err)),
    }?;

    while content_len > 0 || response.body().len() < content_len {
        let mut buffer = [0_u8; 512];
        let bytes_read = stream
            .read(&mut buffer)
            .or_else(|err| Err(Error::ConnectionError(err)))?;
        if bytes_read == 0 {
            // The server has hung up!
            if content_len == 0 {
                // We've reached the end of the response
                break;
            } else {
                // Content-Length was set, but the server hung up before we managed to read that
                // number of bytes
                return Err(Error::ContentLengthMismatch);
            }
        }

        // Make sure the server doesn't send more bytes than it promised to send
        if content_len > 0 && response.body().len() + bytes_read > content_len {
            return Err(Error::ContentLengthMismatch);
        }

        // Make sure server doesn't send more bytes than we allow
        if response.body().len() + bytes_read > 10000000 {
            return Err(Error::ResponseBodyTooLarge);
        }

        // Append received bytes to the response body
        response.body_mut().extend_from_slice(&buffer[..bytes_read]);
    }

    Ok(())
}

/// build the response object to send to the client
pub fn build_response() -> String {
    let status_line = "HTTP/1.1 200 OK";
    let contents = Utc::now().to_string();
    let content_len = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {content_len}\r\n\r\n{contents}");
    let mut headers = [httparse::EMPTY_HEADER; MAX_NUM_HEADERS];
    let checked_res = Response::new(&mut headers);

    response
}
