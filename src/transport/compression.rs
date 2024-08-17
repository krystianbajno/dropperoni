use std::error::Error;
use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;
use flate2::write::GzEncoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::prelude::*;

pub fn split_headers_and_body(response: &[u8]) -> (&[u8], &[u8]) {
    let split_pos = response.windows(4).position(|window| window == b"\r\n\r\n").map(|pos| pos + 4).unwrap_or(response.len());
    response.split_at(split_pos)
}

pub fn detect_encoding(headers: &[u8]) -> Option<&str> {
    let headers_str = std::str::from_utf8(headers).ok()?;
    if headers_str.contains("Content-Encoding: gzip") {
        Some("gzip")
    } else if headers_str.contains("Content-Encoding: deflate") {
        Some("deflate")
    } else {
        None
    }
}

pub fn decompress_body(body: &[u8], encoding: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut decoder: Box<dyn std::io::Read> = match encoding {
        "gzip" => Box::new(GzDecoder::new(body)),
        "deflate" => Box::new(ZlibDecoder::new(body)),
        _ => return Err("Unknown encoding".into()),
    };
    
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}

pub fn compress_body(body: &[u8], encoding: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut compressed_data = Vec::new();
    match encoding {
        "gzip" => {
            let mut encoder = GzEncoder::new(&mut compressed_data, Compression::default());
            encoder.write_all(body)?;
            encoder.finish()?;
        }
        "deflate" => {
            let mut encoder = ZlibEncoder::new(&mut compressed_data, Compression::default());
            encoder.write_all(body)?;
            encoder.finish()?;
        }
        _ => return Err("Unknown encoding".into()),
    }
    Ok(compressed_data)
}