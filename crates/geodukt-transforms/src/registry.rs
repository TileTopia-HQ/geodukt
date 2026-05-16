//! Transform registry — maps operation names to transform implementations.

use std::collections::HashMap;

use geodukt_core::pipeline::TransformOp;

use crate::buffer::BufferTransform;
use crate::centroid::CentroidTransform;
use crate::clip::ClipTransform;
use crate::dissolve::DissolveTransform;
use crate::expression::ExpressionTransform;
use crate::filter::FilterTransform;
use crate::reproject::ReprojectTransform;
use crate::schema::SchemaMapTransform;
use crate::simplify::SimplifyTransform;
use crate::spatial_join::SpatialJoinTransform;

/// Build the default transform registry with all built-in operations.
pub fn default_registry() -> HashMap<String, Box<dyn TransformOp>> {
    let mut registry: HashMap<String, Box<dyn TransformOp>> = HashMap::new();
    registry.insert("buffer".into(), Box::new(BufferTransform));
    registry.insert("centroid".into(), Box::new(CentroidTransform));
    registry.insert("clip".into(), Box::new(ClipTransform::new()));
    registry.insert("dissolve".into(), Box::new(DissolveTransform));
    registry.insert("expression".into(), Box::new(ExpressionTransform));
    registry.insert("filter".into(), Box::new(FilterTransform));
    registry.insert("reproject".into(), Box::new(ReprojectTransform));
    registry.insert("schema_map".into(), Box::new(SchemaMapTransform));
    registry.insert("simplify".into(), Box::new(SimplifyTransform));
    registry.insert("spatial_join".into(), Box::new(SpatialJoinTransform::new()));
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry() {
        let reg = default_registry();
        assert!(reg.contains_key("buffer"));
        assert!(reg.contains_key("centroid"));
        assert!(reg.contains_key("clip"));
        assert!(reg.contains_key("dissolve"));
        assert!(reg.contains_key("expression"));
        assert!(reg.contains_key("filter"));
        assert!(reg.contains_key("reproject"));
        assert!(reg.contains_key("schema_map"));
        assert!(reg.contains_key("simplify"));
        assert!(reg.contains_key("spatial_join"));
        assert_eq!(reg.len(), 10);
    }
}
