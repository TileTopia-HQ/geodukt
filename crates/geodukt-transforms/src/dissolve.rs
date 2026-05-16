//! Dissolve transform — merge features by property value, unioning geometries.

use std::collections::HashMap;

use geo::{BooleanOps, Geometry, MultiPolygon, Polygon};
use geodukt_core::feature::{Feature, FeatureCollection, Value};
use geodukt_core::pipeline::{PipelineError, TransformOp};

/// Dissolve operation: groups features by a property key and unions their geometries.
pub struct DissolveTransform;

impl TransformOp for DissolveTransform {
    fn apply(
        &self,
        input: &FeatureCollection,
        params: &HashMap<String, toml::Value>,
    ) -> Result<FeatureCollection, PipelineError> {
        let group_by = params
            .get("group_by")
            .and_then(|v: &toml::Value| v.as_str())
            .unwrap_or("");

        // Group features by property value
        let mut groups: HashMap<String, Vec<&Feature>> = HashMap::new();
        for f in &input.features {
            let key = if group_by.is_empty() {
                "__all__".to_string()
            } else {
                match f.properties.get(group_by) {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Integer(n)) => n.to_string(),
                    Some(Value::Float(n)) => n.to_string(),
                    _ => "__null__".to_string(),
                }
            };
            groups.entry(key).or_default().push(f);
        }

        let features: Vec<Feature> = groups
            .into_iter()
            .filter_map(|(key, group)| {
                let merged = merge_geometries(&group)?;
                let mut props = HashMap::new();
                if !group_by.is_empty() {
                    props.insert(group_by.to_string(), Value::String(key));
                }
                props.insert("count".to_string(), Value::Integer(group.len() as i64));
                Some(Feature {
                    geometry: merged,
                    properties: props,
                })
            })
            .collect();

        Ok(FeatureCollection::new(features, input.crs.clone()))
    }
}

fn merge_geometries(features: &[&Feature]) -> Option<Geometry<f64>> {
    let polys: Vec<&Polygon<f64>> = features
        .iter()
        .filter_map(|f| match &f.geometry {
            Geometry::Polygon(p) => Some(p),
            _ => None,
        })
        .collect();

    if polys.is_empty() {
        // For non-polygon features, just return a multi-point/line collection
        return features.first().map(|f| f.geometry.clone());
    }

    // Union all polygons
    let mut result = MultiPolygon(vec![polys[0].clone()]);
    for poly in polys.iter().skip(1) {
        result = result.union(&MultiPolygon(vec![(*poly).clone()]));
    }

    if result.0.len() == 1 {
        Some(Geometry::Polygon(result.0[0].clone()))
    } else {
        Some(Geometry::MultiPolygon(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::polygon;

    #[test]
    fn test_dissolve_by_property() {
        let features = vec![
            Feature {
                geometry: Geometry::Polygon(polygon![
                    (x: 0.0, y: 0.0), (x: 1.0, y: 0.0), (x: 1.0, y: 1.0), (x: 0.0, y: 1.0), (x: 0.0, y: 0.0),
                ]),
                properties: HashMap::from([("zone".into(), Value::String("A".into()))]),
            },
            Feature {
                geometry: Geometry::Polygon(polygon![
                    (x: 1.0, y: 0.0), (x: 2.0, y: 0.0), (x: 2.0, y: 1.0), (x: 1.0, y: 1.0), (x: 1.0, y: 0.0),
                ]),
                properties: HashMap::from([("zone".into(), Value::String("A".into()))]),
            },
            Feature {
                geometry: Geometry::Polygon(polygon![
                    (x: 5.0, y: 5.0), (x: 6.0, y: 5.0), (x: 6.0, y: 6.0), (x: 5.0, y: 6.0), (x: 5.0, y: 5.0),
                ]),
                properties: HashMap::from([("zone".into(), Value::String("B".into()))]),
            },
        ];
        let fc = FeatureCollection::new(features, None);
        let params = HashMap::from([("group_by".into(), toml::Value::String("zone".into()))]);

        let result = DissolveTransform.apply(&fc, &params).unwrap();
        assert_eq!(result.len(), 2);
    }
}
