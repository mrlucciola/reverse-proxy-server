// libs
use std::{
    io,
    net::{TcpListener, TcpStream},
    process::exit,
    thread::{self, JoinHandle},
};
// local
pub mod http_utils;
use http_utils::connection::{handle_connection, new_endpoint_str};

fn handle_incoming_client_stream(
    client_proxy_stream_res: Result<TcpStream, io::Error>,
    origin_endpoint: String,
) -> JoinHandle<()> {
    let mut client_proxy_connection = client_proxy_stream_res.expect("Connection error");

    // spawn and return new thread
    let new_thread = thread::spawn(move || {
        if let Err(err) = handle_connection(&mut client_proxy_connection, &origin_endpoint) {
            eprintln!("Error- Handling new client connection: {:?}", err);
        };
    });

    // return the thread
    new_thread
}

fn main() {
    let origin_endpoint = new_endpoint_str("127.0.0.1", 8080);
    let proxy_endpoint = new_endpoint_str("127.0.0.1", 8081);

    // start the socket server at `proxy endpoint`
    let proxy_listener_result = TcpListener::bind(proxy_endpoint);
    if let Err(err) = proxy_listener_result {
        eprintln!("Unable to bind to specified proxy port: {}", err);
        exit(1);
    }

    let proxy_listener = proxy_listener_result.unwrap();
    let port = proxy_listener.local_addr().unwrap().port();
    let addr = proxy_listener.local_addr().unwrap().ip();

    println!("Running at endpoint: {addr}:{port}");

    // check/handle threads for connections
    let mut thread_handles: Vec<JoinHandle<()>> = Vec::new();
    for client_proxy_stream in proxy_listener.incoming() {
        let handle = handle_incoming_client_stream(client_proxy_stream, origin_endpoint.clone());

        // add to 'thread pool'
        thread_handles.push(handle);
        println!("\nend of connection loop\n");
    }

    // manage threads
    for handle in thread_handles {
        handle.join().expect("Unable to join child thread");
    }
}

// const MAX_NUM_HEADERS: usize = 32;

fn write_to_origin() {}
/*  */
