use crate::domain::response::{Response, ResponseData};
use anyhow;
use reqwest::StatusCode;
use serde_json::json;

pub fn build_response(
    res_type: &str,
    res_status: StatusCode,
    res_text: String,
) -> anyhow::Result<Response> {
    if !res_type.starts_with("application/json")
        && !res_type.starts_with("text/plain")
        && !res_type.starts_with("text/html")
        && !res_type.starts_with("application/xml")
        && !res_type.starts_with("text/xml")
    {
        // I couldn't figure out how to safely throw an error so I'm just returning this for now
        println!(
                        "expected application/json, application/xml, text/xml, text/html, or text/plain, got {}",
                        res_type
                    );
        return Ok(Response {
            data: ResponseData::JSON(
                json!({"err": println!("unsupported response type {}!", res_type)}),
            ),
            status: res_status.to_string(),
        });
    }
    let res_data = match res_type {
        "application/json" => {
            let res_json = serde_json::from_str(&res_text)?;
            ResponseData::JSON(res_json)
        }
        "application/xml" => {
            // TODO - validate xml, currently trying to use serder_xml_rs::from_str()
            // fails
            ResponseData::XML(res_text)
        }
        "text/plain" => ResponseData::TEXT(res_text),
        "text/html" => ResponseData::TEXT(res_text),
        "text/xml" => {
            // TODO - validate xml, currently trying to use serder_xml_rs::from_str()
            // fails
            ResponseData::XML(res_text)
        }
        _ => ResponseData::UNKNOWN("".into()),
    };
    Ok(Response {
        data: res_data,
        status: res_status.to_string(),
    })
}
