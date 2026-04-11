use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::error::AppResult;
use aff_core::services::admin_service;

#[derive(Debug, Deserialize)]
pub struct CreateAdminDto {
    pub username: String,
    pub password: String,
    pub role: String,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/admins")
        .route("", web::get().to(list))
        .route("", web::post().to(create))
        .route("/{id}", web::delete().to(delete))
}

async fn list(db: web::Data<DatabaseConnection>) -> AppResult<HttpResponse> {
    let admins = admin_service::list_admins(&db).await?;
    Ok(HttpResponse::Ok().json(admins))
}

async fn create(
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateAdminDto>,
) -> AppResult<HttpResponse> {
    let dto = body.into_inner();
    if dto.role != "admin" && dto.role != "super_admin" {
        return Err(aff_common::error::AppError::BadRequest(
            "Role must be 'admin' or 'super_admin'".into(),
        ));
    }
    let admin = admin_service::create_admin(&db, &dto.username, &dto.password, &dto.role).await?;
    Ok(HttpResponse::Created().json(admin))
}

async fn delete(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    admin_service::delete_admin(&db, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}
