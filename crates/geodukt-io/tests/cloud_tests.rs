//! Integration tests for cloud_io module.

use geodukt_io::cloud_io::{CloudBackend, create_store};

#[test]
fn test_create_s3_store() {
    let backend = CloudBackend::S3 {
        bucket: "test-bucket".into(),
        region: "us-east-1".into(),
    };
    // Should succeed in creating the store (doesn't connect)
    let store = create_store(&backend);
    assert!(store.is_ok());
}

#[test]
fn test_create_gcs_store() {
    let backend = CloudBackend::Gcs {
        bucket: "test-bucket".into(),
    };
    let store = create_store(&backend);
    assert!(store.is_ok());
}

#[test]
fn test_create_azure_store() {
    let backend = CloudBackend::Azure {
        container: "test-container".into(),
        account: "testaccount".into(),
    };
    let store = create_store(&backend);
    assert!(store.is_ok());
}
