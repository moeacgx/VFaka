use chrono::Utc;
use rand::Rng;

/// Generate a unique order number: timestamp + random alphanumeric
pub fn generate_order_no() -> String {
    let ts = Utc::now().format("%Y%m%d%H%M%S");
    let rand_part: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect::<String>()
        .to_uppercase();
    format!("{}{}", ts, rand_part)
}

/// Generate a short AFF code (6 chars, uppercase alphanumeric)
pub fn generate_aff_code() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_no_length() {
        let no = generate_order_no();
        assert_eq!(no.len(), 22); // 14 (timestamp) + 8 (random)
    }

    #[test]
    fn test_aff_code_length() {
        let code = generate_aff_code();
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn test_uniqueness() {
        let a = generate_order_no();
        let b = generate_order_no();
        assert_ne!(a, b);
    }
}
