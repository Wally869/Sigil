use crate::error::{Result, SigilError, Span};
use crate::parser::{
    ContentItem, Parameter, ParameterKind, RenderAttrValue, RenderType, Section,
};
use std::collections::HashMap;

/// Rust type for a parameter
#[derive(Debug, Clone, PartialEq)]
pub enum RustType {
    String,
    OptionString,
    VecString,
}

impl RustType {
    pub fn as_str(&self) -> &str {
        match self {
            RustType::String => "String",
            RustType::OptionString => "Option<String>",
            RustType::VecString => "Vec<String>",
        }
    }
}

/// Information about a parameter after type inference
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub rust_type: RustType,
    pub is_required: bool,
    pub default_value: Option<String>,
    pub render_type: Option<RenderType>,
    pub first_occurrence: Span,
}

impl ParameterInfo {
    pub fn new(name: String, first_occurrence: Span) -> Self {
        Self {
            name,
            rust_type: RustType::String,
            is_required: true,
            default_value: None,
            render_type: None,
            first_occurrence,
        }
    }
}

/// Type checker for analyzing parameters
pub struct TypeChecker {
    parameters: HashMap<String, ParameterInfo>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }

    /// Analyze all parameters in sections
    pub fn analyze_sections(&mut self, sections: &[Section]) -> Result<()> {
        // First pass: collect all parameters and their usages
        for section in sections {
            self.analyze_section(section)?;
        }

        // Second pass: infer types based on all usages
        for section in sections {
            self.infer_types(section)?;
        }

        // Third pass: validate consistency
        self.validate_consistency(sections)?;

        Ok(())
    }

    /// Analyze a single section
    fn analyze_section(&mut self, section: &Section) -> Result<()> {
        let is_optional_section = section.is_optional();

        for item in &section.content.items {
            if let ContentItem::Parameter(param) = item {
                self.register_parameter(param, is_optional_section)?;
            }
        }

        Ok(())
    }

    /// Register a parameter
    fn register_parameter(&mut self, param: &Parameter, in_optional_section: bool) -> Result<()> {
        if let Some(info) = self.parameters.get_mut(&param.name) {
            // Parameter already exists, check for type conflicts
            match &param.kind {
                ParameterKind::Plain => {
                    // If this is in a required section and param was optional, upgrade to required
                    if !in_optional_section && !info.is_required {
                        info.is_required = true;
                    }
                }

                ParameterKind::WithDefault(default) => {
                    // Check for multiple different defaults
                    if let Some(existing_default) = &info.default_value {
                        if existing_default != default {
                            return Err(SigilError::MultipleDefaults {
                                param_name: param.name.clone(),
                                first_span: info.first_occurrence,
                                second_span: param.span,
                            });
                        }
                    } else {
                        info.default_value = Some(default.clone());
                        info.is_required = false;
                    }
                }

                ParameterKind::WithRenderType { render_type, .. } => {
                    // Check for type conflict
                    if let Some(existing_render_type) = &info.render_type {
                        if existing_render_type != render_type {
                            return Err(SigilError::TypeConflict {
                                param_name: param.name.clone(),
                                first_type: format!("{:?}", existing_render_type),
                                first_span: info.first_occurrence,
                                second_type: format!("{:?}", render_type),
                                second_span: param.span,
                            });
                        }
                    } else {
                        info.render_type = Some(render_type.clone());
                        if !in_optional_section {
                            info.is_required = true;
                        }
                    }
                }
            }
        } else {
            // New parameter
            let mut info = ParameterInfo::new(param.name.clone(), param.span);

            match &param.kind {
                ParameterKind::Plain => {
                    info.is_required = !in_optional_section;
                }

                ParameterKind::WithDefault(default) => {
                    info.default_value = Some(default.clone());
                    info.is_required = false;
                }

                ParameterKind::WithRenderType { render_type, .. } => {
                    info.render_type = Some(render_type.clone());
                    info.is_required = !in_optional_section;
                }
            }

            self.parameters.insert(param.name.clone(), info);
        }

        Ok(())
    }

    /// Infer Rust types for parameters
    fn infer_types(&mut self, section: &Section) -> Result<()> {
        for item in &section.content.items {
            if let ContentItem::Parameter(param) = item {
                if let Some(info) = self.parameters.get_mut(&param.name) {
                    // Determine Rust type based on render type
                    if let ParameterKind::WithRenderType { render_type, .. } = &param.kind {
                        let rust_type = match render_type {
                            RenderType::List => RustType::VecString,
                            _ => {
                                if info.is_required {
                                    RustType::String
                                } else {
                                    RustType::OptionString
                                }
                            }
                        };

                        // Check for type conflict
                        if info.rust_type != rust_type && info.rust_type != RustType::String {
                            return Err(SigilError::TypeConflict {
                                param_name: param.name.clone(),
                                first_type: info.rust_type.as_str().to_string(),
                                first_span: info.first_occurrence,
                                second_type: rust_type.as_str().to_string(),
                                second_span: param.span,
                            });
                        }

                        info.rust_type = rust_type;
                    } else {
                        // Update type based on required/optional status
                        info.rust_type = if info.is_required {
                            RustType::String
                        } else {
                            RustType::OptionString
                        };
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate consistency across all sections
    fn validate_consistency(&self, sections: &[Section]) -> Result<()> {
        // Check for duplicate section names
        let mut section_names: HashMap<String, Span> = HashMap::new();

        for section in sections {
            if let Some(first_span) = section_names.get(&section.name) {
                return Err(SigilError::DuplicateSection {
                    section_name: section.name.clone(),
                    first_span: *first_span,
                    second_span: section.span,
                });
            }
            section_names.insert(section.name.clone(), section.span);
        }

        // Validate that list types are consistent
        for section in sections {
            for item in &section.content.items {
                if let ContentItem::Parameter(param) = item {
                    if let Some(info) = self.parameters.get(&param.name) {
                        // If this parameter is a list type, verify it's not used as plain elsewhere
                        if info.rust_type == RustType::VecString {
                            if !matches!(
                                &param.kind,
                                ParameterKind::WithRenderType {
                                    render_type: RenderType::List,
                                    ..
                                }
                            ) {
                                return Err(SigilError::TypeConflict {
                                    param_name: param.name.clone(),
                                    first_type: "Vec<String>".to_string(),
                                    first_span: info.first_occurrence,
                                    second_type: "String".to_string(),
                                    second_span: param.span,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get analyzed parameter information
    pub fn get_parameters(&self) -> &HashMap<String, ParameterInfo> {
        &self.parameters
    }

    /// Extract parameters from render attributes as well
    pub fn extract_attribute_parameters(&mut self, sections: &[Section]) -> Result<()> {
        for section in sections {
            for item in &section.content.items {
                if let ContentItem::Parameter(param) = item {
                    if let ParameterKind::WithRenderType { attributes, .. } = &param.kind {
                        for attr in attributes {
                            if let RenderAttrValue::ParamRef { name, default } = &attr.value {
                                // Register this parameter
                                let param_info = ParameterInfo {
                                    name: name.clone(),
                                    rust_type: if default.is_some() {
                                        RustType::OptionString
                                    } else {
                                        RustType::String
                                    },
                                    is_required: default.is_none(),
                                    default_value: default.clone(),
                                    render_type: None,
                                    first_occurrence: attr.span,
                                };

                                if let Some(existing) = self.parameters.get(name) {
                                    // Check for default conflicts
                                    if let Some(existing_default) = &existing.default_value {
                                        if let Some(new_default) = default {
                                            if existing_default != new_default {
                                                return Err(SigilError::MultipleDefaults {
                                                    param_name: name.clone(),
                                                    first_span: existing.first_occurrence,
                                                    second_span: attr.span,
                                                });
                                            }
                                        }
                                    }
                                } else {
                                    self.parameters.insert(name.clone(), param_info);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SourceLocation;
    use crate::parser::{Section, SectionAttribute, SectionContent};

    fn make_span() -> Span {
        Span::new(SourceLocation::new(1, 1), SourceLocation::new(1, 10))
    }

    #[test]
    fn test_parameter_info_creation() {
        let info = ParameterInfo::new("test".to_string(), make_span());
        assert_eq!(info.name, "test");
        assert!(info.is_required);
        assert!(info.default_value.is_none());
    }

    #[test]
    fn test_rust_type_display() {
        assert_eq!(RustType::String.as_str(), "String");
        assert_eq!(RustType::OptionString.as_str(), "Option<String>");
        assert_eq!(RustType::VecString.as_str(), "Vec<String>");
    }

    #[test]
    fn test_type_checker_basic() {
        let mut checker = TypeChecker::new();

        let param = Parameter::new("name".to_string(), ParameterKind::Plain, make_span());

        checker.register_parameter(&param, false).unwrap();

        assert_eq!(checker.parameters.len(), 1);
        assert!(checker.parameters.contains_key("name"));
    }

    #[test]
    fn test_type_checker_duplicate_defaults_error() {
        let mut checker = TypeChecker::new();

        let param1 = Parameter::new(
            "name".to_string(),
            ParameterKind::WithDefault("default1".to_string()),
            make_span(),
        );
        let param2 = Parameter::new(
            "name".to_string(),
            ParameterKind::WithDefault("default2".to_string()),
            make_span(),
        );

        checker.register_parameter(&param1, false).unwrap();
        let result = checker.register_parameter(&param2, false);

        assert!(result.is_err());
        match result.unwrap_err() {
            SigilError::MultipleDefaults { .. } => {}
            _ => panic!("Expected MultipleDefaults error"),
        }
    }
}
