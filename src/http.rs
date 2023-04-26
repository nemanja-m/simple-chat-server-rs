use log::{debug, error};
use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;
use std::net::TcpStream;
use urlencoding::decode;

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Post,
    None,
}

impl From<String> for HttpMethod {
    fn from(value: String) -> Self {
        match value.as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            _ => HttpMethod::None,
        }
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_uppercase())
    }
}

pub enum ContentType {
    ApplicationFormUrlEncoded,
    Html,
    Json,
    TextPlain,
}

impl From<String> for ContentType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "application/x-www-form-urlencoded" => ContentType::ApplicationFormUrlEncoded,
            other => panic!("Invalid content type {}", other),
        }
    }
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub content_type: Option<ContentType>,
    pub query_params: HashMap<String, String>,
}

impl HttpRequest {
    pub fn from(stream: &TcpStream) -> HttpRequest {
        let request = read_request(stream);
        debug!("{request}\n\n");

        let method = parse_method(&request);
        let path = parse_path(&request);
        let content_type = parse_content_type(&request);
        let query_params = match content_type {
            Some(ContentType::ApplicationFormUrlEncoded) => parse_form_url_encoded_params(&request),
            _ => HashMap::new(),
        };

        HttpRequest {
            method,
            path,
            content_type,
            query_params,
        }
    }

    pub fn route(&self) -> String {
        format!("{} {}", self.method, self.path)
    }
}

fn read_request(mut stream: &TcpStream) -> String {
    // Hard-coded request size is good enough for demo purposes.
    // Parsing will fail if request size is larger than buffer.
    let mut buffer = [0; 4096];

    // Read the request header
    let bytes_read = stream.read(&mut buffer).unwrap();

    if bytes_read == buffer.len() {
        panic!("Buffer overflow. Try increasing buffer size.");
    }

    // Append the header to the request
    let request = String::from_utf8_lossy(&buffer).to_string();

    request
}

fn parse_method(header: &String) -> HttpMethod {
    header
        .lines()
        .next()
        .unwrap()
        .split(' ')
        .next()
        .map(ToString::to_string)
        .map(HttpMethod::from)
        .unwrap()
}

fn parse_path(header: &String) -> String {
    let parts: Vec<_> = header.lines().next().unwrap().split(' ').collect();

    if parts.len() < 2 {
        error!("{}", header);
    }

    parts[1].to_string()
}

fn parse_content_type(header: &String) -> Option<ContentType> {
    header_value(header, "Content-Type").map(|value| ContentType::from(value))
}

fn parse_form_url_encoded_params(header: &String) -> HashMap<String, String> {
    let parts = header.split("\r\n\r\n");
    let encoded_params = parts.last().unwrap();

    encoded_params
        .split("&")
        .map(|pair| {
            let kv: Vec<_> = pair.split('=').collect();
            let key = kv[0].to_string();
            let value = decode(kv[1]).expect("UTF-8").to_string().replace("+", " ");
            (key, value)
        })
        .collect()
}

fn header_value(request_header: &String, key: &str) -> Option<String> {
    let pattern = format!("\r\n{}: ", key);

    match request_header.find(pattern.as_str()) {
        Some(index) => {
            let start = index + pattern.len();
            let end = request_header[start..].find('\r').unwrap() + start;
            let p = Some(request_header[start..end].to_string());
            p
        }
        None => None,
    }
}
