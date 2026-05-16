//! Transform registry — maps operation names to transform implementations.

use std::collections::HashMap;

use geodukt_core::pipeline::TransformOp;

use crate::buffer::BufferTransform;
use crate::centroid::CentroidTransform;
use crate::filter::FilterTransform;

/// Build the default transform registry with all built-in operations.
pub fn default_registry() -> HashMap<String, Box<dyn TransformOp>> {
    let mut registry: HashMap<String, Box<dyn TransformOp>> = HashMap::new();
    registry.insert("buffer".into(), Box::new(BufferTransform));
    registry.insert("centroid".into(), Box::new(CentroidTransform));
    registry.insert("filter".into(), Box::new(FilterTransform));
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
        assert!(reg.contains_key("filter"));
        assert_eq!(reg.len(), 3);
    }
}
