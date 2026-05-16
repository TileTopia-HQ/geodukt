//! # geodukt-plugins
//!
//! Dynamic plugin system for loading custom transforms at runtime.

use std::collections::HashMap;
use std::path::Path;

use geodukt_core::feature::FeatureCollection;
use geodukt_core::pipeline::{PipelineError, TransformOp};
use libloading::{Library, Symbol};

/// Type signature for plugin transform functions.
/// Plugins export: `extern "C" fn geodukt_transform(input: &[u8], params: &[u8]) -> Vec<u8>`
/// Where input/output are JSON-serialized FeatureCollections.
type TransformFn =
    unsafe extern "C" fn(*const u8, usize, *const u8, usize, *mut u8, *mut usize) -> i32;

/// A dynamically loaded plugin transform.
pub struct PluginTransform {
    _library: Library,
    transform_fn: TransformFn,
}

impl PluginTransform {
    /// Load a plugin from a shared library path.
    ///
    /// # Safety
    /// The shared library must export a `geodukt_transform` function with the correct signature.
    pub unsafe fn load(path: &Path) -> Result<Self, PipelineError> {
        let library = unsafe {
            Library::new(path).map_err(|e| PipelineError::Transform {
                name: "plugin".into(),
                message: format!("failed to load plugin: {e}"),
            })?
        };

        let transform_fn: Symbol<TransformFn> = unsafe {
            library
                .get(b"geodukt_transform")
                .map_err(|e| PipelineError::Transform {
                    name: "plugin".into(),
                    message: format!("plugin missing 'geodukt_transform' symbol: {e}"),
                })?
        };

        let transform_fn = *transform_fn;

        Ok(Self {
            _library: library,
            transform_fn,
        })
    }
}

impl TransformOp for PluginTransform {
    fn apply(
        &self,
        input: &FeatureCollection,
        params: &HashMap<String, toml::Value>,
    ) -> Result<FeatureCollection, PipelineError> {
        // Serialize input to JSON bytes
        let input_json = serde_json::to_vec(&SerializableFc::from(input)).map_err(|e| {
            PipelineError::Transform {
                name: "plugin".into(),
                message: format!("serialization error: {e}"),
            }
        })?;

        let params_json = serde_json::to_vec(params).map_err(|e| PipelineError::Transform {
            name: "plugin".into(),
            message: format!("params serialization error: {e}"),
        })?;

        // Allocate output buffer
        let mut output_buf = vec![0u8; 64 * 1024 * 1024]; // 64MB max
        let mut output_len: usize = 0;

        let result = unsafe {
            (self.transform_fn)(
                input_json.as_ptr(),
                input_json.len(),
                params_json.as_ptr(),
                params_json.len(),
                output_buf.as_mut_ptr(),
                &mut output_len,
            )
        };

        if result != 0 {
            return Err(PipelineError::Transform {
                name: "plugin".into(),
                message: format!("plugin returned error code: {result}"),
            });
        }

        output_buf.truncate(output_len);
        let fc: SerializableFc =
            serde_json::from_slice(&output_buf).map_err(|e| PipelineError::Transform {
                name: "plugin".into(),
                message: format!("plugin output deserialization error: {e}"),
            })?;

        Ok(fc.into())
    }
}

/// Plugin registry for managing loaded plugins.
pub struct PluginRegistry {
    plugins: HashMap<String, PluginTransform>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Load all plugins from a directory.
    ///
    /// # Safety
    /// All shared libraries in the directory must be trusted.
    pub unsafe fn load_from_dir(&mut self, dir: &Path) -> Result<Vec<String>, PipelineError> {
        let mut loaded = Vec::new();
        if !dir.exists() {
            return Ok(loaded);
        }

        for entry in std::fs::read_dir(dir).map_err(PipelineError::Io)? {
            let entry = entry.map_err(PipelineError::Io)?;
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "so" || ext == "dylib" || ext == "dll" {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let plugin = unsafe { PluginTransform::load(&path)? };
                self.plugins.insert(name.clone(), plugin);
                loaded.push(name);
            }
        }
        Ok(loaded)
    }

    /// Get a plugin transform by name.
    pub fn get(&self, name: &str) -> Option<&PluginTransform> {
        self.plugins.get(name)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable wrapper for FeatureCollection (JSON interchange with plugins).
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SerializableFc {
    features: Vec<SerializableFeature>,
    crs: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SerializableFeature {
    geometry_wkt: String,
    properties: HashMap<String, geodukt_core::feature::Value>,
}

impl From<&FeatureCollection> for SerializableFc {
    fn from(fc: &FeatureCollection) -> Self {
        Self {
            features: fc
                .features
                .iter()
                .map(|f| SerializableFeature {
                    geometry_wkt: format!("{:?}", f.geometry), // Simplified; real impl would use WKT
                    properties: f.properties.clone(),
                })
                .collect(),
            crs: fc.crs.clone(),
        }
    }
}

impl From<SerializableFc> for FeatureCollection {
    fn from(sfc: SerializableFc) -> Self {
        // Simplified: in real impl, parse WKT back to geometry
        FeatureCollection::new(Vec::new(), sfc.crs)
    }
}
