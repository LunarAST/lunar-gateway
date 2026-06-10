use lunar_gateway::auth::verify_token;

#[test]
fn test_invalid_token() {
    // An obviously invalid token should return false, not panic
    let result = verify_token("invalid", "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789");
    assert_eq!(result.unwrap(), false);
}
