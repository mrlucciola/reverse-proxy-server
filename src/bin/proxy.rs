pub mod http;
use crate::http::request::parse_request;
use std::{
    env,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    process::exit,
    thread::{self, JoinHandle},
};
// mod cache;

fn new_endpoint_str(addr: &str, port: u16) -> String {
    let addr_parsed = IpAddr::V4(addr.parse::<Ipv4Addr>().unwrap());
    let endpoint = SocketAddr::new(addr_parsed, port);

    endpoint.to_string()
}

fn main() {
    let origin_endpoint = &new_endpoint_str("127.0.0.1", 8080);
    let proxy_endpoint = new_endpoint_str("127.0.0.1", 8081);

    // start the socket server at `proxy endpoint`
    let proxy_listener: TcpListener;
    if let Ok(proxy) = TcpListener::bind(proxy_endpoint) {
        proxy_listener = proxy;
        let port = proxy_listener.local_addr().unwrap().port();
        let addr = proxy_listener.local_addr().unwrap().ip();

        // handle error if no origin
        if let Err(_err) = TcpStream::connect(origin_endpoint) {
            println!("Error: Please re-start the origin server");
            exit(1);
        }
        println!("Running at endpoint: {addr}:{port}");

        // check threads for connections
        let mut thread_handles: Vec<JoinHandle<()>> = Vec::new();
        for proxy_stream in proxy_listener.incoming() {
            let mut proxy_connection = proxy_stream.expect("Connection error");

            // establish new tcp connection to origin
            let mut origin_connection = TcpStream::connect(origin_endpoint)
                .expect("Error with origin server, please re-connect");

            // spawn a new thread
            let handle = thread::spawn(move || {
                handle_connection(&mut proxy_connection, &mut origin_connection)
            });
            thread_handles.push(handle);
        }

        // manage threads
        for handle in thread_handles {
            handle.join().expect("Unable to join child thread");
        }
    } else {
        eprintln!("Unable to bind to specified proxy port");
        exit(1);
    }
}

/*  */
fn handle_connection(proxy_connection: &mut TcpStream, origin_connection: &mut TcpStream) {
    // init buffers
    let mut in_buffer: Vec<u8> = vec![0; 2_usize.pow(8)];
    let mut out_buffer: Vec<u8> = vec![0; 2_usize.pow(8)];

    // 1) read client req
    if let Err(err) = proxy_connection.read(&mut in_buffer) {
        println!("Error: incoming proxy connection: {}", err);
    } else {
        println!(
            "1) incoming client req: {}",
            String::from_utf8_lossy(&in_buffer)
        );
    }
    // serde::Deserialize::deserialize(deserializer);
    println!("in_buffer: {:?}", String::from_utf8_lossy(&in_buffer));

    // 2) write to origin
    let _orig_fwd = origin_connection
        .write(&mut in_buffer)
        .expect("Error writing to origin");
    println!("2) Forwarding req to origin");

    // 3) read res from origin
    let _orig_recv = origin_connection.read(&mut out_buffer).unwrap();
    println!(
        "3) Received res from origin: {}",
        String::from_utf8_lossy(&out_buffer)
    );

    // 4) write res to proxy
    let _proxy_fwd = proxy_connection
        .write(&mut out_buffer)
        .expect("Error: writing to proxy");
    println!("4) Forwarding res to client");
}
