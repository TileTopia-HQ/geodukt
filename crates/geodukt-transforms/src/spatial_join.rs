//! Spatial join transform — joins features based on spatial relationships.

use std::collections::HashMap;

use geo::{Contains, Geometry, Intersects};
use geodukt_core::feature::{Feature, FeatureCollection};
use geodukt_core::pipeline::{PipelineError, TransformOp};

/// Spatial join operation: enriches features with properties from spatially related features.
/// Requires a secondary dataset loaded via `join_to` param.
#[derive(Default)]
pub struct SpatialJoinTransform {
    /// The dataset to join against.
    pub join_dataset: Option<FeatureCollection>,
}

impl SpatialJoinTransform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dataset(dataset: FeatureCollection) -> Self {
        Self {
            join_dataset: Some(dataset),
        }
    }
}

impl TransformOp for SpatialJoinTransform {
    fn apply(
        &self,
        input: &FeatureCollection,
        params: &HashMap<String, toml::Value>,
    ) -> Result<FeatureCollection, PipelineError> {
        let join_type = params
            .get("join_type")
            .and_then(|v: &toml::Value| v.as_str())
            .unwrap_or("intersects");

        let join_data = self
            .join_dataset
            .as_ref()
            .ok_or_else(|| PipelineError::Transform {
                name: "spatial_join".into(),
                message: "no join dataset provided".into(),
            })?;

        let features: Vec<Feature> = input
            .features
            .iter()
            .map(|f| {
                let mut props = f.properties.clone();
                // Find matching features from join dataset
                for jf in &join_data.features {
                    let matches = match join_type {
                        "contains" => geometry_contains(&f.geometry, &jf.geometry),
                        "within" => geometry_contains(&jf.geometry, &f.geometry),
                        _ => f.geometry.intersects(&jf.geometry),
                    };
                    if matches {
                        // Merge properties with prefix
                        for (k, v) in &jf.properties {
                            props.insert(format!("joined_{k}"), v.clone());
                        }
                        break; // Take first match
                    }
                }
                Feature {
                    geometry: f.geometry.clone(),
                    properties: props,
                }
            })
            .collect();

        Ok(FeatureCollection::new(features, input.crs.clone()))
    }
}

fn geometry_contains(container: &Geometry<f64>, contained: &Geometry<f64>) -> bool {
    match (container, contained) {
        (Geometry::Polygon(p), Geometry::Point(pt)) => p.contains(pt),
        (Geometry::MultiPolygon(mp), Geometry::Point(pt)) => mp.contains(pt),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{point, polygon};
    use geodukt_core::feature::Value;

    #[test]
    fn test_spatial_join_intersects() {
        let poly = polygon![
            (x: 0.0, y: 0.0),
            (x: 10.0, y: 0.0),
            (x: 10.0, y: 10.0),
            (x: 0.0, y: 10.0),
            (x: 0.0, y: 0.0),
        ];
        let join_fc = FeatureCollection::new(
            vec![Feature {
                geometry: Geometry::Polygon(poly),
                properties: HashMap::from([("zone".into(), Value::String("residential".into()))]),
            }],
            None,
        );

        let input_features = vec![
            Feature {
                geometry: Geometry::Point(point!(x: 5.0, y: 5.0)),
                properties: HashMap::from([("id".into(), Value::Integer(1))]),
            },
            Feature {
                geometry: Geometry::Point(point!(x: 20.0, y: 20.0)),
                properties: HashMap::from([("id".into(), Value::Integer(2))]),
            },
        ];
        let input_fc = FeatureCollection::new(input_features, None);

        let transform = SpatialJoinTransform::with_dataset(join_fc);
        let result = transform.apply(&input_fc, &HashMap::new()).unwrap();
        assert_eq!(result.len(), 2);
        // First point is inside polygon, should have joined property
        assert_eq!(
            result.features[0].properties.get("joined_zone"),
            Some(&Value::String("residential".into()))
        );
        // Second point is outside, no joined property
        assert!(!result.features[1].properties.contains_key("joined_zone"));
    }
}
