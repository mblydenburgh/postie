use api::PostieApi;

use crate::helpers::spawn_test_app;

#[test]
fn can_parse_collection_with_all_fields() {
    let app = spawn_test_app();
    let test_collection = app.load_test_collection();
    let parsed = PostieApi::parse_collection(&test_collection);
    assert_eq!(parsed.info.name, "Test Collection");
    assert_eq!(
        parsed.info.description,
        Some(String::from("A collection for unit testing"))
    );
    assert_eq!(parsed.item.len(), 2);
    assert_eq!(parsed.item.first().unwrap().name, "Request 1");
    assert_eq!(parsed.item.first().unwrap().request.method, "GET");
    assert_eq!(
        parsed.item.first().unwrap().request.url.raw,
        "http://localhost:3000"
    );
    assert_eq!(parsed.item.get(1).unwrap().name, "Request 2");
    assert_eq!(parsed.item.get(1).unwrap().request.method, "GET");
    assert_eq!(
        parsed.item.get(1).unwrap().request.url.raw,
        "http://localhost:3000"
    );
    assert_eq!(
        parsed.item.get(1).unwrap().request.url.path,
        Some(vec![String::from("foo")])
    );
}
