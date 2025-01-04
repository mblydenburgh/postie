use crate::domain::request::HttpMethod;
use reqwest::Method;

pub fn convert_http_method(input: HttpMethod) -> Method {
    match input {
        HttpMethod::GET => Method::GET,
        HttpMethod::POST => Method::POST,
        HttpMethod::PUT => Method::PUT,
        HttpMethod::PATCH => Method::PATCH,
        HttpMethod::DELETE => Method::DELETE,
        HttpMethod::HEAD => Method::HEAD,
        HttpMethod::OPTIONS => Method::OPTIONS,
    }
}
