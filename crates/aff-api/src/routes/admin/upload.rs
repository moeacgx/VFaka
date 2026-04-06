use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures_util::StreamExt;
use sea_orm::DatabaseConnection;
use std::path::Path;
use tracing::info;

use aff_common::error::{AppError, AppResult};
use aff_core::services::settings_service;

const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const MAX_VIDEO_SIZE: usize = 50 * 1024 * 1024; // 50MB
const UPLOAD_DIR: &str = "data/uploads";

const ALLOWED_IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];
const ALLOWED_VIDEO_EXTS: &[&str] = &["mp4", "webm"];

pub async fn upload_file(
    db: web::Data<DatabaseConnection>,
    mut payload: Multipart,
) -> AppResult<HttpResponse> {
    let storage_type = settings_service::get_setting(db.get_ref(), "storage_type")
        .await?
        .unwrap_or_else(|| "local".to_string());

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?;

        let content_disposition = field.content_disposition().clone();
        let cd = content_disposition
            .ok_or_else(|| AppError::BadRequest("Missing content disposition".to_string()))?;
        let filename = cd
            .get_filename()
            .ok_or_else(|| AppError::BadRequest("No filename provided".to_string()))?
            .to_string();

        let ext = Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .ok_or_else(|| AppError::BadRequest("File must have an extension".to_string()))?;

        let is_image = ALLOWED_IMAGE_EXTS.contains(&ext.as_str());
        let is_video = ALLOWED_VIDEO_EXTS.contains(&ext.as_str());

        if !is_image && !is_video {
            return Err(AppError::BadRequest(format!(
                "Unsupported file type: .{}. Allowed: images (jpg, jpeg, png, gif, webp), videos (mp4, webm)",
                ext
            )));
        }

        let max_size = if is_video { MAX_VIDEO_SIZE } else { MAX_IMAGE_SIZE };

        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| AppError::Internal(format!("Read error: {}", e)))?;
            data.extend_from_slice(&chunk);
            if data.len() > max_size {
                return Err(AppError::BadRequest(format!(
                    "File too large. Max: {}MB",
                    max_size / 1024 / 1024
                )));
            }
        }

        let unique_name = format!("{}.{}", uuid::Uuid::new_v4(), ext);

        match storage_type.as_str() {
            "s3" => {
                let url = upload_to_s3(db.get_ref(), &unique_name, &data).await?;
                info!(filename = %unique_name, "File uploaded to S3");
                return Ok(HttpResponse::Ok().json(serde_json::json!({
                    "url": url,
                    "filename": unique_name,
                })));
            }
            _ => {
                let dir = Path::new(UPLOAD_DIR);
                tokio::fs::create_dir_all(dir)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to create upload dir: {}", e)))?;

                let filepath = dir.join(&unique_name);
                tokio::fs::write(&filepath, &data)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to write file: {}", e)))?;

                let url = format!("/uploads/{}", unique_name);
                info!(filename = %unique_name, "File uploaded locally");
                return Ok(HttpResponse::Ok().json(serde_json::json!({
                    "url": url,
                    "filename": unique_name,
                })));
            }
        }
    }

    Err(AppError::BadRequest("No file provided".to_string()))
}

async fn upload_to_s3(db: &DatabaseConnection, filename: &str, data: &[u8]) -> AppResult<String> {
    let endpoint = settings_service::get_setting(db, "s3_endpoint")
        .await?
        .unwrap_or_default();
    let bucket = settings_service::get_setting(db, "s3_bucket")
        .await?
        .unwrap_or_default();
    let access_key = settings_service::get_setting(db, "s3_access_key")
        .await?
        .unwrap_or_default();
    let secret_key = settings_service::get_setting(db, "s3_secret_key")
        .await?
        .unwrap_or_default();
    let region = settings_service::get_setting(db, "s3_region")
        .await?
        .unwrap_or_else(|| "auto".to_string());

    if endpoint.is_empty() || bucket.is_empty() || access_key.is_empty() || secret_key.is_empty() {
        return Err(AppError::BadRequest("S3 storage not configured".to_string()));
    }

    let content_type = mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string();

    let client = reqwest::Client::new();

    // Use pre-signed URL style: PUT directly to S3-compatible endpoint
    let url = format!("{}/{}/{}", endpoint.trim_end_matches('/'), bucket, filename);

    let resp = client
        .put(&url)
        .header("Content-Type", &content_type)
        .header("x-amz-acl", "public-read")
        .body(data.to_vec())
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("S3 upload failed: {}", e)))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!("S3 upload error: {} - region: {}, access_key present: {}", body, region, !access_key.is_empty())));
    }

    Ok(format!("{}/{}/{}", endpoint.trim_end_matches('/'), bucket, filename))
}

pub fn scope() -> actix_web::Scope {
    web::scope("/upload")
        .route("", web::post().to(upload_file))
}
