use api::PostieApi;

pub struct TestApp {
    pub app: PostieApi,
}

impl TestApp {
    pub fn new() -> Self {
        Self {
            app: PostieApi {
                environment: None,
                collection: Some("test_collection.json".to_string()),
            },
        }
    }

    pub fn load_test_collection(&self) -> &str {
        include_str!("test_collection.json")
    }
}

pub fn spawn_test_app() -> TestApp {
    let app = TestApp::new();
    app
}
