// libs
use std::{
    net::TcpListener,
    process::exit,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};
// local
pub mod cache_utils;
pub mod http_utils;
pub use cache_utils::cache::{Cache, HTTPCache};
pub use http_utils::{
    connection::{handle_client_proxy_connection, new_endpoint_str},
    errors::*,
};

fn write_error_response_to_client(e: failure::Error) -> Result<()> {
    eprintln!("WRITING ERROR RESPONSE TO CLIENT: \n_______\n{e}\n_______\n");

    Ok(())
}

fn main() {
    // 0.1) init proxy server/listener
    let origin_endpoint = new_endpoint_str("127.0.0.1", 8080);
    let proxy_endpoint = new_endpoint_str("127.0.0.1", 8081);
    let proxy_listener = match TcpListener::bind(&proxy_endpoint) {
        Ok(pl) => {
            let port = pl.local_addr().unwrap().port();
            let addr = pl.local_addr().unwrap().ip();
            println!("Running at endpoint: {addr}:{port}");

            pl
        }
        Err(e) => {
            eprintln!("Unable to bind to specified proxy port: {}", e);
            exit(1);
        }
    };

    // 0.2) init cache
    let cache_arc_rw = HTTPCache::new();

    // 0.3) init thread pool
    let mut thread_handles: Vec<JoinHandle<()>> = Vec::new();

    // 1) handle incoming connections
    for client_proxy_stream_result in proxy_listener.incoming() {
        let cache = cache_arc_rw.clone();
        let origin_endpoint_clone = origin_endpoint.clone();

        // TODO: Clean up nested match
        let handle = match client_proxy_stream_result {
            Ok(mut client_proxy_stream) => thread::spawn(move || {
                // return handle, otherwise show error
                if let Err(err) = handle_client_proxy_connection(
                    &mut client_proxy_stream,
                    origin_endpoint_clone,
                    cache,
                ) {
                    eprintln!("Error while handling request: {:?}", err);
                    // write the error http response to client, or log error
                    if let Err(e) = write_error_response_to_client(err) {
                        eprintln!("Error while writing to client: {:?}", e);
                    };
                };
            }),
            // if the client stream result returns error
            Err(e) => {
                eprintln!("Error: could not connect to client: {:?}", e);
                continue;
            }
        };

        // add handle to 'thread pool'
        thread_handles.push(handle);
        println!("\nend of connection loop\n");
    }

    // manage threads
    for handle in thread_handles {
        handle.join().expect("Unable to join child thread");
    }
}
