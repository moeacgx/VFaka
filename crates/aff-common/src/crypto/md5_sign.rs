use md5::{Md5, Digest};
use std::collections::BTreeMap;

/// Sort params by ASCII key order, join as key=value&key=value, then append secret
/// This is the TokenPay signing method
pub fn build_sign_string_with_secret(params: &BTreeMap<String, String>, secret: &str) -> String {
    let param_str: String = params
        .iter()
        .filter(|(k, v)| !v.is_empty() && *k != "Signature")
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    format!("{}{}", param_str, secret)
}

/// Calculate MD5 hash of the input string (lowercase hex)
pub fn md5_hash(input: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// TokenPay signature: sort params + append secret + MD5
pub fn tokenpay_sign(params: &BTreeMap<String, String>, secret: &str) -> String {
    let sign_str = build_sign_string_with_secret(params, secret);
    md5_hash(&sign_str)
}

/// Verify TokenPay signature
pub fn tokenpay_verify(params: &BTreeMap<String, String>, signature: &str, secret: &str) -> bool {
    let computed = tokenpay_sign(params, secret);
    computed == signature
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenpay_sign() {
        // Example from TokenPay docs
        let mut params = BTreeMap::new();
        params.insert("ActualAmount".to_string(), "15".to_string());
        params.insert("Currency".to_string(), "TRX".to_string());
        params.insert("NotifyUrl".to_string(), "http://localhost:1011/pay/tokenpay/notify_url".to_string());
        params.insert("OrderUserKey".to_string(), "admin@qq.com".to_string());
        params.insert("OutOrderId".to_string(), "AJIHK72N34BR2CWG".to_string());
        params.insert("RedirectUrl".to_string(), "http://localhost:1011/pay/tokenpay/return_url?order_id=AJIHK72N34BR2CWG".to_string());

        let signature = tokenpay_sign(&params, "666");
        assert_eq!(signature, "e9765880db6081496456283678e70152");
    }

    #[test]
    fn test_tokenpay_verify() {
        let mut params = BTreeMap::new();
        params.insert("ActualAmount".to_string(), "15".to_string());
        params.insert("Currency".to_string(), "TRX".to_string());
        params.insert("NotifyUrl".to_string(), "http://localhost:1011/pay/tokenpay/notify_url".to_string());
        params.insert("OrderUserKey".to_string(), "admin@qq.com".to_string());
        params.insert("OutOrderId".to_string(), "AJIHK72N34BR2CWG".to_string());
        params.insert("RedirectUrl".to_string(), "http://localhost:1011/pay/tokenpay/return_url?order_id=AJIHK72N34BR2CWG".to_string());

        assert!(tokenpay_verify(&params, "e9765880db6081496456283678e70152", "666"));
        assert!(!tokenpay_verify(&params, "wrong_signature", "666"));
    }

    #[test]
    fn test_md5_hash() {
        let hash = md5_hash("hello");
        assert_eq!(hash, "5d41402abc4b2a76b9719d911017c592");
    }
}
