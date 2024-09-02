use rouille::{Request, Response};
use std::path::PathBuf;
use std::sync::Arc;

use crate::http::controller::{get, index, store};
use crate::http::intercept::intercept_request; 
use crate::http::intercept::intercept_response;

pub fn handle_request(request: &Request, dir: &Arc<PathBuf>) -> Response {
    intercept_request(request);

    let response = match request.method() {
        "POST" if request.url() == "/" => store(request, dir),
        "GET" if request.url() == "/" => index(request, dir),
        _ => get(request, dir),
    };

    let response = intercept_response(response);

    response
}
