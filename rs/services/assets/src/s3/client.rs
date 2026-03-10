use std::time::Duration;

use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use uuid::Uuid;

use crate::config::S3Config;

use super::error::S3Error;

#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
    presigned_url_expiry: Duration,
}

impl S3Client {
    pub async fn new(config: &S3Config) -> Result<Self, S3Error> {
        let credentials = Credentials::new(
            &config.access_key_id,
            &config.secret_access_key,
            None,
            None,
            "assets-service",
        );

        let mut s3_config_builder = S3ConfigBuilder::new()
            .behavior_version_latest()
            .region(Region::new(config.region.clone()))
            .credentials_provider(credentials)
            .force_path_style(true);

        if let Some(endpoint) = &config.endpoint {
            s3_config_builder = s3_config_builder.endpoint_url(endpoint);
        }

        let s3_config = s3_config_builder.build();
        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
            presigned_url_expiry: config.presigned_url_expiry,
        })
    }

    /// Generate a unique object key for a file
    /// Format: {org_id}/{uuid}.{ext} or {uuid}.{ext} if no org
    pub fn generate_object_key(org_id: Option<Uuid>, extension: &str) -> String {
        let file_id = Uuid::new_v4();
        let ext = if extension.is_empty() {
            String::new()
        } else {
            format!(".{extension}")
        };

        match org_id {
            Some(org) => format!("{org}/{file_id}{ext}"),
            None => format!("{file_id}{ext}"),
        }
    }

    /// Upload a file to S3
    pub async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), S3Error> {
        let body = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| S3Error::UploadFailed(e.to_string()))?;

        Ok(())
    }

    /// Delete a file from S3
    pub async fn delete(&self, key: &str) -> Result<(), S3Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3Error::DeleteFailed(e.to_string()))?;

        Ok(())
    }

    /// Generate a presigned download URL
    pub async fn presigned_download_url(&self, key: &str) -> Result<String, S3Error> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(self.presigned_url_expiry)
            .build()
            .map_err(|e| S3Error::PresignFailed(e.to_string()))?;

        let request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| S3Error::PresignFailed(e.to_string()))?;

        Ok(request.uri().to_string())
    }

    /// Generate a presigned upload URL
    pub async fn presigned_upload_url(
        &self,
        key: &str,
        content_type: &str,
    ) -> Result<String, S3Error> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(self.presigned_url_expiry)
            .build()
            .map_err(|e| S3Error::PresignFailed(e.to_string()))?;

        let request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| S3Error::PresignFailed(e.to_string()))?;

        Ok(request.uri().to_string())
    }

    /// Check if S3 connection is healthy
    pub async fn health_check(&self) -> bool {
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .is_ok()
    }
}
