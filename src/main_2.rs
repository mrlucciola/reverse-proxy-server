// use anyhow::{self, *};
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    str, thread, fmt::Result,
};

fn handle_connection(mut connection: TcpStream) -> Result<()> {
    println!("Connection request from: {connection:?}");

    // 4kb buffer
    let mut buffer = [0; 1024 * 4];

    // get amt of bytes
    // let num_bytes = match connection.read(&mut buffer) {
    //     Ok(num) => num,
    //     Err(_) => {
    //         println!("error reading bytes");
    //         return Result<()>;
    //     }
    // };
    let num_bytes = connection.read(&mut buffer)?;

    println!("\n# of bytes: {num_bytes}");

    // convert to string
    // let req = match str::from_utf8(&buffer) {
    //     Ok(output) => output,
    //     Err(_) => {
    //         println!("error converting buffer to string");
    //         return Result<()>;
    //     }
    // };
    let req = str::from_utf8(&buffer)?;
    println!("\nRequest details (below):\n\n{req}Request details (above):");

    // get the url from the request
    let url = req.split_whitespace().collect::<Vec<&str>>()[1];
    println!("url: {url}");

    // open the connection stream
    // let stream = match TcpStream::connect(url) {
    //     Ok(s) => s,
    //     Err(_) => {println!("error getting stream"); return ();}
    // };
    let stream = TcpStream::connect(url)?;

    println!("\ntcp stream: {stream:?}\n");
    return Ok(1);
}

fn main() {
    const PROXY_PORT: u16 = 7070;
    const PROXY_ADDR: &str = "127.0.0.1";
    let endpoint: String = format!("{}:{}", PROXY_ADDR, PROXY_PORT);

    // create listener
    let listener = TcpListener::bind(endpoint).unwrap();
    println!("Listening on port {PROXY_PORT}:  ${listener:?}");

    // check threads for connections
    // server.incoming() -> stream_result
    for connection_result in listener.incoming() {
        let connection = match connection_result {
            Ok(connection) => thread::spawn(move || handle_connection(connection)),
            _ => (),
        };
    }
    println!("here");

    println!("endddd");
    // println!("Listening on port {PROXY_PORT}:  ${connection:?}");
}
// fn handle_connection(tcp: TcpStream) {
//     println!("Opened connection: {:?}", tcp)
// }

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
