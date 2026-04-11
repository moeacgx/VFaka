pub mod services;
pub mod tasks;

/// Round a money value to 2 decimal places (banker's rounding avoided; floor-based for predictability).
pub fn round_money(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
