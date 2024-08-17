use rouille::{Request, Response};
use std::path::PathBuf;
use std::sync::Arc;

use crate::controller::{get, index, store};

pub fn handle_request(request: &Request, dir: &Arc<PathBuf>) -> Response {
    println!("[{}] - {} {} \n{:#?}", request.remote_addr(), request.method(), request.raw_url(), request.headers());

    match request.method() {
        "POST" if request.url() == "/" => { 
            store(request, dir)
        },
        "GET" if request.url() == "/" => index(dir),
        _ => get(request, dir),
    }
}