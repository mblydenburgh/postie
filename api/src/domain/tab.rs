use uuid::Uuid;

use crate::HttpMethod;
use super::request::RequestHeaders;


#[derive(Clone, Debug, PartialEq)]
pub struct Tab {
    pub id: String,
    pub method: HttpMethod,
    pub url: String,
    pub req_body: String,
    pub req_headers: RequestHeaders,
    pub res_status: Option<String>,
    pub res_body: String,
    pub res_headers: RequestHeaders,
}
impl Default for Tab {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            url: "".into(),
            req_body: "".into(),
            req_headers: RequestHeaders(vec![]),
            method: HttpMethod::GET,
            res_status: None,
            res_body: "".into(),
            res_headers: RequestHeaders(vec![]),
        }
    }
}
