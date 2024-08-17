use rouille::input::multipart::{get_multipart_input, MultipartError};
use rouille::{Request, Response};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Arc;

use crate::views::format_index_html;


pub fn index(dir: &PathBuf) -> Response {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            files.push(file_name.to_string());
                        }
                    }
                }
            }
        }
    }
    
    let body = format_index_html(files);
    Response::html(body)
}

pub fn get(request: &Request, dir: &PathBuf) -> Response {
    let filepath = dir.join(&request.url()[1..]);
    if filepath.is_file() {
        match fs::read(filepath) {
            Ok(content) => Response::from_data("application/octet-stream", content),
            Err(_) => Response::empty_404(),
        }
    } else {
        Response::empty_404()
    }
}

pub fn store(request: &Request, dir: &Arc<PathBuf>) -> Response {
    let mut multipart = match get_multipart_input(request) {
        Ok(multipart) => multipart,
        Err(MultipartError::WrongContentType) => {
            return Response::text("Invalid Content-Type").with_status_code(400)
        }
        Err(MultipartError::BodyAlreadyExtracted) => {
            return Response::text("Body already extracted").with_status_code(400)
        }
    };

    while let Some(mut field) = multipart.next() {
        if let Some(filename) = field.headers.filename.clone() {
            let filepath = dir.join(filename);
            let mut file = File::create(filepath).unwrap();
            std::io::copy(&mut field.data, &mut file).unwrap();
            return index(dir)
        }
    }

    Response::text("No file uploaded").with_status_code(400)
}