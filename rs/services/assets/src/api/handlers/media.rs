use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::Multipart;
use pagination::{PaginatedResponse, with_pagination};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::AppState;
use crate::api::error::ApiError;
use crate::dto::request::{CreateMedia, UpdateMedia};
use crate::dto::response::MediaResponse;
use crate::repository::filter::MediaFilter;
use crate::repository::{MediaRepository, MediaRepositoryImpl};
use crate::s3::S3Client;

const MAX_MEDIA_SIZE: usize = 500 * 1024 * 1024; // 500MB

// Media MIME types (audio and video)
const MEDIA_MIME_TYPES: &[&str] = &[
    "video/mp4",
    "video/webm",
    "video/ogg",
    "video/quicktime",
    "video/x-msvideo",
    "audio/mpeg",
    "audio/wav",
    "audio/ogg",
    "audio/webm",
    "audio/aac",
    "audio/flac",
];

#[with_pagination]
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListMediaQuery {
    /// Filter by organization ID
    pub org_id: Option<Uuid>,
    /// Filter by uploader ID
    pub uploader_id: Option<Uuid>,
    /// Filter by MIME type
    pub mime_type: Option<String>,
    /// Search by file name
    pub search: Option<String>,
    /// Minimum duration in milliseconds
    pub min_duration_ms: Option<i32>,
    /// Maximum duration in milliseconds
    pub max_duration_ms: Option<i32>,
}

/// List media files with pagination
#[utoipa::path(
    get,
    path = "/media",
    params(ListMediaQuery),
    responses(
        (status = 200, description = "List of media files with pagination", body = PaginatedResponse<MediaResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "media"
)]
pub async fn list_media(
    State(state): State<AppState>,
    Query(query): Query<ListMediaQuery>,
) -> Result<Json<PaginatedResponse<MediaResponse>>, ApiError> {
    let repo = MediaRepositoryImpl::new(&state.db);

    let page = query.to_page()?;

    let mut filter = MediaFilter::new();
    if let Some(org_id) = query.org_id {
        filter = filter.org_id(org_id);
    }
    if let Some(uploader_id) = query.uploader_id {
        filter = filter.uploader_id(uploader_id);
    }
    if let Some(ref mime_type) = query.mime_type {
        filter = filter.mime_type(mime_type.clone());
    }
    if let Some(ref search) = query.search {
        filter = filter.search(search.clone());
    }
    if let Some(min_duration) = query.min_duration_ms {
        filter = filter.min_duration_ms(min_duration);
    }
    if let Some(max_duration) = query.max_duration_ms {
        filter = filter.max_duration_ms(max_duration);
    }

    let media = repo.list(filter, page.clone()).await?;
    let responses: Vec<MediaResponse> = media.into_iter().map(|m| m.into()).collect();
    let response = PaginatedResponse::from_page(responses, &page);

    Ok(Json(response))
}

/// Upload media metadata (for files uploaded via presigned URL)
///
/// Media files are typically too large for direct upload. Use the presigned
/// upload flow and then call this endpoint to register the metadata.
#[utoipa::path(
    post,
    path = "/media",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Media uploaded successfully", body = MediaResponse),
        (status = 400, description = "Bad request"),
        (status = 413, description = "File too large"),
        (status = 415, description = "Unsupported media type"),
        (status = 500, description = "Internal server error")
    ),
    tag = "media"
)]
pub async fn upload_media(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut org_id: Option<Uuid> = None;
    let mut uploader_id: Option<Uuid> = None;
    let mut width: Option<i32> = None;
    let mut height: Option<i32> = None;
    let mut duration_ms: Option<i32> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Failed to read file: {e}")))?;

                if data.len() > MAX_MEDIA_SIZE {
                    return Err(ApiError::PayloadTooLarge(format!(
                        "Media size exceeds maximum of {} bytes",
                        MAX_MEDIA_SIZE
                    )));
                }

                file_data = Some(data.to_vec());
            }
            "org_id" => {
                let value = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Failed to read org_id: {e}")))?;
                if !value.is_empty() {
                    org_id =
                        Some(value.parse().map_err(|_| {
                            ApiError::BadRequest("Invalid org_id format".to_string())
                        })?);
                }
            }
            "uploader_id" => {
                let value = field.text().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read uploader_id: {e}"))
                })?;
                if !value.is_empty() {
                    uploader_id = Some(value.parse().map_err(|_| {
                        ApiError::BadRequest("Invalid uploader_id format".to_string())
                    })?);
                }
            }
            "width" => {
                let value = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Failed to read width: {e}")))?;
                if !value.is_empty() {
                    width =
                        Some(value.parse().map_err(|_| {
                            ApiError::BadRequest("Invalid width format".to_string())
                        })?);
                }
            }
            "height" => {
                let value = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Failed to read height: {e}")))?;
                if !value.is_empty() {
                    height =
                        Some(value.parse().map_err(|_| {
                            ApiError::BadRequest("Invalid height format".to_string())
                        })?);
                }
            }
            "duration_ms" => {
                let value = field.text().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read duration_ms: {e}"))
                })?;
                if !value.is_empty() {
                    duration_ms = Some(value.parse().map_err(|_| {
                        ApiError::BadRequest("Invalid duration_ms format".to_string())
                    })?);
                }
            }
            _ => {}
        }
    }

    let data = file_data.ok_or_else(|| ApiError::BadRequest("No file provided".to_string()))?;
    let original_name =
        file_name.ok_or_else(|| ApiError::BadRequest("No file name provided".to_string()))?;

    // Media metadata is required since we can't extract it from binary
    let width = width.ok_or_else(|| ApiError::BadRequest("width is required".to_string()))?;
    let height = height.ok_or_else(|| ApiError::BadRequest("height is required".to_string()))?;
    let duration_ms =
        duration_ms.ok_or_else(|| ApiError::BadRequest("duration_ms is required".to_string()))?;

    // Detect MIME type from content
    let mime_type = infer::get(&data)
        .map(|t| t.mime_type().to_string())
        .unwrap_or_else(|| {
            mime_guess::from_path(&original_name)
                .first_or_octet_stream()
                .to_string()
        });

    // Validate media type
    if !MEDIA_MIME_TYPES.contains(&mime_type.as_str()) {
        return Err(ApiError::UnsupportedMediaType(format!(
            "Unsupported media type: {mime_type}"
        )));
    }

    // Get file extension
    let extension = std::path::Path::new(&original_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    // Generate object key and upload
    let object_key = S3Client::generate_object_key(org_id, extension);
    let size_bytes = data.len() as i64;

    state.s3.upload(&object_key, data, &mime_type).await?;

    // Create database record
    let repo = MediaRepositoryImpl::new(&state.db);
    let media = repo
        .create(CreateMedia {
            org_id,
            uploader_id,
            object_key,
            file_name: original_name,
            mime_type,
            size_bytes,
            height,
            width,
            duration_ms,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(MediaResponse::from(media))))
}

/// Get a presigned download URL for media
#[utoipa::path(
    get,
    path = "/media/{id}",
    params(
        ("id" = Uuid, Path, description = "Media ID")
    ),
    responses(
        (status = 200, description = "Presigned download URL"),
        (status = 404, description = "Media not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "media"
)]
pub async fn get_media_url(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let repo = MediaRepositoryImpl::new(&state.db);
    let media = repo.get(id).await?.ok_or(ApiError::NotFound)?;

    let download_url = state.s3.presigned_download_url(&media.object_key).await?;

    Ok(Json(serde_json::json!({
        "download_url": download_url
    })))
}

/// Get media metadata
#[utoipa::path(
    get,
    path = "/media/{id}/metadata",
    params(
        ("id" = Uuid, Path, description = "Media ID")
    ),
    responses(
        (status = 200, description = "Media metadata", body = MediaResponse),
        (status = 404, description = "Media not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "media"
)]
pub async fn get_media_metadata(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MediaResponse>, ApiError> {
    let repo = MediaRepositoryImpl::new(&state.db);
    let media = repo.get(id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(media.into()))
}

/// Update media metadata
#[utoipa::path(
    put,
    path = "/media/{id}/metadata",
    params(
        ("id" = Uuid, Path, description = "Media ID")
    ),
    request_body = UpdateMedia,
    responses(
        (status = 200, description = "Media metadata updated", body = MediaResponse),
        (status = 404, description = "Media not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "media"
)]
pub async fn update_media_metadata(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<UpdateMedia>,
) -> Result<Json<MediaResponse>, ApiError> {
    let repo = MediaRepositoryImpl::new(&state.db);
    let media = repo.update(id, update).await?;
    Ok(Json(media.into()))
}

/// Delete media
#[utoipa::path(
    delete,
    path = "/media/{id}",
    params(
        ("id" = Uuid, Path, description = "Media ID")
    ),
    responses(
        (status = 204, description = "Media deleted successfully"),
        (status = 404, description = "Media not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "media"
)]
pub async fn delete_media(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let repo = MediaRepositoryImpl::new(&state.db);

    let media = repo.delete(id).await?;
    state.s3.delete(&media.object_key).await?;

    Ok(StatusCode::NO_CONTENT)
}
