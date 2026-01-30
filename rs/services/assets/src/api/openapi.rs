use pagination::PaginatedResponse;
use utoipa::OpenApi;

use crate::dto::request::{
    PresignedUploadRequest, UpdateDoc, UpdateFile, UpdateImage, UpdateMedia,
};
use crate::dto::response::{
    DocResponse, FileResponse, ImageResponse, MediaResponse, PresignedUploadResponse,
    UploadResponse,
};

/// OpenAPI documentation for the Assets Service
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Assets Service API",
        version = "1.0.0",
        description = "File and asset management service API with S3 storage"
    ),
    paths(
        // Health
        crate::api::handlers::health::health_check,
        // Files
        crate::api::handlers::files::list_files,
        crate::api::handlers::files::upload_file,
        crate::api::handlers::files::presign_upload,
        crate::api::handlers::files::get_file_url,
        crate::api::handlers::files::get_file_metadata,
        crate::api::handlers::files::update_file_metadata,
        crate::api::handlers::files::delete_file,
        // Docs
        crate::api::handlers::docs::list_docs,
        crate::api::handlers::docs::upload_doc,
        crate::api::handlers::docs::get_doc_url,
        crate::api::handlers::docs::get_doc_metadata,
        crate::api::handlers::docs::update_doc_metadata,
        crate::api::handlers::docs::delete_doc,
        // Images
        crate::api::handlers::images::list_images,
        crate::api::handlers::images::upload_image,
        crate::api::handlers::images::get_image_url,
        crate::api::handlers::images::get_image_metadata,
        crate::api::handlers::images::update_image_metadata,
        crate::api::handlers::images::delete_image,
        // Media
        crate::api::handlers::media::list_media,
        crate::api::handlers::media::upload_media,
        crate::api::handlers::media::get_media_url,
        crate::api::handlers::media::get_media_metadata,
        crate::api::handlers::media::update_media_metadata,
        crate::api::handlers::media::delete_media,
    ),
    components(
        schemas(
            // Request DTOs
            UpdateFile,
            UpdateDoc,
            UpdateImage,
            UpdateMedia,
            PresignedUploadRequest,
            // Response DTOs
            FileResponse,
            DocResponse,
            ImageResponse,
            MediaResponse,
            UploadResponse,
            PresignedUploadResponse,
            // Paginated responses
            PaginatedResponse<FileResponse>,
            PaginatedResponse<DocResponse>,
            PaginatedResponse<ImageResponse>,
            PaginatedResponse<MediaResponse>,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "files", description = "General file management endpoints"),
        (name = "docs", description = "Document management endpoints"),
        (name = "images", description = "Image management endpoints"),
        (name = "media", description = "Audio/video media management endpoints")
    )
)]
pub struct ApiDoc;
