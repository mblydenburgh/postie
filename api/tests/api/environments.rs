use api::PostieApi;

use crate::helpers::spawn_test_app;

#[test]
fn can_parse_environment_files() {
    let app = spawn_test_app();
    let test_environment = app.load_test_environment();
    let parsed = PostieApi::parse_environment(test_environment);

    assert_eq!(parsed.id, "3ab687f6-4d2d-4d15-b129-962721cd5c5a");
    assert_eq!(parsed.name, "Local - QP External Partner");
    let environment_values = parsed.values;
    if let Some(vals) = environment_values {
        assert_eq!(vals.len(), 1);
        let first_val = vals.first().unwrap();
        assert_eq!(first_val.key, "HOST_URL");
        assert_eq!(first_val.value, "http://localhost:3000/external-partner/v1");
        assert_eq!(first_val.r#type, "default");
        assert_eq!(first_val.enabled, true);
    } 
    
}
