use ammonia::clean;
use rouille::input::multipart::{get_multipart_input, MultipartError};
use rouille::{Request, Response};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::views::views::index_view;

pub fn index(request: &Request, dir: &PathBuf) -> Response {
    let mut files = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Response::html(index_view(files)),
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if !file_type.is_file() {
            continue;
        }

        if let Some(file_name) = entry.file_name().to_str() {
            let sanitized_file_name = clean(file_name);
            files.push(sanitized_file_name.to_string());
        }
    }

    Response::html(index_view(files))
}

fn is_within_dir(base: &Path, target: &Path) -> bool {
    match target.canonicalize() {
        Ok(canonical_target) => canonical_target.starts_with(base),
        Err(_) => false,
    }
}

pub fn get(request: &Request, dir: &PathBuf) -> Response {
    let filepath = dir.join(&request.url()[1..]);
    
    // Prevent path traversal by ensuring the final path is within the intended directory
    if !is_within_dir(dir, &filepath) {
        return Response::empty_404();
    }
    
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
            let sanitized_filename = clean(&filename);

            // Prevent path traversal by ensuring the final path is within the intended directory
            let filepath = dir.join(sanitized_filename.to_string());
            if !is_within_dir(dir, &filepath) {
                return Response::text("Invalid file path").with_status_code(400);
            }

            let mut file = File::create(filepath).unwrap();
            std::io::copy(&mut field.data, &mut file).unwrap();
            return index(request, dir);
        }
    }

    Response::text("No file uploaded").with_status_code(400)
}
