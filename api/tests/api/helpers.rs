use api::{initialize_db, PostieApi};
use wiremock::MockServer;

pub struct TestApp {
  pub app: PostieApi,
  pub test_server: MockServer,
}

impl TestApp {
  pub async fn new() -> Self {
    let mock_server = MockServer::start().await;
    let mock_client = reqwest::Client::builder().build().unwrap();
    let db_connection = initialize_db().await.unwrap();
    Self {
      test_server: mock_server,
      app: PostieApi {
        client: mock_client,
        db_connection,
        environment: None,
        collection: Some("test_collection.json".to_string()),
      },
    }
  }

  pub fn load_test_collection(&self) -> &str {
    include_str!("test_collection.json")
  }

  pub fn load_test_environment(&self) -> &str {
    include_str!("test_environment.json")
  }
}

pub async fn spawn_test_app() -> TestApp {
  let app = TestApp::new();
  app.await
}
