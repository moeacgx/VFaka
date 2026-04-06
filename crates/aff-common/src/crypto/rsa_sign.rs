use std::collections::BTreeMap;
use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey},
    Pkcs1v15Sign,
    RsaPrivateKey, RsaPublicKey,
};
use sha2::{Sha256, Digest};
use base64::Engine;

/// Sort params by ASCII key order, join as key=value&key=value, excluding empty values
pub fn build_sign_string(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .filter(|(k, v)| !v.is_empty() && *k != "sign" && *k != "sign_type")
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
}

/// Sign with SHA256WithRSA using merchant private key (PEM format)
pub fn rsa_sign(data: &str, private_key_pem: &str) -> Result<String, String> {
    let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)
        .map_err(|e| format!("Failed to parse private key: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let hash = hasher.finalize();

    let scheme = Pkcs1v15Sign::new::<Sha256>();
    let signature = private_key
        .sign(scheme, &hash)
        .map_err(|e| format!("Failed to sign: {}", e))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(&signature))
}

/// Verify SHA256WithRSA signature using platform public key (PEM format)
pub fn rsa_verify(data: &str, signature_b64: &str, public_key_pem: &str) -> Result<bool, String> {
    let public_key = RsaPublicKey::from_public_key_pem(public_key_pem)
        .map_err(|e| format!("Failed to parse public key: {}", e))?;

    let signature_bytes = base64::engine::general_purpose::STANDARD
        .decode(signature_b64)
        .map_err(|e| format!("Failed to decode signature: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let hash = hasher.finalize();

    let scheme = Pkcs1v15Sign::new::<Sha256>();
    match public_key.verify(scheme, &hash, &signature_bytes) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sign_string() {
        let mut params = BTreeMap::new();
        params.insert("pid".to_string(), "1001".to_string());
        params.insert("type".to_string(), "alipay".to_string());
        params.insert("money".to_string(), "1.00".to_string());
        params.insert("sign".to_string(), "should_be_excluded".to_string());

        let result = build_sign_string(&params);
        assert_eq!(result, "money=1.00&pid=1001&type=alipay");
    }

    #[test]
    fn test_build_sign_string_excludes_empty() {
        let mut params = BTreeMap::new();
        params.insert("a".to_string(), "1".to_string());
        params.insert("b".to_string(), "".to_string());
        params.insert("c".to_string(), "3".to_string());

        let result = build_sign_string(&params);
        assert_eq!(result, "a=1&c=3");
    }
}
