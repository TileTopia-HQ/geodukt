//! Buffer transform — expands/shrinks geometries by a distance.

use std::collections::HashMap;

use geo::{BoundingRect, Geometry, Polygon, Rect};
use geodukt_core::feature::{Feature, FeatureCollection};
use geodukt_core::pipeline::{PipelineError, TransformOp};

/// Buffer operation: expands point/line/polygon geometries.
pub struct BufferTransform;

impl TransformOp for BufferTransform {
    fn apply(
        &self,
        input: &FeatureCollection,
        params: &HashMap<String, toml::Value>,
    ) -> Result<FeatureCollection, PipelineError> {
        let distance = params
            .get("distance")
            .and_then(|v: &toml::Value| v.as_float())
            .unwrap_or(1.0);

        let features: Vec<Feature> = input
            .features
            .iter()
            .map(|f| {
                let buffered = buffer_geometry(&f.geometry, distance);
                Feature {
                    geometry: buffered,
                    properties: f.properties.clone(),
                }
            })
            .collect();

        Ok(FeatureCollection::new(features, input.crs.clone()))
    }
}

/// Simple buffer approximation using bounding rect expansion.
/// A proper implementation would use offset curves; this is a minimal placeholder.
fn buffer_geometry(geom: &Geometry<f64>, distance: f64) -> Geometry<f64> {
    match geom.bounding_rect() {
        Some(rect) => {
            let min = rect.min();
            let max = rect.max();
            let buffered = Rect::new(
                geo::Coord {
                    x: min.x - distance,
                    y: min.y - distance,
                },
                geo::Coord {
                    x: max.x + distance,
                    y: max.y + distance,
                },
            );
            Geometry::Polygon(Polygon::from(buffered))
        }
        None => geom.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::point;
    use geodukt_core::feature::Value;

    #[test]
    fn test_buffer_point() {
        let features = vec![Feature {
            geometry: Geometry::Point(point!(x: 0.0, y: 0.0)),
            properties: HashMap::from([("id".into(), Value::Integer(1))]),
        }];
        let fc = FeatureCollection::new(features, None);
        let params = HashMap::from([("distance".into(), toml::Value::Float(5.0))]);

        let result = BufferTransform.apply(&fc, &params).unwrap();
        assert_eq!(result.len(), 1);
        // Result should be a polygon (buffered bounding rect)
        assert!(matches!(result.features[0].geometry, Geometry::Polygon(_)));
    }
}
