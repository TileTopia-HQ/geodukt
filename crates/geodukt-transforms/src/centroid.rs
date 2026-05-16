//! Centroid transform — replaces geometries with their centroid point.

use std::collections::HashMap;

use geo::{Centroid, Geometry};
use geodukt_core::feature::{Feature, FeatureCollection};
use geodukt_core::pipeline::{PipelineError, TransformOp};

/// Centroid operation: replaces each geometry with its centroid.
pub struct CentroidTransform;

impl TransformOp for CentroidTransform {
    fn apply(
        &self,
        input: &FeatureCollection,
        _params: &HashMap<String, toml::Value>,
    ) -> Result<FeatureCollection, PipelineError> {
        let features: Vec<Feature> = input
            .features
            .iter()
            .filter_map(|f| {
                let centroid = f.geometry.centroid()?;
                Some(Feature {
                    geometry: Geometry::Point(centroid),
                    properties: f.properties.clone(),
                })
            })
            .collect();

        Ok(FeatureCollection::new(features, input.crs.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{Geometry, polygon};
    use geodukt_core::feature::Value;

    #[test]
    fn test_centroid_polygon() {
        let poly = polygon![
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 0.0, y: 4.0),
            (x: 0.0, y: 0.0),
        ];
        let features = vec![Feature {
            geometry: Geometry::Polygon(poly),
            properties: HashMap::from([("name".into(), Value::String("square".into()))]),
        }];
        let fc = FeatureCollection::new(features, None);

        let result = CentroidTransform.apply(&fc, &HashMap::new()).unwrap();
        assert_eq!(result.len(), 1);
        if let Geometry::Point(p) = &result.features[0].geometry {
            assert!((p.x() - 2.0).abs() < f64::EPSILON);
            assert!((p.y() - 2.0).abs() < f64::EPSILON);
        } else {
            panic!("expected Point geometry");
        }
    }
}
