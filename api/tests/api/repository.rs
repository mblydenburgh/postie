use api::{domain::collection::Collection, PostieApi};
use serde_json::json;

use crate::helpers::{initialize_test_db, spawn_test_app};

#[tokio::test]
async fn can_delete_collection_request() {
  println!("can delete collection request test");
  let test_app = spawn_test_app().await;
  if let Ok(mut db) = initialize_test_db().await {
    println!("initialized test db");
    let collection = test_app.app.parse_collection(&test_json.to_string());
    let _ = db.save_collection(collection).await;
    let collections = db.get_all_collections().await;

    let _ = PostieApi::delete_collection_request(
      collections.unwrap()[0].info.id.clone(),
      "delete-me".into(),
    )
    .await;

    let expected_json = json!(
        {
          "name": "test collection",
          "item": [
            {
              "name": "folder",
              "item": [
                {
                  "name": "req1",
                  "request": {
                    "method": "GET",
                    "url": {
                      "raw": "https://httpbin.org/json"
                    },
                  }
                }
              ]
            }
          ]
        }
    );
    let actual = PostieApi::load_collections().await.unwrap()[0].clone();
    assert_eq!(
      serde_json::to_value::<Collection>(actual.clone()).expect("could not serialize json"),
      expected_json
    );
  };
}
