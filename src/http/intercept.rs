use rouille::{Request, Response};
use std::io::Read;
use url::form_urlencoded;
use colored::*;

pub fn intercept_request(request: &Request) {
    pretty_print_headers(request);

    match request.method() {
        "POST" => {
            match handle_post_request(request) {
                Ok(parsed_data) => println!("{}", format!("Parsed POST data:\n{}", parsed_data).cyan()),
                Err(err_msg) => eprintln!("{}", err_msg.red()),
            }
        },
        "GET" => {
            let parsed_query = handle_get_request(request);
            println!("{}", format!("Parsed GET query:\n{}", parsed_query).cyan());
        },
        _ => {}
    }
}

pub fn handle_post_request(request: &Request) -> Result<String, String> {
    let mut data = request.data().ok_or("No POST data available")?;
    
    let mut buf = Vec::new();
    data.read_to_end(&mut buf).map_err(|e| format!("Failed to read POST data: {}", e))?;
    
    let body_str = match String::from_utf8(buf.clone()) {
        Ok(body_str) => body_str,
        Err(_) => {
            return Err(format!("Received POST data (binary):\n{:?}", buf));
        }
    };
    
    let parsed_body: Vec<(String, String)> = form_urlencoded::parse(body_str.as_bytes()).into_owned().collect();
    
    let mut parsed_data = String::new();
    for (key, value) in parsed_body {
        parsed_data.push_str(&format!("{}: {}\n", key, value));
    }
    Ok(parsed_data)
}

pub fn handle_get_request(request: &Request) -> String {
    let query = request.raw_query_string();
    let parsed_query: Vec<(String, String)> = form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    let mut parsed_data = String::new();
    for (key, value) in parsed_query {
        parsed_data.push_str(&format!("{}: {}\n", key, value));
    }
    parsed_data
}

fn pretty_print_headers(request: &Request) {
    println!("{}", format!("[{}] - {} {}", request.remote_addr(), request.method(), request.raw_url()).cyan());
    println!("{}", "Headers:".cyan());
    for (key, value) in request.headers() {
        println!("{}", format!("  {}: {}", key, value).cyan());
    }
    println!();
}

pub fn intercept_response(response: Response) -> Response {
    println!("{}", format!("Response: {}", response.status_code).green());
    
    println!("{}", "Headers:".green());
    for (key, value) in &response.headers {
        println!("{}", format!("  {}: {}", key, value).green());
    }

    let (mut body_reader, body_size) = response.data.into_reader_and_size();

    let mut body_buf = Vec::new();
    if body_reader.read_to_end(&mut body_buf).is_ok() {
        if let Ok(body_str) = String::from_utf8(body_buf.clone()) {
            println!("{}", format!("Body ({} bytes):\n{}", body_size.unwrap_or(body_buf.len()), body_str).green());
        } else {
            println!("{}", format!("Body ({} bytes, binary):\n{:?}", body_size.unwrap_or(body_buf.len()), body_buf).green());
        }
    } else {
        println!("{}", "Failed to read the body.".red());
    }

    println!(); 

    let mut new_response = Response::from_data("application/octet-stream", body_buf);
    for (key, value) in &response.headers {
        new_response = new_response.with_additional_header(
            key.clone().into_owned(),
            value.clone().into_owned(), 
        );
    }

    new_response.with_status_code(response.status_code)
}
