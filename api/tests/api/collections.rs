use api::{domain::collection::CollectionItemOrFolder, PostieApi};

use crate::helpers::spawn_test_app;

#[tokio::test]
async fn can_parse_collection_with_all_fields() {
  let app = spawn_test_app().await;
  let test_collection = app.load_test_collection();
  let parsed = PostieApi::parse_collection(&test_collection);
  assert_eq!(parsed.info.name, "qp-external-partner");
  assert_eq!(parsed.info.description, None);
  assert_eq!(parsed.item.len(), 1);
  let first_item = parsed.item.first().unwrap();
  match first_item {
    CollectionItemOrFolder::Item(_) => panic!("First item is folder, not Request"),
    CollectionItemOrFolder::Folder(folder) => {
      assert_eq!(folder.name, "/agent");
      let folder_item = folder.item.first().unwrap();
      match folder_item {
        CollectionItemOrFolder::Item(item) => {
          assert_eq!(item.name, "GET /");
          assert_eq!(item.request.method, "GET");
          assert_eq!(item.request.url.raw, "{{HOST_URL}}/agent");
          assert_eq!(item.request.url.path, Some(vec![String::from("agent")]));
          if let Some(h) = &item.request.header {
            let troux_header = h.first().unwrap();
            assert_eq!(troux_header.key, String::from("X-Troux-ID"));
            assert_eq!(troux_header.r#type, String::from("text"));
            assert_eq!(troux_header.value, String::from("{{TROUX_ID}}"));
          }
        }
        CollectionItemOrFolder::Folder(_) => panic!("Should be a request, not Folder"),
      };
    }
  }
  if let Some(auth) = parsed.auth {
    assert_eq!(auth.r#type, "oauth2");
    assert!(auth.bearer.is_none());
    assert!(auth.oauth2.is_some());
    if let Some(oauth2) = auth.oauth2 {
      assert_eq!(oauth2.len(), 10);
      assert_eq!(
        oauth2
          .get(0)
          .expect("missing field 'key' on oauth2 item")
          .key,
        String::from("audience")
      );
      assert_eq!(
        oauth2
          .get(1)
          .expect("missing field 'key' on oauth2 item")
          .key,
        String::from("tokenName")
      );
    }
  } else {
    panic!("Couldnt parse auth object")
  }
}
