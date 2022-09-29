// imports
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

// local
use super::constants::*;

pub fn get_proxy_addr() -> String {
    new_endpoint_str(PROXY_ADDR, PROXY_PORT)
}
pub fn get_origin_addr() -> String {
    new_endpoint_str(ORIGIN_ADDR, ORIGIN_PORT)
}

pub fn new_endpoint_str(addr: &str, port: u16) -> String {
    let addr_parsed = IpAddr::V4(addr.parse::<Ipv4Addr>().unwrap());
    let endpoint = SocketAddr::new(addr_parsed, port);

    endpoint.to_string()
}

pub type Result<T> = std::result::Result<T, failure::Error>;
