//! Filter transform — filters features by property predicates.

use std::collections::HashMap;

use geodukt_core::feature::{Feature, FeatureCollection, Value};
use geodukt_core::pipeline::{PipelineError, TransformOp};

/// Filter operation: keeps only features matching a property condition.
pub struct FilterTransform;

impl TransformOp for FilterTransform {
    fn apply(
        &self,
        input: &FeatureCollection,
        params: &HashMap<String, toml::Value>,
    ) -> Result<FeatureCollection, PipelineError> {
        let field = params
            .get("field")
            .and_then(|v: &toml::Value| v.as_str())
            .unwrap_or("");
        let equals = params.get("equals");

        let features: Vec<Feature> = input
            .features
            .iter()
            .filter(|f| {
                if field.is_empty() {
                    return true;
                }
                match (f.properties.get(field), equals) {
                    (Some(Value::String(s)), Some(toml::Value::String(expected))) => s == expected,
                    (Some(Value::Integer(n)), Some(toml::Value::Integer(expected))) => {
                        n == expected
                    }
                    (Some(Value::Float(n)), Some(toml::Value::Float(expected))) => {
                        (n - expected).abs() < f64::EPSILON
                    }
                    (Some(Value::Bool(b)), Some(toml::Value::Boolean(expected))) => b == expected,
                    _ => false,
                }
            })
            .cloned()
            .collect();

        Ok(FeatureCollection::new(features, input.crs.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{Geometry, point};

    #[test]
    fn test_filter_by_string() {
        let features = vec![
            Feature {
                geometry: Geometry::Point(point!(x: 0.0, y: 0.0)),
                properties: HashMap::from([("type".into(), Value::String("road".into()))]),
            },
            Feature {
                geometry: Geometry::Point(point!(x: 1.0, y: 1.0)),
                properties: HashMap::from([("type".into(), Value::String("building".into()))]),
            },
        ];
        let fc = FeatureCollection::new(features, None);
        let params = HashMap::from([
            ("field".into(), toml::Value::String("type".into())),
            ("equals".into(), toml::Value::String("road".into())),
        ]);

        let result = FilterTransform.apply(&fc, &params).unwrap();
        assert_eq!(result.len(), 1);
    }
}
