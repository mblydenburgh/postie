use api::{
  domain::environment::{EnvironmentFile, EnvironmentValue},
  PostieApi,
};
use reqwest::Url;
use wiremock::Match;

// wiremock matches dont include a raw url match? i think
// i only saw path mainly. this is used to validate that a Url
// matches a specified assertion url.
pub struct MockUrlMatcher(String);
impl Match for MockUrlMatcher {
  fn matches(&self, request: &wiremock::Request) -> bool {
    if request.url == Url::parse(&self.0).unwrap() {
      true
    } else {
      false
    }
  }
}

#[test]
fn env_var_substitution_applies_correctly_in_urls() {
  let environment = EnvironmentFile {
    id: String::from("id"),
    name: String::from("some environment"),
    values: Some(vec![EnvironmentValue {
      key: String::from("HOST_URL"),
      value: String::from("https://httpbin.org"),
      r#type: String::from("default"),
      enabled: true,
    }]),
  };
  let raw_url = String::from("{{HOST_URL}}/json");
  let converted_url = PostieApi::substitute_variables_in_url(&environment, raw_url);
  assert_eq!(converted_url, "https://httpbin.org/json");
}

#[test]
fn returns_base_url_if_env_vars_dont_exist() {
  let environment = EnvironmentFile {
    id: String::from("id"),
    name: String::from("some environment"),
    values: None,
  };
  let raw_url = String::from("{{BOGUS}}/json");
  let converted_url = PostieApi::substitute_variables_in_url(&environment, raw_url);
  assert_eq!(converted_url, "{{BOGUS}}/json");
}

// TODO - figure out how to make this test work to validate the substitution when it is a private
// method. Goal is to validate the url the reqest client is using.
//#[tokio::test]
//async fn it_substitutes_env_vars_into_the_url_and_returns_200() {
//    let test_app = spawn_test_app().await;
//    let environment = EnvironmentFile {
//        id: String::from("id"),
//        name: String::from("some environment"),
//        values: Some(vec![EnvironmentValue {
//            key: String::from("HOST_URL"),
//            value: String::from("https://httpbin.org"),
//            r#type: String::from("default"),
//            enabled: true,
//        }]),
//    };
//    let raw_url = String::from("{{HOST_URL}}/json");
//    let input = api::HttpRequest {
//        id: Uuid::new_v4(),
//        name: None,
//        method: api::HttpMethod::GET,
//        url: raw_url,
//        headers: None,
//        body: None,
//        environment: Some(environment),
//    };
//    // Given a request to the given url, return a mock 200. This should only work if substitution
//    // works correctly
//    Mock::given(method("GET"))
//        .and(MockUrlMatcher(String::from("https://httpbin.org/json").try_into().unwrap()))
//        .respond_with(ResponseTemplate::new(200))
//        .expect(1)
//        .mount(&test_app.test_server)
//        .await;
//
//    let _ = PostieApi::make_request(&test_app.app,input);
//}
