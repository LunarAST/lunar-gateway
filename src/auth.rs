use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use worker::*;

pub fn verify_token(token: &str, public_key_hex: &str) -> Result<bool> {
    // Token format: header.payload.signature (base64url encoded)
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Ok(false);
    }

    // Decode public key from hex
    let public_key_bytes = hex::decode(public_key_hex)
        .map_err(|_| Error::RustError("Invalid public key hex".into()))?;
    let public_key = VerifyingKey::from_bytes(
        &public_key_bytes[..32].try_into()
            .map_err(|_| Error::RustError("Invalid public key length".into()))?
    ).map_err(|_| Error::RustError("Invalid public key".into()))?;

    // Decode signature (base64url -> bytes)
    let signature_bytes = base64_url_decode(parts[2])
        .ok_or_else(|| Error::RustError("Invalid signature encoding".into()))?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| Error::RustError("Invalid signature".into()))?;

    // Reconstruct signing message (header.payload)
    let message = format!("{}.{}", parts[0], parts[1]);
    Ok(public_key.verify(message.as_bytes(), &signature).is_ok())
}

fn base64_url_decode(input: &str) -> Option<Vec<u8>> {
    // Convert standard base64url to base64
    let mut buf = input.replace('-', "+").replace('_', "/");
    while buf.len() % 4 != 0 {
        buf.push('=');
    }
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, buf).ok()
}
