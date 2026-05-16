//! Cloud object store I/O — S3, GCS, Azure Blob.

use std::sync::Arc;

use bytes::Bytes;
use geodukt_core::pipeline::PipelineError;
use object_store::path::Path as ObjectPath;
use object_store::{ObjectStore, PutPayload};

/// Supported cloud backends.
#[derive(Debug, Clone)]
pub enum CloudBackend {
    S3 { bucket: String, region: String },
    Gcs { bucket: String },
    Azure { container: String, account: String },
}

/// Create an object store client for the given backend.
pub fn create_store(backend: &CloudBackend) -> Result<Arc<dyn ObjectStore>, PipelineError> {
    match backend {
        CloudBackend::S3 { bucket, region } => {
            let store = object_store::aws::AmazonS3Builder::new()
                .with_bucket_name(bucket)
                .with_region(region)
                .build()
                .map_err(|e| PipelineError::Source {
                    name: "cloud_s3".into(),
                    message: e.to_string(),
                })?;
            Ok(Arc::new(store))
        }
        CloudBackend::Gcs { bucket } => {
            let store = object_store::gcp::GoogleCloudStorageBuilder::new()
                .with_bucket_name(bucket)
                .build()
                .map_err(|e| PipelineError::Source {
                    name: "cloud_gcs".into(),
                    message: e.to_string(),
                })?;
            Ok(Arc::new(store))
        }
        CloudBackend::Azure { container, account } => {
            let store = object_store::azure::MicrosoftAzureBuilder::new()
                .with_container_name(container)
                .with_account(account)
                .build()
                .map_err(|e| PipelineError::Source {
                    name: "cloud_azure".into(),
                    message: e.to_string(),
                })?;
            Ok(Arc::new(store))
        }
    }
}

/// Read bytes from a cloud object.
pub async fn cloud_read(store: &dyn ObjectStore, key: &str) -> Result<Bytes, PipelineError> {
    let path = ObjectPath::from(key);
    let result = store.get(&path).await.map_err(|e| PipelineError::Source {
        name: "cloud".into(),
        message: e.to_string(),
    })?;
    result.bytes().await.map_err(|e| PipelineError::Source {
        name: "cloud".into(),
        message: e.to_string(),
    })
}

/// Write bytes to a cloud object.
pub async fn cloud_write(
    store: &dyn ObjectStore,
    key: &str,
    data: Bytes,
) -> Result<(), PipelineError> {
    let path = ObjectPath::from(key);
    let payload = PutPayload::from_bytes(data);
    store
        .put(&path, payload)
        .await
        .map_err(|e| PipelineError::Sink {
            name: "cloud".into(),
            message: e.to_string(),
        })?;
    Ok(())
}
