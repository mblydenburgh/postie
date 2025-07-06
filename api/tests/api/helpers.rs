use api::{db::repository::PostieDb, PostieApi};
use sqlx::{sqlite::SqlitePoolOptions, Connection, SqliteConnection};
use wiremock::MockServer;

pub struct TestApp {
  pub app: PostieApi,
  pub test_server: MockServer,
}

impl TestApp {
  pub async fn new() -> Self {
    let mock_server = MockServer::start().await;
    let mock_client = reqwest::Client::builder().build().unwrap();
    let db = initialize_test_db().await.unwrap();
    Self {
      test_server: mock_server,
      app: PostieApi {
        client: mock_client,
        db,
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
pub async fn initialize_test_db() -> anyhow::Result<PostieDb> {
  println!("acquiring sqlite connection");
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 2 {
    println!("Usage: {} <sqlite db file>", args[0]);
    std::process::exit(1);
  }
  let pool = SqlitePoolOptions::new()
    .connect(":memory:")
    .await
    .expect("could not create test connection pool");
  //let connection = SqliteConnection::connect(":memory:").await?;
  println!("{:?} sqlite connection established", pool);
  sqlx::migrate!().run(&pool).await?;
  let db = PostieDb { pool };
  Ok(db)
}
