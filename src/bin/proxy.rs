// libs
use std::{
    net::TcpListener,
    process::exit,
    sync::Arc,
    thread::{self, JoinHandle},
};
// local
use tcp_proxy::{
    cache_utils::cache::HTTPCache,
    http_utils::{
        connection::handle_client_proxy_connection,
        formatting::{get_proxy_addr, Result},
    },
};

/// TODO: incomplete. Handle all error cases with appropriate error messages
/// And HTTP response status
fn write_error_response_to_client(e: failure::Error) -> Result<()> {
    eprintln!("WRITING ERROR RESPONSE TO CLIENT: \n_______\n{e}\n_______\n");

    Ok(())
}

fn main() {
    let proxy_listener = match TcpListener::bind(&get_proxy_addr()) {
        Ok(pl) => {
            println!("Running at endpoint: {}", pl.local_addr().unwrap());

            pl
        }
        Err(e) => {
            eprintln!("Unable to bind to specified proxy port: {}", e);
            exit(1);
        }
    };

    // 0.2) init cache
    let cache_arc_rw = Arc::from(HTTPCache::new());

    // 0.3) init thread pool
    let mut thread_handles: Vec<JoinHandle<()>> = Vec::new();

    // 1) handle incoming connections
    for client_connection in proxy_listener.incoming() {
        println!("\nIncoming request: ");
        let cache = Arc::clone(&cache_arc_rw);

        // init the stream
        let client_proxy_stream = match client_connection {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error: handling connection - {}", e);
                continue;
            }
        };

        let handle = thread::spawn(move || {
            let t = chrono::offset::Local::now();
            println!("Thread-req from: {:?} {t}", client_proxy_stream.peer_addr());

            // handle errors during connection
            match handle_client_proxy_connection(client_proxy_stream, &Arc::clone(&cache)) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("error handling client connection: {}", e)
                }
            };
        });

        // 3) push handle
        thread_handles.push(handle);

        println!("\n\nEnd of connection\n");
    }

    // manage threads
    for handle in thread_handles {
        handle.join().expect("Unable to join child thread");
    }
}
