//! Pipeline manifest — TOML-based project definition.

use serde::{Deserialize, Serialize};

/// Top-level pipeline manifest (geodukt.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub project: Project,
    #[serde(default)]
    pub source: Vec<Source>,
    #[serde(default)]
    pub transform: Vec<Transform>,
    #[serde(default)]
    pub sink: Vec<Sink>,
}

/// Project metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// A data source node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub name: String,
    pub format: String,
    pub path: String,
    pub crs: Option<String>,
}

/// A transform node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub name: String,
    pub input: String,
    pub operation: String,
    /// Additional operation-specific parameters.
    #[serde(flatten)]
    pub params: std::collections::HashMap<String, toml::Value>,
}

/// A data sink (output) node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sink {
    pub name: String,
    pub input: String,
    pub format: String,
    pub path: String,
}

impl Manifest {
    /// Parse a manifest from TOML string.
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Serialize to TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let toml = r#"
[project]
name = "test-pipeline"
version = "0.1.0"

[[source]]
name = "buildings"
format = "geojson"
path = "data/buildings.geojson"

[[transform]]
name = "buildings_buffered"
input = "buildings"
operation = "buffer"
distance = 10.0

[[sink]]
name = "output"
input = "buildings_buffered"
format = "geojson"
path = "output/buffered.geojson"
"#;
        let manifest = Manifest::from_toml(toml).unwrap();
        assert_eq!(manifest.project.name, "test-pipeline");
        assert_eq!(manifest.source.len(), 1);
        assert_eq!(manifest.transform.len(), 1);
        assert_eq!(manifest.sink.len(), 1);
        assert_eq!(manifest.transform[0].operation, "buffer");
    }

    #[test]
    fn test_roundtrip() {
        let manifest = Manifest {
            project: Project {
                name: "roundtrip".into(),
                version: "1.0.0".into(),
            },
            source: vec![Source {
                name: "src".into(),
                format: "csv".into(),
                path: "data.csv".into(),
                crs: Some("EPSG:4326".into()),
            }],
            transform: vec![],
            sink: vec![],
        };
        let s = manifest.to_toml().unwrap();
        let parsed = Manifest::from_toml(&s).unwrap();
        assert_eq!(parsed.project.name, "roundtrip");
    }
}
