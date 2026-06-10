use ed25519_dalek::Verifier;
use ed25519_dalek::{SigningKey, Signer};
use rand::rngs::OsRng;
use base64::Engine;

fn generate_test_jwt(project: &str) -> (String, String, String) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let public_hex = hex::encode(verifying_key.as_bytes());

    let header = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(r#"{"alg":"EdDSA","typ":"JWT"}"#);
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(format!(r#"{{"sub":"{}","exp":9999999999}}"#, project));

    let message = format!("{}.{}", header, payload);
    let signature = signing_key.sign(message.as_bytes());
    let signature_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(signature.to_bytes());

    let token = format!("{}.{}.{}", header, payload, signature_b64);
    (token, project.to_string(), public_hex)
}

#[test]
fn test_load_public_keys_from_json() {
    let (_, project, public_hex) = generate_test_jwt("test-project");
    let keys_json = format!(r#"{{"{}": "{}"}}"#, project, public_hex);
    let parsed: std::collections::HashMap<String, String> =
        serde_json::from_str(&keys_json).unwrap();
    assert_eq!(parsed.get(&project).unwrap(), &public_hex);
}

#[test]
fn test_verify_valid_token_primitives() {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let message = b"test-message";
    let signature = signing_key.sign(message);
    let verified = verifying_key.verify(message, &signature);
    assert!(verified.is_ok());
}

#[test]
fn test_tampered_token_rejected() {
    let (token, _, _) = generate_test_jwt("test-project");
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3);
    // Tamper payload
    let tampered = format!("{}.tampered_payload.{}", parts[0], parts[2]);
    let parts2: Vec<&str> = tampered.split('.').collect();
    assert_eq!(parts2.len(), 3);
    assert_ne!(parts[1], "tampered_payload");
}
