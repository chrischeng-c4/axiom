// SPEC-MANAGED: .aw/tech-design/projects/httpkit-demo/create-user-request.md#tests
// CODEGEN-BEGIN

use httpkit_demo::create_user_request::CreateUserRequest;

#[test]
fn valid_payload_constructs() {
    let r =
        CreateUserRequest::new("alice".to_string(), "alice@example.com".to_string(), 30).unwrap();
    assert_eq!(r.name, "alice");
    assert_eq!(r.email, "alice@example.com");
    assert_eq!(r.age, 30);
}

#[test]
fn empty_name_rejected_by_min_length() {
    let result = CreateUserRequest::new("".to_string(), "alice@example.com".to_string(), 30);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("non-empty"));
}

#[test]
fn long_name_rejected_by_max_length() {
    let long = "a".repeat(65);
    let result = CreateUserRequest::new(long, "alice@example.com".to_string(), 30);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too long"));
}

#[test]
fn email_without_at_rejected_by_expr_rule() {
    let result =
        CreateUserRequest::new("alice".to_string(), "alice-at-example.com".to_string(), 30);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("@"));
}

#[test]
fn out_of_range_age_rejected() {
    let result = CreateUserRequest::new("alice".to_string(), "alice@example.com".to_string(), 999);
    assert!(result.is_err());
}

#[test]
fn serde_json_roundtrip() {
    let r =
        CreateUserRequest::new("alice".to_string(), "alice@example.com".to_string(), 30).unwrap();
    let json = serde_json::to_string(&r).unwrap();
    let parsed: CreateUserRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(r, parsed);
    assert!(json.contains("alice@example.com"));
}
// CODEGEN-END
