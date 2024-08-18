static INDEX: &str = include_str!("../static/index.html");

pub fn index_view(files: Vec<String>) -> String {
    let mut file_list = String::new();
    for file in files {
        file_list.push_str(&format!("<li><a href=\"/{file}\">{file}</a></li>\n"));
    }
    INDEX.replace("<!-- File list will be dynamically inserted here -->", &file_list)
}
