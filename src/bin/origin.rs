use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    str::{self, FromStr},
    string::ParseError,
};

#[derive(Debug)]
struct RequestLine {
    method: Option<String>,
    path: Option<String>,
    protocol: Option<String>,
}
impl RequestLine {
    fn method(&self) -> String {
        if let Some(method) = &self.method {
            method.to_string()
        } else {
            String::from("")
        }
    }
    fn path(&self) -> String {
        if let Some(path) = &self.path {
            path.to_string()
        } else {
            String::from("")
        }
    }
    // get item number
    fn get_resource_id(&self) -> String {
        let path = self.path();
        println!("path: {path}");
        let path_tokens: Vec<String> = path.split("/").map(|line| line.parse().unwrap()).collect();
        path_tokens[path_tokens.len() - 1].clone()
    }
}
impl FromStr for RequestLine {
    type Err = ParseError;
    fn from_str(msg: &str) -> Result<Self, Self::Err> {
        let mut msg_tokens = msg.split_ascii_whitespace();
        let method = match msg_tokens.next() {
            Some(token) => Some(String::from(token)),
            None => None,
        };
        let path = match msg_tokens.next() {
            Some(token) => Some(String::from(token)),
            None => None,
        };
        let protocol = match msg_tokens.next() {
            Some(token) => Some(String::from(token)),
            None => None,
        };

        Ok(Self {
            method: method,
            path: path,
            protocol: protocol,
        })
    }
}

fn main() {
    const ORIG_PORT: u16 = 8080;
    const ORIG_ADDR: &str = "127.0.0.1";
    let orig_addr_parsed = IpAddr::V4(ORIG_ADDR.parse::<Ipv4Addr>().unwrap());
    let endpoint = SocketAddr::new(orig_addr_parsed, ORIG_PORT);

    // create listener
    let listener = TcpListener::bind(endpoint).unwrap();
    println!("Listening at endpoint: {}", listener.local_addr().unwrap());

    // check listener for incoming connections/http requests
    for connection in listener.incoming() {
        // connection == stream
        let mut connection = connection.unwrap();

        // init the buffer
        let mut buffer = [0; 2_usize.pow(9)];

        // read to the buffer
        connection.read(&mut buffer).unwrap();

        // request lines
        let req_line = "";
        // str_request_line == string_request_line
        let str_request_line = if let Some(line) = str::from_utf8(&buffer).unwrap().lines().next() {
            line
        } else {
            println!("Error parsing request line");
            req_line
        };
        let req_line = RequestLine::from_str(str_request_line).unwrap();

        println!("Incoming request: {req_line:#?}");

        // build the logic to build responses from requests
        let html_res_str = build_response(req_line);

        println!("res to send: {:?}", html_res_str);

        // connection == stream
        // TODO: handle potential writing issues
        connection.write(html_res_str.as_bytes()).unwrap();

        // let connection = match connection_result {
        //     Ok(connection) => thread::spawn(move || handle_connection(connection)),
        //     _ => (),
        // };
    }
    // println!("Listening on port {ORIG_PORT}:  ${connection:?}");
}

fn build_response(req_line: RequestLine) -> String {
    let html_res_str: String;
    // status == order_status
    let status: String;

    println!("len is {}", req_line.get_resource_id().len());

    let cond_invalid_resource = req_line.get_resource_id().len() == 0;
    let cond_invalid_req = req_line.method() != "GET"
        || !req_line.path().starts_with("/status")
        || cond_invalid_resource;
    if cond_invalid_req {
        if cond_invalid_resource {
            // status == order_status
            status = format!("Invalid resource id");
        } else {
            // status == order_status
            status = format!("Not found");
        }
        html_res_str = format!(
            "{}\n{}\nContent-Length:{}\n\n{}",
            "HTTP/1.1 404 Not Found\n",
            "Content-Type: text/html",
            status.len(),
            status
        );
    } else {
        // status == order_status
        // get_resource_id == get_order_number
        status = format!(
            "{} {} is: Shipped\n",
            "Status for Item #",
            req_line.get_resource_id()
        );

        html_res_str = format!(
            "{} {} {}\n\n{}",
            "HTTP/1.1 200 OK\nContent-Type:",
            "text/html\nContent-Length:",
            status.len(),
            status
        );
    }
    html_res_str
}

// init connection
// match listener.accept() {
//     // Ok((sock, _)) => println!("matched: {sock:?}"),
//     Ok((sock, _)) => handle_connection(sock),
//     Err(e) => panic!("Error match: {}", e),
// }
// println!("endddd1");
// let connection = listener.accept();
// .map_err(|error| println!("{error:?}"))
// .unwrap_err();
