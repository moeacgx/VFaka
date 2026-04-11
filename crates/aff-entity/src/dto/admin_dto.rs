use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub admin: AdminInfo,
}

#[derive(Debug, Serialize)]
pub struct AdminInfo {
    pub id: i32,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryDto {
    pub name: String,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryDto {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ImportCardsDto {
    pub product_id: i32,
    pub variant_id: Option<i32>,
    pub cards: String,
}
