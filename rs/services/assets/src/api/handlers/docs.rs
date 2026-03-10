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
use crate::dto::request::{CreateDoc, UpdateDoc};
use crate::dto::response::DocResponse;
use crate::repository::filter::DocFilter;
use crate::repository::{DocRepository, DocRepositoryImpl};
use crate::s3::S3Client;

const MAX_DOC_SIZE: usize = 50 * 1024 * 1024; // 50MB

// Document MIME types
const DOC_MIME_TYPES: &[&str] = &[
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/vnd.ms-powerpoint",
    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    "text/plain",
    "text/csv",
    "text/markdown",
    "application/json",
    "application/xml",
];

#[with_pagination]
#[derive(Debug, Deserialize, IntoParams)]
pub struct ListDocsQuery {
    /// Filter by organization ID
    pub org_id: Option<Uuid>,
    /// Filter by uploader ID
    pub uploader_id: Option<Uuid>,
    /// Filter by MIME type
    pub mime_type: Option<String>,
    /// Search by file name
    pub search: Option<String>,
}

/// List documents with pagination
#[utoipa::path(
    get,
    path = "/docs",
    params(ListDocsQuery),
    responses(
        (status = 200, description = "List of documents with pagination", body = PaginatedResponse<DocResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "docs"
)]
pub async fn list_docs(
    State(state): State<AppState>,
    Query(query): Query<ListDocsQuery>,
) -> Result<Json<PaginatedResponse<DocResponse>>, ApiError> {
    let repo = DocRepositoryImpl::new(&state.db);

    let page = query.to_page()?;

    let mut filter = DocFilter::new();
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

    let docs = repo.list(filter, page.clone()).await?;
    let responses: Vec<DocResponse> = docs.into_iter().map(|d| d.into()).collect();
    let response = PaginatedResponse::from_page(responses, &page);

    Ok(Json(response))
}

/// Upload a document via multipart form
#[utoipa::path(
    post,
    path = "/docs",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Document uploaded successfully", body = DocResponse),
        (status = 400, description = "Bad request"),
        (status = 413, description = "File too large"),
        (status = 415, description = "Unsupported media type"),
        (status = 500, description = "Internal server error")
    ),
    tag = "docs"
)]
pub async fn upload_doc(
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

                if data.len() > MAX_DOC_SIZE {
                    return Err(ApiError::PayloadTooLarge(format!(
                        "Document size exceeds maximum of {} bytes",
                        MAX_DOC_SIZE
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

    // Validate document type
    if !DOC_MIME_TYPES.contains(&mime_type.as_str()) {
        return Err(ApiError::UnsupportedMediaType(format!(
            "Unsupported document type: {mime_type}"
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
    let repo = DocRepositoryImpl::new(&state.db);
    let doc = repo
        .create(CreateDoc {
            org_id,
            uploader_id,
            object_key,
            file_name: original_name,
            mime_type,
            size_bytes,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(DocResponse::from(doc))))
}

/// Get a presigned download URL for a document
#[utoipa::path(
    get,
    path = "/docs/{id}",
    params(
        ("id" = Uuid, Path, description = "Document ID")
    ),
    responses(
        (status = 200, description = "Presigned download URL"),
        (status = 404, description = "Document not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "docs"
)]
pub async fn get_doc_url(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let repo = DocRepositoryImpl::new(&state.db);
    let doc = repo.get(id).await?.ok_or(ApiError::NotFound)?;

    let download_url = state.s3.presigned_download_url(&doc.object_key).await?;

    Ok(Json(serde_json::json!({
        "download_url": download_url
    })))
}

/// Get document metadata
#[utoipa::path(
    get,
    path = "/docs/{id}/metadata",
    params(
        ("id" = Uuid, Path, description = "Document ID")
    ),
    responses(
        (status = 200, description = "Document metadata", body = DocResponse),
        (status = 404, description = "Document not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "docs"
)]
pub async fn get_doc_metadata(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocResponse>, ApiError> {
    let repo = DocRepositoryImpl::new(&state.db);
    let doc = repo.get(id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(doc.into()))
}

/// Update document metadata
#[utoipa::path(
    put,
    path = "/docs/{id}/metadata",
    params(
        ("id" = Uuid, Path, description = "Document ID")
    ),
    request_body = UpdateDoc,
    responses(
        (status = 200, description = "Document metadata updated", body = DocResponse),
        (status = 404, description = "Document not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "docs"
)]
pub async fn update_doc_metadata(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(update): Json<UpdateDoc>,
) -> Result<Json<DocResponse>, ApiError> {
    let repo = DocRepositoryImpl::new(&state.db);
    let doc = repo.update(id, update).await?;
    Ok(Json(doc.into()))
}

/// Delete a document
#[utoipa::path(
    delete,
    path = "/docs/{id}",
    params(
        ("id" = Uuid, Path, description = "Document ID")
    ),
    responses(
        (status = 204, description = "Document deleted successfully"),
        (status = 404, description = "Document not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "docs"
)]
pub async fn delete_doc(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let repo = DocRepositoryImpl::new(&state.db);

    let doc = repo.delete(id).await?;
    state.s3.delete(&doc.object_key).await?;

    Ok(StatusCode::NO_CONTENT)
}
