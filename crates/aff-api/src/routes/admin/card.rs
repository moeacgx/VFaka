use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::error::AppResult;
use aff_core::services::card_service;
use aff_entity::dto::ImportCardsDto;

#[derive(Debug, Deserialize)]
pub struct CardListQuery {
    pub product_id: Option<i32>,
    pub status: Option<String>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/cards")
            .route("", web::get().to(list))
            .route("/import", web::post().to(import))
            .route("/{id}", web::delete().to(delete)),
    );
}

async fn list(
    db: web::Data<DatabaseConnection>,
    query: web::Query<CardListQuery>,
) -> AppResult<HttpResponse> {
    let cards =
        card_service::list_cards(&db, query.product_id, query.status.clone()).await?;
    Ok(HttpResponse::Ok().json(cards))
}

async fn import(
    db: web::Data<DatabaseConnection>,
    body: web::Json<ImportCardsDto>,
) -> AppResult<HttpResponse> {
    let dto = body.into_inner();
    let count = card_service::import_cards(&db, dto.product_id, &dto.cards).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "imported": count,
    })))
}

async fn delete(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    card_service::delete_card(&db, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}
