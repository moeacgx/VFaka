use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use aff_common::error::AppResult;
use aff_core::services::card_service;
use aff_entity::dto::ImportCardsDto;

#[derive(Debug, Deserialize)]
pub struct CardListQuery {
    pub product_id: Option<i32>,
    pub variant_id: Option<i32>,
    pub status: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCardDto {
    pub content: Option<String>,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/cards")
        .route("", web::get().to(list))
        .route("/import", web::post().to(import))
        .route("/{id}", web::put().to(update))
        .route("/{id}", web::delete().to(delete))
}

async fn list(
    db: web::Data<DatabaseConnection>,
    query: web::Query<CardListQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let (cards, total) =
        card_service::list_cards(&db, query.product_id, query.variant_id, query.status.clone(), page, per_page).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "items": cards,
        "total": total,
        "page": page,
        "per_page": per_page,
    })))
}

async fn import(
    db: web::Data<DatabaseConnection>,
    body: web::Json<ImportCardsDto>,
) -> AppResult<HttpResponse> {
    let dto = body.into_inner();
    let count = card_service::import_cards(&db, dto.product_id, dto.variant_id, &dto.cards).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "imported": count,
    })))
}

async fn update(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
    body: web::Json<UpdateCardDto>,
) -> AppResult<HttpResponse> {
    let card = card_service::update_card(&db, path.into_inner(), body.into_inner().content).await?;
    Ok(HttpResponse::Ok().json(card))
}

async fn delete(
    db: web::Data<DatabaseConnection>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    card_service::delete_card(&db, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}
