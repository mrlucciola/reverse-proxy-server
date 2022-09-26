const MAX_NUM_HEADERS: usize = 32;

#[derive(Debug)]

pub enum Error {
    /// Client hung up before sending a complete request. IncompleteRequest contains the number of
    /// bytes that were successfully read before the client hung up
    IncompleteRequest(usize),
    /// Client sent an invalid HTTP request. httparse::Error contains more details
    MalformedRequest(httparse::Error),
    /// The Content-Length header is present, but does not contain a valid numeric value
    InvalidContentLength,
    /// The Content-Length header does not match the size of the request body that was sent
    ContentLengthMismatch,
    /// The request body is bigger than MAX_BODY_SIZE
    RequestBodyTooLarge,
    /// Encountered an I/O error when reading/writing a TcpStream
    ConnectionError(std::io::Error),
}

/// Attempts to parse the data in the supplied buffer as an HTTP request. Returns one of the
/// following:
///
/// * If there is a complete and valid request in the buffer, returns Ok(Some(http::Request))
/// * If there is an incomplete but valid-so-far request in the buffer, returns Ok(None)
/// * If there is data in the buffer that is definitely not a valid HTTP request, returns Err(Error)
///
/// You won't need to touch this function.
pub fn parse_request(buffer: &[u8]) -> Result<Option<(http::Request<Vec<u8>>, usize)>, Error> {
    let mut headers = [httparse::EMPTY_HEADER; MAX_NUM_HEADERS];
    let mut req = httparse::Request::new(&mut headers);
    let res = req
        .parse(buffer)
        .or_else(|err| Err(Error::MalformedRequest(err)))?;

    if let httparse::Status::Complete(len) = res {
        let mut request = http::Request::builder()
            .method(req.method.unwrap())
            .uri(req.path.unwrap())
            .version(http::Version::HTTP_11);
        for header in req.headers {
            request = request.header(header.name, header.value);
        }
        let request = request.body(Vec::new()).unwrap();
        Ok(Some((request, len)))
    } else {
        Ok(None)
    }
}
