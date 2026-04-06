use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::*;
use serde::{Deserialize, Serialize};

use aff_common::config::AppConfig;
use aff_common::error::{AppError, AppResult};
use aff_entity::entities::admin;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

pub async fn login(
    db: &DatabaseConnection,
    config: &AppConfig,
    username: &str,
    password: &str,
) -> AppResult<(admin::Model, String)> {
    let admin = admin::Entity::find()
        .filter(admin::Column::Username.eq(username))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    let valid = bcrypt::verify(password, &admin.password_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !valid {
        return Err(AppError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    let token = generate_token(&admin, config)?;
    Ok((admin, token))
}

pub fn verify_token(token: &str, secret: &str) -> AppResult<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims)
}

pub async fn create_admin(
    db: &DatabaseConnection,
    username: &str,
    password: &str,
    role: &str,
) -> AppResult<admin::Model> {
    let existing = admin::Entity::find()
        .filter(admin::Column::Username.eq(username))
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if existing.is_some() {
        return Err(AppError::Conflict(format!(
            "Admin '{}' already exists",
            username
        )));
    }

    let password_hash =
        bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| AppError::Internal(e.to_string()))?;

    let now = chrono::Utc::now();
    let model = admin::ActiveModel {
        username: Set(username.to_string()),
        password_hash: Set(password_hash),
        role: Set(role.to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    admin::Entity::insert(model)
        .exec_with_returning(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn list_admins(db: &DatabaseConnection) -> AppResult<Vec<admin::Model>> {
    admin::Entity::find()
        .order_by_asc(admin::Column::Id)
        .all(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn delete_admin(db: &DatabaseConnection, id: i32) -> AppResult<()> {
    let existing = admin::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Admin {} not found", id)))?;

    if existing.role == "super_admin" {
        let super_admin_count = admin::Entity::find()
            .filter(admin::Column::Role.eq("super_admin"))
            .count(db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if super_admin_count <= 1 {
            return Err(AppError::BadRequest(
                "Cannot delete the last super admin".to_string(),
            ));
        }
    }

    admin::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

fn generate_token(admin: &admin::Model, config: &AppConfig) -> AppResult<String> {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(config.jwt.expiration_hours as i64))
        .ok_or_else(|| AppError::Internal("Failed to compute token expiration".to_string()))?
        .timestamp() as usize;

    let claims = Claims {
        sub: admin.id,
        username: admin.username.clone(),
        role: admin.role.clone(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))
}
