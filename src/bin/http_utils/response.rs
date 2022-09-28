// libs
use http::Response;
use httparse;
use std::{
    io::{Read, Write},
    net::TcpStream,
};
// local
pub use super::{
    connection::check_body_len,
    errors::{fmt_error, ResponseError, Result},
};

fn check_for_complete_request(res_status: httparse::Status<usize>) -> Option<usize> {
    if let httparse::Status::Complete(res_len) = res_status {
        Some(res_len)
    } else {
        None
    }
}

fn parse_res_from_origin(buffer: &[u8]) -> Result<Option<(http::Response<Vec<u8>>, usize)>> {
    println!("3.0) start: parse_res_from_origin");
    // init headers
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut res_init = httparse::Response::new(&mut headers);

    println!("3.1) parsing response");
    // parse the response into res_init, get status
    let res_status = res_init
        .parse(buffer)
        .or_else(|err| Err(fmt_error(ResponseError::MalformedResponse(err), "")))?;

    println!("3.2) parsed: res_status = {res_status:?}");
    // if this is a complete request, build and return response
    let res_len = match check_for_complete_request(res_status) {
        Some(len) => len,
        None => return Err(failure::err_msg("Buffer overflow")),
    };

    println!("3.3) res_len: {res_len}");
    // init the response builder
    let mut res = http::Response::builder()
        .status(res_init.code.unwrap())
        .version(http::Version::HTTP_11);
    println!("3.4) res builder: {res:?}");

    // add headers to the response builder
    for header in res_init.headers {
        res = res.header(header.name, header.value);
    }
    println!("3.5) headers: done");

    // init the response body
    let res: Response<Vec<u8>> = res.body(Vec::new()).unwrap();
    println!("3.6) response body: built");

    Ok(Some((res, res_len)))
}

/// For Proxy: read the response from origin
pub fn read_res_from_origin(proxy_origin_stream: &mut TcpStream) -> Result<Response<Vec<u8>>> {
    // init response buffer
    let mut res_buffer = [0_u8; 2_usize.pow(10) * 8]; // 8 kb buffer
    let mut bytes_read = 0;

    loop {
        println!("2b.1) reading new bytes - bytes read: {bytes_read}");
        // read incoming stream and write bytes into the buffer
        let new_bytes = proxy_origin_stream
            .read(&mut res_buffer[bytes_read..])
            .or_else(|err| {
                Err(fmt_error(
                    ResponseError::ConnectionError(err),
                    "Error reading new byes:",
                ))
            })?;

        println!("2b.2) handle incomplete respoisen - new bytes: {new_bytes}");
        // handle incomplete response
        if new_bytes == 0 {
            println!("2b.2.x) new bytes == 0 - {new_bytes}");
            break;
        }
        bytes_read += new_bytes;
        println!("2b.3) bytes read = {}", &bytes_read);
    }

    // check for valid response
    let parsed_res_option = parse_res_from_origin(&res_buffer[..bytes_read])?;
    println!("2b.4.a) parsed_res_option");
    if let None = parsed_res_option {
        return Err(failure::err_msg("Incomplete response - returned none"));
    }
    println!("2b.4.b) parsed_res_option - no failure!");

    let (mut parsed_res, headers_len) = parsed_res_option.unwrap();
    println!("2b.5) parsed_res: x  headers_len: {}", headers_len);

    // return the remainder of the response body (without the headers)
    parsed_res
        .body_mut()
        .extend_from_slice(&res_buffer[headers_len..bytes_read]);

    println!("2b.6) parsed_res (after extending): \n{:?}", parsed_res);

    println!("Success: response from origin = read");
    return Ok(parsed_res);
}

/// build the response object to send to the client
pub fn write_response_to_client(
    stream: &mut TcpStream,
    res: http::Response<Vec<u8>>,
) -> Result<()> {
    let data_to_forward = format!(
        "{:?} {} {}",
        res.version(),
        res.status().as_str(),
        res.status().canonical_reason().unwrap_or("")
    );
    stream.write(&data_to_forward.into_bytes())?;
    stream.write(b"\r\n")?;

    for (header_name, header_value) in res.headers() {
        stream.write(&format!("{}: ", header_name).as_bytes())?;
        stream.write(header_value.as_bytes())?;
        stream.write(b"\r\n")?;
    }
    stream.write(b"\r\n")?;

    if res.body().len() > 0 {
        stream.write(res.body())?;
    }

    Ok(())
}
