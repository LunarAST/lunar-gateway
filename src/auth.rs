use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use std::collections::HashMap;
use worker::*;

/// Load public keys from the LUNAR_PUBLIC_KEYS environment variable.
fn load_public_keys(env: &Env) -> HashMap<String, VerifyingKey> {
    let mut map = HashMap::new();
    if let Ok(raw) = env.var("LUNAR_PUBLIC_KEYS") {
        if let Ok(parsed) = serde_json::from_str::<HashMap<String, String>>(&raw.to_string()) {
            for (project, hex_key) in parsed {
                if let Ok(bytes) = hex::decode(&hex_key) {
                    if let Ok(key) = VerifyingKey::from_bytes(&bytes[..32].try_into().unwrap_or_default()) {
                        map.insert(project, key);
                    }
                }
            }
        }
    }
    map
}

/// Verify a JWT token against the public key for a given project.
pub fn verify_token(token: &str, project: &str, env: &Env) -> Result<bool> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Ok(false);
    }

    let payload_json = base64_url_decode(parts[1])
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default();
    let sub_matches = payload_json.contains(&format!("\"sub\":\"{}\"", project));
    if !sub_matches {
        return Ok(false);
    }

    let public_keys = load_public_keys(env);
    let public_key = match public_keys.get(project) {
        Some(k) => k,
        None => return Ok(false),
    };

    let signature_bytes = base64_url_decode(parts[2])
        .ok_or_else(|| Error::RustError("Invalid signature encoding".into()))?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| Error::RustError("Invalid signature".into()))?;

    let message = format!("{}.{}", parts[0], parts[1]);
    Ok(public_key.verify(message.as_bytes(), &signature).is_ok())
}

fn base64_url_decode(input: &str) -> Option<Vec<u8>> {
    let mut buf = input.replace('-', "+").replace('_', "/");
    while buf.len() % 4 != 0 {
        buf.push('=');
    }
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, buf).ok()
}
