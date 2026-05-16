//! Template/macro system — reusable transform snippets.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::manifest::Transform;

/// A reusable transform template with parameterized values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Vec<TemplateParam>,
    pub transforms: Vec<Transform>,
}

/// A parameter definition for a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParam {
    pub name: String,
    pub description: Option<String>,
    pub default: Option<toml::Value>,
    pub required: bool,
}

/// Template registry — stores loaded templates.
#[derive(Debug, Default)]
pub struct TemplateRegistry {
    pub templates: HashMap<String, Template>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load templates from a directory (each .toml file is a template).
    pub fn load_from_dir(&mut self, dir: &Path) -> std::io::Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml")
                && let Some(template) = fs::read_to_string(&path)
                    .ok()
                    .and_then(|content| toml::from_str::<Template>(&content).ok())
            {
                self.templates.insert(template.name.clone(), template);
            }
        }
        Ok(())
    }

    /// Register a template.
    pub fn register(&mut self, template: Template) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Expand a template with given arguments, producing concrete transforms.
    pub fn expand(
        &self,
        template_name: &str,
        args: &HashMap<String, toml::Value>,
        prefix: &str,
    ) -> Option<Vec<Transform>> {
        let template = self.templates.get(template_name)?;

        let transforms: Vec<Transform> = template
            .transforms
            .iter()
            .map(|t| {
                let mut expanded = t.clone();
                // Prefix transform names to avoid collisions
                expanded.name = format!("{prefix}_{}", t.name);
                if !t.input.is_empty() {
                    // Check if input refers to another template transform
                    let is_internal = template
                        .transforms
                        .iter()
                        .any(|other| other.name == t.input);
                    if is_internal {
                        expanded.input = format!("{prefix}_{}", t.input);
                    }
                }
                // Substitute template parameters in params
                for value in expanded.params.values_mut() {
                    if let toml::Value::String(s) = value
                        && let Some(param_name) =
                            s.strip_prefix("{{").and_then(|s| s.strip_suffix("}}"))
                    {
                        let param_name = param_name.trim();
                        if let Some(arg_value) = args.get(param_name) {
                            *value = arg_value.clone();
                        } else if let Some(default) = template
                            .parameters
                            .iter()
                            .find(|p| p.name == param_name)
                            .and_then(|p| p.default.clone())
                        {
                            *value = default;
                        }
                    }
                }
                expanded
            })
            .collect();

        Some(transforms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Transform;

    #[test]
    fn test_template_expansion() {
        let mut registry = TemplateRegistry::new();
        registry.register(Template {
            name: "buffer_and_simplify".into(),
            description: Some("Buffer then simplify".into()),
            parameters: vec![TemplateParam {
                name: "distance".into(),
                description: None,
                default: Some(toml::Value::Float(1.0)),
                required: false,
            }],
            transforms: vec![
                Transform {
                    name: "buffered".into(),
                    input: "".into(), // Will be set by caller
                    operation: "buffer".into(),
                    params: HashMap::from([(
                        "distance".into(),
                        toml::Value::String("{{distance}}".into()),
                    )]),
                },
                Transform {
                    name: "simplified".into(),
                    input: "buffered".into(),
                    operation: "simplify".into(),
                    params: HashMap::from([("epsilon".into(), toml::Value::Float(0.001))]),
                },
            ],
        });

        let args = HashMap::from([("distance".into(), toml::Value::Float(5.0))]);
        let expanded = registry
            .expand("buffer_and_simplify", &args, "step1")
            .unwrap();
        assert_eq!(expanded.len(), 2);
        assert_eq!(expanded[0].name, "step1_buffered");
        assert_eq!(expanded[1].name, "step1_simplified");
        assert_eq!(expanded[1].input, "step1_buffered");
        assert_eq!(
            expanded[0].params.get("distance"),
            Some(&toml::Value::Float(5.0))
        );
    }
}
