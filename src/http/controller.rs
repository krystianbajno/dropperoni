use ammonia::clean;
use rouille::input::multipart::{get_multipart_input, MultipartError};
use rouille::{Request, Response};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
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
            let sanitized_filename = clean(&filename);
            let filepath = dir.join(sanitized_filename.to_string());

            let mut file = match File::create(&filepath) {
                Ok(file) => file,
                Err(_) => return Response::text("Failed to create file").with_status_code(500),
            };

            let mut buffer = [0u8; 4096];
            while let Ok(bytes_read) = field.data.read(&mut buffer) {
                if bytes_read == 0 {
                    break;
                }

                if let Err(_) = file.write_all(&buffer[..bytes_read]) {
                    return Response::text("Failed to write file").with_status_code(500);
                }
            }

            return index(request, dir);
        }
    }

    Response::text("No file uploaded").with_status_code(400)
}
