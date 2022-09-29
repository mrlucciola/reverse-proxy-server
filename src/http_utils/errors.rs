// TODO: clean up unused errors

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug)]
pub enum RequestError {
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
    ConnectionError(failure::Error),
    /// Cannot handle certain method
    InvalidMethod,
    MiscError(ResponseError),
}

#[derive(Debug)]
pub enum ResponseError {
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
    ResponseBodyError(ConnectionError),
    /// Encountered an I/O error when reading/writing a TcpStream
    ConnectionError(std::io::Error),
}

#[derive(Debug)]
pub enum ConnectionError {
    /// The Content-Length header is present, but does not contain a valid numeric value
    InvalidContentLength,
    /// The request body is bigger than MAX_BODY_SIZE
    BodySizeTooLarge,
    /// No headers in map
    EmptyHeaderValue,
    /// Error while parsing
    ParseError(failure::Error),
    /// Error while client and proxy connection open
    ClientProxyStream,
}

pub fn fmt_error<T>(e: T, msg: &str) -> failure::Error
where
    T: std::fmt::Debug,
{
    failure::err_msg(format!("{msg}: {:?}", e))
}
