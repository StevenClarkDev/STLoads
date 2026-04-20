use anyhow::{Context, anyhow};
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
use aws_sdk_s3::{Client, config::Builder as S3ConfigBuilder, primitives::ByteStream};
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

use crate::config::RuntimeConfig;

#[derive(Clone, Debug)]
pub struct DocumentStorageService {
    backend: String,
    root: PathBuf,
    object_storage: Option<ObjectStorageConfig>,
}

#[derive(Clone, Debug)]
struct ObjectStorageConfig {
    provider_label: String,
    bucket: String,
    region: String,
    endpoint: Option<String>,
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    session_token: Option<String>,
    force_path_style: bool,
    prefix: String,
}

#[derive(Clone, Debug)]
pub struct SavedDocumentFile {
    pub storage_provider: String,
    pub file_path: String,
}

impl DocumentStorageService {
    pub fn from_config(config: &RuntimeConfig) -> Self {
        let backend = config.document_storage_backend.trim().to_ascii_lowercase();
        let object_storage = matches!(backend.as_str(), "ibm_cos" | "s3")
            .then(|| ObjectStorageConfig::from_runtime(config, &backend))
            .transpose()
            .unwrap_or_else(|error| {
                tracing::warn!(error = %error, "object storage configuration is incomplete; document uploads will fail until OBJECT_STORAGE_* env vars are fixed");
                None
            });

        Self {
            backend,
            root: PathBuf::from(config.document_storage_root.trim()),
            object_storage,
        }
    }

    pub fn backend(&self) -> &str {
        &self.backend
    }

    pub async fn save_load_document(
        &self,
        load_id: i64,
        original_name: &str,
        bytes: &[u8],
    ) -> anyhow::Result<SavedDocumentFile> {
        match self.backend.as_str() {
            "local" => {
                self.save_local_load_document(load_id, original_name, bytes)
                    .await
            }
            "ibm_cos" | "s3" => {
                self.save_object_storage_document(load_id, original_name, bytes)
                    .await
            }
            other => Err(anyhow!(
                "document storage backend '{}' is not supported yet in this Rust slice",
                other
            )),
        }
    }

    pub async fn save_execution_document(
        &self,
        leg_id: i64,
        original_name: &str,
        bytes: &[u8],
    ) -> anyhow::Result<SavedDocumentFile> {
        match self.backend.as_str() {
            "local" => {
                self.save_local_execution_document(leg_id, original_name, bytes)
                    .await
            }
            "ibm_cos" | "s3" => {
                self.save_object_storage_execution_document(leg_id, original_name, bytes)
                    .await
            }
            other => Err(anyhow!(
                "document storage backend '{}' is not supported yet in this Rust slice",
                other
            )),
        }
    }

    pub async fn save_kyc_document(
        &self,
        user_id: i64,
        original_name: &str,
        bytes: &[u8],
    ) -> anyhow::Result<SavedDocumentFile> {
        match self.backend.as_str() {
            "local" => {
                self.save_local_kyc_document(user_id, original_name, bytes)
                    .await
            }
            "ibm_cos" | "s3" => {
                self.save_object_storage_kyc_document(user_id, original_name, bytes)
                    .await
            }
            other => Err(anyhow!(
                "document storage backend '{}' is not supported yet in this Rust slice",
                other
            )),
        }
    }
    pub async fn read_document(
        &self,
        storage_provider: &str,
        file_path: &str,
    ) -> anyhow::Result<Vec<u8>> {
        match storage_provider.trim().to_ascii_lowercase().as_str() {
            "local" => {
                let relative = file_path
                    .strip_prefix("local://")
                    .unwrap_or(file_path)
                    .trim_start_matches('/');
                let disk_path = self
                    .root
                    .join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
                fs::read(&disk_path).await.with_context(|| {
                    format!("failed to read local document {}", disk_path.display())
                })
            }
            "ibm_cos" | "s3" => {
                self.read_object_storage_document(storage_provider, file_path)
                    .await
            }
            other => Err(anyhow!(
                "document storage provider '{}' is not readable yet in this Rust slice",
                other
            )),
        }
    }

    pub async fn delete_document(
        &self,
        storage_provider: &str,
        file_path: &str,
    ) -> anyhow::Result<()> {
        match storage_provider.trim().to_ascii_lowercase().as_str() {
            "local" => {
                let relative = file_path
                    .strip_prefix("local://")
                    .unwrap_or(file_path)
                    .trim_start_matches('/');
                let disk_path = self
                    .root
                    .join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
                match fs::remove_file(&disk_path).await {
                    Ok(_) => Ok(()),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    Err(error) => Err(anyhow!(error)).with_context(|| {
                        format!("failed to delete local document {}", disk_path.display())
                    }),
                }
            }
            "ibm_cos" | "s3" => {
                self.delete_object_storage_document(storage_provider, file_path)
                    .await
            }
            other => Err(anyhow!(
                "document storage provider '{}' cannot delete documents yet in this Rust slice",
                other
            )),
        }
    }

    async fn save_local_kyc_document(
        &self,
        user_id: i64,
        original_name: &str,
        bytes: &[u8],
    ) -> anyhow::Result<SavedDocumentFile> {
        let sanitized = sanitize_file_name(original_name);
        let relative = format!(
            "kyc-documents/user-{}/{}-{}",
            user_id.max(0),
            Uuid::new_v4(),
            sanitized
        );
        let disk_path = self
            .root
            .join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        let parent = disk_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow!("document target directory could not be derived"))?;

        fs::create_dir_all(&parent)
            .await
            .with_context(|| format!("failed to create document directory {}", parent.display()))?;
        fs::write(&disk_path, bytes).await.with_context(|| {
            format!("failed to write uploaded document {}", disk_path.display())
        })?;

        Ok(SavedDocumentFile {
            storage_provider: "local".into(),
            file_path: format!("local://{}", relative.replace('\\', "/")),
        })
    }

    async fn save_object_storage_kyc_document(
        &self,
        user_id: i64,
        original_name: &str,
        bytes: &[u8],
    ) -> anyhow::Result<SavedDocumentFile> {
        let object_storage = self.object_storage_config()?;
        let key = object_storage.object_key_for_kyc(user_id, original_name);
        let client = object_storage.client().await?;

        client
            .put_object()
            .bucket(&object_storage.bucket)
            .key(&key)
            .body(ByteStream::from(bytes.to_vec()))
            .send()
            .await
            .with_context(|| {
                format!(
                    "failed to upload document {} to object storage bucket {}",
                    key, object_storage.bucket
                )
            })?;

        Ok(SavedDocumentFile {
            storage_provider: object_storage.provider_label.clone(),
            file_path: format!(
                "{}://{}/{}",
                object_storage.scheme(),
                object_storage.bucket,
                key
            ),
        })
    }
    async fn save_local_load_document(
        &self,
        load_id: i64,
        original_name: &str,
        bytes: &[u8],
    ) -> anyhow::Result<SavedDocumentFile> {
        let sanitized = sanitize_file_name(original_name);
        let relative = format!(
            "load-documents/load-{}/{}-{}",
            load_id.max(0),
            Uuid::new_v4(),
            sanitized
        );
        let disk_path = self
            .root
            .join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        let parent = disk_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow!("document target directory could not be derived"))?;

        fs::create_dir_all(&parent)
            .await
            .with_context(|| format!("failed to create document directory {}", parent.display()))?;
        fs::write(&disk_path, bytes).await.with_context(|| {
            format!("failed to write uploaded document {}", disk_path.display())
        })?;

        Ok(SavedDocumentFile {
            storage_provider: "local".into(),
            file_path: format!("local://{}", relative.replace('\\', "/")),
        })
    }

    async fn save_object_storage_document(
        &self,
        load_id: i64,
        original_name: &str,
        bytes: &[u8],
    ) -> anyhow::Result<SavedDocumentFile> {
        let object_storage = self.object_storage_config()?;
        let key = object_storage.object_key_for_load(load_id, original_name);
        let client = object_storage.client().await?;

        client
            .put_object()
            .bucket(&object_storage.bucket)
            .key(&key)
            .body(ByteStream::from(bytes.to_vec()))
            .send()
            .await
            .with_context(|| {
                format!(
                    "failed to upload document {} to object storage bucket {}",
                    key, object_storage.bucket
                )
            })?;

        Ok(SavedDocumentFile {
            storage_provider: object_storage.provider_label.clone(),
            file_path: format!(
                "{}://{}/{}",
                object_storage.scheme(),
                object_storage.bucket,
                key
            ),
        })
    }

    async fn save_local_execution_document(
        &self,
        leg_id: i64,
        original_name: &str,
        bytes: &[u8],
    ) -> anyhow::Result<SavedDocumentFile> {
        let sanitized = sanitize_file_name(original_name);
        let relative = format!(
            "leg-documents/leg-{}/{}-{}",
            leg_id.max(0),
            Uuid::new_v4(),
            sanitized
        );
        let disk_path = self
            .root
            .join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        let parent = disk_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow!("document target directory could not be derived"))?;

        fs::create_dir_all(&parent)
            .await
            .with_context(|| format!("failed to create document directory {}", parent.display()))?;
        fs::write(&disk_path, bytes).await.with_context(|| {
            format!("failed to write uploaded document {}", disk_path.display())
        })?;

        Ok(SavedDocumentFile {
            storage_provider: "local".into(),
            file_path: format!("local://{}", relative.replace('\\', "/")),
        })
    }

    async fn save_object_storage_execution_document(
        &self,
        leg_id: i64,
        original_name: &str,
        bytes: &[u8],
    ) -> anyhow::Result<SavedDocumentFile> {
        let object_storage = self.object_storage_config()?;
        let key = object_storage.object_key_for_leg(leg_id, original_name);
        let client = object_storage.client().await?;

        client
            .put_object()
            .bucket(&object_storage.bucket)
            .key(&key)
            .body(ByteStream::from(bytes.to_vec()))
            .send()
            .await
            .with_context(|| {
                format!(
                    "failed to upload document {} to object storage bucket {}",
                    key, object_storage.bucket
                )
            })?;

        Ok(SavedDocumentFile {
            storage_provider: object_storage.provider_label.clone(),
            file_path: format!(
                "{}://{}/{}",
                object_storage.scheme(),
                object_storage.bucket,
                key
            ),
        })
    }

    async fn read_object_storage_document(
        &self,
        storage_provider: &str,
        file_path: &str,
    ) -> anyhow::Result<Vec<u8>> {
        let object_storage = self.object_storage_config()?;
        let (bucket, key) = parse_object_storage_path(file_path)
            .unwrap_or_else(|| (object_storage.bucket.clone(), file_path.trim().to_string()));
        let client = object_storage.client().await?;

        let output = client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .with_context(|| {
                format!(
                    "failed to fetch {} from {} object storage",
                    file_path, storage_provider
                )
            })?;

        let aggregated = output
            .body
            .collect()
            .await
            .with_context(|| format!("failed to stream document body for {}", file_path))?;
        Ok(aggregated.into_bytes().to_vec())
    }

    async fn delete_object_storage_document(
        &self,
        storage_provider: &str,
        file_path: &str,
    ) -> anyhow::Result<()> {
        let object_storage = self.object_storage_config()?;
        let (bucket, key) = parse_object_storage_path(file_path)
            .unwrap_or_else(|| (object_storage.bucket.clone(), file_path.trim().to_string()));
        let client = object_storage.client().await?;

        client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .with_context(|| {
                format!(
                    "failed to delete {} from {} object storage",
                    file_path, storage_provider
                )
            })?;

        Ok(())
    }

    fn object_storage_config(&self) -> anyhow::Result<&ObjectStorageConfig> {
        self.object_storage.as_ref().ok_or_else(|| {
            anyhow!(
                "object storage is not configured for backend {}",
                self.backend
            )
        })
    }
}

impl ObjectStorageConfig {
    fn from_runtime(config: &RuntimeConfig, backend: &str) -> anyhow::Result<Self> {
        let bucket = config
            .object_storage_bucket
            .clone()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                anyhow!(
                    "OBJECT_STORAGE_BUCKET is required for {} document storage",
                    backend
                )
            })?;

        Ok(Self {
            provider_label: if backend == "ibm_cos" {
                "ibm_cos".into()
            } else {
                "s3".into()
            },
            bucket,
            region: config.object_storage_region.trim().to_string(),
            endpoint: config
                .object_storage_endpoint
                .clone()
                .filter(|value| !value.trim().is_empty()),
            access_key_id: config
                .object_storage_access_key_id
                .clone()
                .filter(|value| !value.trim().is_empty()),
            secret_access_key: config
                .object_storage_secret_access_key
                .clone()
                .filter(|value| !value.trim().is_empty()),
            session_token: config
                .object_storage_session_token
                .clone()
                .filter(|value| !value.trim().is_empty()),
            force_path_style: config.object_storage_force_path_style,
            prefix: config.object_storage_prefix.trim_matches('/').to_string(),
        })
    }

    fn object_key_for_kyc(&self, user_id: i64, original_name: &str) -> String {
        let sanitized = sanitize_file_name(original_name);
        let root = if self.prefix.is_empty() {
            "kyc-documents".to_string()
        } else {
            self.prefix.clone()
        };

        format!(
            "{}/user-{}/{}-{}",
            root,
            user_id.max(0),
            Uuid::new_v4(),
            sanitized
        )
    }
    fn object_key_for_load(&self, load_id: i64, original_name: &str) -> String {
        let sanitized = sanitize_file_name(original_name);
        let root = if self.prefix.is_empty() {
            "load-documents".to_string()
        } else {
            self.prefix.clone()
        };

        format!(
            "{}/load-{}/{}-{}",
            root,
            load_id.max(0),
            Uuid::new_v4(),
            sanitized
        )
    }

    fn object_key_for_leg(&self, leg_id: i64, original_name: &str) -> String {
        let sanitized = sanitize_file_name(original_name);
        let root = if self.prefix.is_empty() {
            "leg-documents".to_string()
        } else {
            self.prefix.clone()
        };

        format!(
            "{}/leg-{}/{}-{}",
            root,
            leg_id.max(0),
            Uuid::new_v4(),
            sanitized
        )
    }

    fn scheme(&self) -> &'static str {
        if self.provider_label == "ibm_cos" {
            "ibm-cos"
        } else {
            "s3"
        }
    }

    async fn client(&self) -> anyhow::Result<Client> {
        let mut builder = S3ConfigBuilder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(self.region.clone()))
            .force_path_style(self.force_path_style);

        if let Some(endpoint) = self.endpoint.as_deref() {
            builder = builder.endpoint_url(endpoint);
        }

        if let (Some(access_key_id), Some(secret_access_key)) = (
            self.access_key_id.as_deref(),
            self.secret_access_key.as_deref(),
        ) {
            let credentials = Credentials::new(
                access_key_id,
                secret_access_key,
                self.session_token.clone(),
                None,
                "rust-port-object-storage",
            );
            builder = builder.credentials_provider(SharedCredentialsProvider::new(credentials));
        }

        Ok(Client::from_conf(builder.build()))
    }
}

fn parse_object_storage_path(file_path: &str) -> Option<(String, String)> {
    let normalized = file_path.trim();
    let trimmed = normalized
        .strip_prefix("ibm-cos://")
        .or_else(|| normalized.strip_prefix("s3://"))?;
    let mut parts = trimmed.splitn(2, '/');
    let bucket = parts.next()?.trim();
    let key = parts.next()?.trim();
    if bucket.is_empty() || key.is_empty() {
        return None;
    }

    Some((bucket.to_string(), key.to_string()))
}

fn sanitize_file_name(value: &str) -> String {
    let mut sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();

    if sanitized.trim_matches('_').is_empty() {
        sanitized = "document.bin".into();
    }

    sanitized
}
