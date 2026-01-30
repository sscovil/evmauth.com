use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::Multipart;
use pagination::{with_pagination, PaginatedResponse};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::dto::request::{CreateFile, PresignedUploadRequest, UpdateFile};
use crate::dto::response::{FileResponse, PresignedUploadResponse, UploadResponse};
use crate::repository::filter::FileFilter;
use crate::repository::{FileRepository, FileRepositoryImpl};
use crate::s3::S3Client;
use crate::AppState;

const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB

#[with_pagination]
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListFilesQuery {
    /// Filter by organization ID
    pub org_id: Option<Uuid>,
    /// Filter by uploader ID
    pub uploader_id: Option<Uuid>,
    /// Filter by MIME type
    pub mime_type: Option<String>,
    /// Search by file name
    pub search: Option<String>,
}

/// List files with pagination
#[utoipa::path(
    get,
    path = "/files",
    params(ListFilesQuery),
    responses(
        (status = 200, description = "List of files with pagination", body = PaginatedResponse<FileResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "files"
)]
pub async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<ListFilesQuery>,
) -> Result<Json<PaginatedResponse<FileResponse>>, ApiError> {
    let repo = FileRepositoryImpl::new(&state.db);

    let page = query.to_page()?;

    let mut filter = FileFilter::new();
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

    let files = repo.list(filter, page.clone()).await?;
    let responses: Vec<FileResponse> = files.into_iter().map(|f| f.into()).collect();
    let response = PaginatedResponse::from_page(responses, &page);

    Ok(Json(response))
}

/// Upload a file via multipart form
#[utoipa::path(
    post,
    path = "/files",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "File uploaded successfully", body = UploadResponse),
        (status = 400, description = "Bad request"),
        (status = 413, description = "File too large"),
        (status = 415, description = "Unsupported media type"),
        (status = 500, description = "Internal server error")
    ),
    tag = "files"
)]
pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut org_id: Option<Uuid> = None;
    let mut uploader_id: Option<Uuid> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Failed to read file: {e}")))?;

                if data.len() > MAX_FILE_SIZE {
                    return Err(ApiError::PayloadTooLarge(format!(
                        "File size exceeds maximum of {} bytes",
                        MAX_FILE_SIZE
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
            _ => {}
        }
    }

    let data = file_data.ok_or_else(|| ApiError::BadRequest("No file provided".to_string()))?;
    let original_name =
        file_name.ok_or_else(|| ApiError::BadRequest("No file name provided".to_string()))?;

    // Detect MIME type from content
    let mime_type = infer::get(&data)
        .map(|t| t.mime_type().to_string())
        .unwrap_or_else(|| {
            mime_guess::from_path(&original_name)
                .first_or_octet_stream()
                .to_string()
        });

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
    let repo = FileRepositoryImpl::new(&state.db);
    let file = repo
        .create(CreateFile {
            org_id,
            uploader_id,
            object_key: object_key.clone(),
            file_name: original_name,
            mime_type,
            size_bytes,
        })
        .await?;

    // Generate download URL
    let download_url = state.s3.presigned_download_url(&object_key).await?;

    Ok((
        StatusCode::CREATED,
        Json(UploadResponse {
            file: file.into(),
            download_url,
        }),
    ))
}

/// Get a presigned upload URL for direct client upload
#[utoipa::path(
    post,
    path = "/files/presign",
    request_body = PresignedUploadRequest,
    responses(
        (status = 200, description = "Presigned upload URL generated", body = PresignedUploadResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "files"
)]
pub async fn presign_upload(
    State(state): State<AppState>,
    Json(request): Json<PresignedUploadRequest>,
) -> Result<Json<PresignedUploadResponse>, ApiError> {
    // Get file extension
    let extension = std::path::Path::new(&request.file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    // Generate object key
    let object_key = S3Client::generate_object_key(request.org_id, extension);

    // Create database record
    let repo = FileRepositoryImpl::new(&state.db);
    let file = repo
        .create(CreateFile {
            org_id: request.org_id,
            uploader_id: None,
            object_key: object_key.clone(),
            file_name: request.file_name,
            mime_type: request.content_type.clone(),
            size_bytes: request.size_bytes,
        })
        .await?;

    // Generate presigned upload URL
    let upload_url = state
        .s3
        .presigned_upload_url(&object_key, &request.content_type)
        .await?;

    Ok(Json(PresignedUploadResponse {
        id: file.id,
        object_key,
        upload_url,
    }))
}

/// Get a presigned download URL for a file
#[utoipa::path(
    get,
    path = "/files/{id}",
    params(
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 200, description = "Presigned download URL"),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "files"
)]
pub async fn get_file_url(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let repo = FileRepositoryImpl::new(&state.db);
    let file = repo.get(id).await?.ok_or(ApiError::NotFound)?;

    let download_url = state.s3.presigned_download_url(&file.object_key).await?;

    Ok(Json(serde_json::json!({
        "download_url": download_url
    })))
}

/// Get file metadata
#[utoipa::path(
    get,
    path = "/files/{id}/metadata",
    params(
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 200, description = "File metadata", body = FileResponse),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "files"
)]
pub async fn get_file_metadata(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<FileResponse>, ApiError> {
    let repo = FileRepositoryImpl::new(&state.db);
    let file = repo.get(id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(file.into()))
}

/// Update file metadata
#[utoipa::path(
    put,
    path = "/files/{id}/metadata",
    params(
        ("id" = Uuid, Path, description = "File ID")
    ),
    request_body = UpdateFile,
    responses(
        (status = 200, description = "File metadata updated", body = FileResponse),
        (status = 404, description = "File not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "files"
)]
pub async fn update_file_metadata(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<UpdateFile>,
) -> Result<Json<FileResponse>, ApiError> {
    let repo = FileRepositoryImpl::new(&state.db);
    let file = repo.update(id, update).await?;
    Ok(Json(file.into()))
}

/// Delete a file
#[utoipa::path(
    delete,
    path = "/files/{id}",
    params(
        ("id" = Uuid, Path, description = "File ID")
    ),
    responses(
        (status = 204, description = "File deleted successfully"),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "files"
)]
pub async fn delete_file(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let repo = FileRepositoryImpl::new(&state.db);

    // Delete from database first (returns the file record)
    let file = repo.delete(id).await?;

    // Delete from S3
    state.s3.delete(&file.object_key).await?;

    Ok(StatusCode::NO_CONTENT)
}
