//! Integration tests for geodukt-io formats.

use std::collections::HashMap;

use geodukt_core::feature::{Feature, FeatureCollection, Value};
use geodukt_io::geopackage_io::{read_geopackage, write_geopackage};

#[test]
fn test_geopackage_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.gpkg");

    let mut props = HashMap::new();
    props.insert("name".into(), Value::String("hello".into()));
    props.insert("count".into(), Value::String("42".into()));

    let features = vec![
        Feature {
            geometry: geo::Geometry::Point(geo::Point::new(1.0, 2.0)),
            properties: props.clone(),
        },
        Feature {
            geometry: geo::Geometry::Point(geo::Point::new(3.0, 4.0)),
            properties: props.clone(),
        },
    ];

    let fc = FeatureCollection::new(features, None);
    write_geopackage(&path, &fc, "test_layer").unwrap();

    let result = read_geopackage(&path, Some("test_layer")).unwrap();
    assert_eq!(result.features.len(), 2);
    assert_eq!(
        result.features[0].properties.get("name"),
        Some(&Value::String("hello".into()))
    );
    assert_eq!(
        result.features[1].properties.get("count"),
        Some(&Value::String("42".into()))
    );
}

#[test]
fn test_geopackage_auto_table_detection() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auto.gpkg");

    let mut props = HashMap::new();
    props.insert("id".into(), Value::String("1".into()));

    let fc = FeatureCollection::new(
        vec![Feature {
            geometry: geo::Geometry::Point(geo::Point::new(0.0, 0.0)),
            properties: props,
        }],
        None,
    );
    write_geopackage(&path, &fc, "auto_layer").unwrap();

    // Read without specifying table name
    let result = read_geopackage(&path, None).unwrap();
    assert_eq!(result.features.len(), 1);
}
