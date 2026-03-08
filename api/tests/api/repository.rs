use api::domain::collection::Collection;
use serde_json::json;

use crate::helpers::{initialize_test_db, spawn_test_app};

#[tokio::test]
async fn can_delete_collection_request() {
  println!("can delete collection request test");
  let mut test_app = spawn_test_app().await;
  if let Ok(mut db) = initialize_test_db().await {
    println!("initialized test db");
    let test_collection_file =
      std::fs::File::open("test_collection_2.json").expect("could not open test collection");
    let collection: Collection = serde_json::from_reader(test_collection_file).unwrap();
    let _ = db.save_collection(collection).await;
    let collections = db.get_all_collections().await;

    let _ = test_app
      .app
      .delete_collection_request(collections.unwrap()[0].info.id.clone(), "delete-me".into())
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
    let actual = test_app.app.load_collections().await.unwrap()[0].clone();
    assert_eq!(
      serde_json::to_value::<Collection>(actual.clone()).expect("could not serialize json"),
      expected_json
    );
  };
}
