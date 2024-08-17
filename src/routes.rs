use rouille::{Request, Response};
use std::path::PathBuf;
use std::sync::Arc;

use crate::{controller::{get, index, store}, views::format_index_html};

pub fn handle_request(request: &Request, dir: &Arc<PathBuf>) -> Response {
    println!("[{}] - {} {} \n{:#?}", request.remote_addr(), request.method(), request.raw_url(), request.headers());

    match request.method() {
        "POST" if request.url() == "/" => store(request, dir),
        "GET" if request.url() == "/" => {
            let files = index(dir);
            let body = format_index_html(files);
            Response::html(body)
        }
        _ => get(request, dir),
    }
}