use axum::{
    routing::{get, post},
    Json, Router,
};
use utoipa::OpenApi;

use crate::AppState;

use super::handlers::{docs, files, health, images, media};
use super::openapi::ApiDoc;

async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

pub fn api_routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/openapi.json", get(openapi_spec))
        .route("/health", get(health::health_check))
        // Files routes
        .route("/files", get(files::list_files).post(files::upload_file))
        .route("/files/presign", post(files::presign_upload))
        .route(
            "/files/{id}",
            get(files::get_file_url).delete(files::delete_file),
        )
        .route(
            "/files/{id}/metadata",
            get(files::get_file_metadata).put(files::update_file_metadata),
        )
        // Docs routes
        .route("/docs", get(docs::list_docs).post(docs::upload_doc))
        .route(
            "/docs/{id}",
            get(docs::get_doc_url).delete(docs::delete_doc),
        )
        .route(
            "/docs/{id}/metadata",
            get(docs::get_doc_metadata).put(docs::update_doc_metadata),
        )
        // Images routes
        .route(
            "/images",
            get(images::list_images).post(images::upload_image),
        )
        .route(
            "/images/{id}",
            get(images::get_image_url).delete(images::delete_image),
        )
        .route(
            "/images/{id}/metadata",
            get(images::get_image_metadata).put(images::update_image_metadata),
        )
        // Media routes
        .route("/media", get(media::list_media).post(media::upload_media))
        .route(
            "/media/{id}",
            get(media::get_media_url).delete(media::delete_media),
        )
        .route(
            "/media/{id}/metadata",
            get(media::get_media_metadata).put(media::update_media_metadata),
        )
}
