//! Optional S3-compatible object storage for uploaded media.
//!
//! Cloudflare R2 is configured entirely from the environment. When the
//! `R2_*` variables are absent the server keeps using the local `data/`
//! directories, so existing deployments are unchanged.

use std::fmt;
use std::sync::Arc;

use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use tokio::sync::Mutex;

use crate::error::ApiError;

/// Object key for an uploaded brand image.
pub fn image_key(file: &str) -> String {
    format!("images/{file}")
}

/// Object key for an uploaded brand font.
pub fn font_key(file: &str) -> String {
    format!("fonts/{file}")
}

/// Object key for the font manifest (family names + upload timestamps).
pub const FONT_MANIFEST_KEY: &str = "fonts/fonts.json";

/// A tiny S3 client configured for Cloudflare R2.
#[derive(Clone)]
pub struct R2Store {
    client: Client,
    bucket: String,
    endpoint: String,
    /// Serializes read-modify-write updates of the font manifest.
    font_lock: Arc<Mutex<()>>,
}

impl fmt::Debug for R2Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never print credentials; the client itself is opaque.
        f.debug_struct("R2Store")
            .field("bucket", &self.bucket)
            .field("endpoint", &self.endpoint)
            .finish_non_exhaustive()
    }
}

fn env_value(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

impl R2Store {
    /// Builds the client when `R2_ACCOUNT_ID`, `R2_BUCKET`,
    /// `R2_ACCESS_KEY_ID` and `R2_SECRET_ACCESS_KEY` are all present.
    /// A partially configured environment logs a warning and falls back to
    /// local storage instead of failing the boot.
    pub fn from_env() -> Option<Self> {
        let keys = [
            "R2_ACCOUNT_ID",
            "R2_BUCKET",
            "R2_ACCESS_KEY_ID",
            "R2_SECRET_ACCESS_KEY",
        ];
        let present = keys.iter().filter(|key| env_value(key).is_some()).count();
        if present == 0 {
            return None;
        }
        let Some(account_id) = env_value("R2_ACCOUNT_ID") else {
            tracing::warn!("R2 disabled: R2_ACCOUNT_ID is missing");
            return None;
        };
        let Some(bucket) = env_value("R2_BUCKET") else {
            tracing::warn!("R2 disabled: R2_BUCKET is missing");
            return None;
        };
        let Some(access_key_id) = env_value("R2_ACCESS_KEY_ID") else {
            tracing::warn!("R2 disabled: R2_ACCESS_KEY_ID is missing");
            return None;
        };
        let Some(secret_access_key) = env_value("R2_SECRET_ACCESS_KEY") else {
            tracing::warn!("R2 disabled: R2_SECRET_ACCESS_KEY is missing");
            return None;
        };
        let endpoint = env_value("R2_ENDPOINT")
            .unwrap_or_else(|| format!("https://{account_id}.r2.cloudflarestorage.com"));

        let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "r2-env");
        let config = aws_sdk_s3::config::Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new("auto"))
            .endpoint_url(endpoint.clone())
            .credentials_provider(credentials)
            .force_path_style(true)
            .build();

        Some(Self {
            client: Client::from_conf(config),
            bucket,
            endpoint,
            font_lock: Arc::new(Mutex::new(())),
        })
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Serializes font manifest mutations (upload/delete) so two concurrent
    /// requests cannot overwrite each other's entries.
    pub async fn lock_fonts(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.font_lock.lock().await
    }

    pub async fn put(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> Result<(), ApiError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(bytes))
            .send()
            .await
            .map_err(|error| {
                tracing::error!(%error, key, "R2 put_object failed");
                ApiError::internal("cannot store file")
            })?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ApiError> {
        match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(output) => {
                let body = output.body.collect().await.map_err(|error| {
                    tracing::error!(%error, key, "R2 response body failed");
                    ApiError::internal("cannot read stored file")
                })?;
                Ok(Some(body.into_bytes().to_vec()))
            }
            Err(error)
                if error
                    .as_service_error()
                    .map(|service| service.is_no_such_key())
                    .unwrap_or(false) =>
            {
                Ok(None)
            }
            Err(error) => {
                tracing::error!(%error, key, "R2 get_object failed");
                Err(ApiError::internal("cannot read stored file"))
            }
        }
    }

    pub async fn head(&self, key: &str) -> Result<bool, ApiError> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(error)
                if error
                    .as_service_error()
                    .map(|service| service.is_not_found())
                    .unwrap_or(false) =>
            {
                Ok(false)
            }
            Err(error) => {
                tracing::error!(%error, key, "R2 head_object failed");
                Err(ApiError::internal("cannot check stored file"))
            }
        }
    }

    pub async fn delete(&self, key: &str) -> Result<(), ApiError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|error| {
                tracing::error!(%error, key, "R2 delete_object failed");
                ApiError::internal("cannot delete stored file")
            })?;
        Ok(())
    }
}
