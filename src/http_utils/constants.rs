///////////////////////////////////////////////
// http-utils

// request & response
/// Use for checking the content length of the incoming request
pub const SIZE_MAX_BODY: usize = 10000000;
/// Use for provisioning buffers
pub const SIZE_MAX_HEADERS: usize = 2_usize.pow(10) * 8; // 1024 * 8 = 8192
pub const AMT_MAX_HEADERS: usize = 64;
// main
pub const ORIGIN_PORT: u16 = 8080;
pub const ORIGIN_ADDR: &str = "127.0.0.1";
pub const PROXY_PORT: u16 = 8081;
pub const PROXY_ADDR: &str = "127.0.0.1";

// http-utils
///////////////////////////////////////////////

// cache-utils > cache
pub const CACHE_MAX_ENTRIES: usize = 1000;
/// needs to be int for date math
pub const CACHE_TTL_SEC: i64 = 30;
