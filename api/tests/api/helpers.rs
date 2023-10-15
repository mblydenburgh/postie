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
    // TODO unit test kept failing due to file not being found then i tried to read file
    // using this for now.
    pub fn load_test_collection(&self) -> &str {
        r#"
        {
            "info": {
                "name": "Test Collection",
                "description": "A collection for unit testing"
            },
            "item": [
                {
                    "name": "Request 1",
                    "request": {
                        "method": "GET",
                        "url": {
                            "raw": "http://localhost:3000"
                        }
                    }
                },
                {
                    "name": "Request 2",
                    "request": {
                        "method": "GET",
                        "url": {
                            "raw": "http://localhost:3000",
                            "path": ["foo"]
                        }
                    }
                }
            ],
            "auth": {
                "type": "bearer",
                "bearer": [
                    {
                        "key": "bearer",
                        "value": "some-token",
                        "type": "string"
                    }
                    
                ]
            }
        }
        "#
    }
}

pub fn spawn_test_app() -> TestApp {
    let app = TestApp::new();
    app
}
