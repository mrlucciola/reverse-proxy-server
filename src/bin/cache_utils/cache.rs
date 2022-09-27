// use super::http::response;
// use super::{handler, utils};
use http::{Request, Response};
// use httparse;
use std::{
    fs::{create_dir_all, read, File},
    io::prelude::*,
    path::Path,
    sync::{Arc, Mutex},
};

pub struct HTTPCache {
    dir_path: String,
    file_locks: Vec<Arc<Mutex<()>>>,
}

impl HTTPCache {
    pub fn new(dir_path: &str) -> Self {
        create_dir_all(dir_path).unwrap();

        Self {
            dir_path: dir_path.to_string(),
            file_locks: vec![Arc::new(Mutex::new(())); 997],
        }
    }

    pub fn contains_entry(&self, req: &Request<Vec<u8>>) -> bool {
        let path = self.get_filepath_from_request(req);
        Path::new(&path).exists()
    }

    pub fn get_cached_response(&self, req: &Request<Vec<u8>>) -> Option<Response<Vec<u8>>> {
        if !Self::contains_entry(self, &req) {
            return None;
        }

        let filepath = self.get_filepath_from_request(req);
        if let Ok(res_bytes) = read(&filepath) {
            // if let Ok(Some((res, _))) = response::parse_response(&res_bytes) {
            //     return Some(res);
            // }
            eprintln!("Failed to parse response from cache file {}", &filepath);
            return None;
        }
        eprintln!("Failed to read response from cache file {}", &filepath);
        return None;
    }

    pub fn add_entry(&self, req: &Request<Vec<u8>>, _res: &Response<Vec<u8>>) {
        let filepath = Self::get_filepath_from_request(&self, req);
        let res_bytes = [0u8; 1000]; //utils::response_to_bytes(res);

        if let Ok(mut file_buf) = File::create(&filepath) {
            match file_buf.write_all(&res_bytes) {
                Ok(_) => println!("Wrote file {} to cache.", &filepath),
                Err(e) => eprintln!("Failed to write response to cache file: {}", e),
            }
        } else {
            eprintln!("Failed to create new cache file!");
        }
    }

    fn get_filepath_from_request(&self, _req: &Request<Vec<u8>>) -> String {
        let hashcode = ""; //utils::get_hashcode(req);
        format!("{}/{}", self.dir_path, hashcode)
    }
}
