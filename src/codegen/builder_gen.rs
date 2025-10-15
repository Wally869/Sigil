use crate::semantic::{AnalyzedPrompt, RustType};
use crate::util::{escape_rust_string, param_name_to_field_name};

/// Generate the builder struct and implementation
pub fn generate_builder(analyzed: &AnalyzedPrompt) -> String {
    let mut code = String::new();
    let struct_name = &analyzed.prompt_file.prompt_name;
    let builder_name = format!("{}Builder", struct_name);

    // Sort parameters by name for consistent output
    let mut params: Vec<_> = analyzed.parameters.values().collect();
    params.sort_by(|a, b| a.name.cmp(&b.name));

    // Builder struct definition
    code.push_str("#[derive(Default)]\n");
    code.push_str(&format!("pub struct {} {{\n", builder_name));

    // All fields in builder are Option<T>
    for param in &params {
        let field_name = param_name_to_field_name(&param.name);
        let field_type = match param.rust_type {
            RustType::String | RustType::OptionString => "Option<String>",
            RustType::VecString => "Option<Vec<String>>",
        };
        code.push_str(&format!("    {}: {},\n", field_name, field_type));
    }

    code.push_str("}\n\n");

    // Builder implementation
    code.push_str(&format!("impl {} {{\n", builder_name));

    // Generate setter methods
    for param in &params {
        let field_name = param_name_to_field_name(&param.name);

        match param.rust_type {
            RustType::String | RustType::OptionString => {
                // Regular setter for String/Option<String>
                code.push_str(&format!(
                    "    pub fn {}(mut self, value: impl Into<String>) -> Self {{\n",
                    field_name
                ));
                code.push_str(&format!("        self.{} = Some(value.into());\n", field_name));
                code.push_str("        self\n");
                code.push_str("    }\n\n");
            }

            RustType::VecString => {
                // add_item method for Vec<String>
                let method_name = format!("add_{}", field_name);
                code.push_str(&format!(
                    "    pub fn {}(mut self, item: impl Into<String>) -> Self {{\n",
                    method_name
                ));
                code.push_str(&format!(
                    "        self.{}.get_or_insert_with(Vec::new).push(item.into());\n",
                    field_name
                ));
                code.push_str("        self\n");
                code.push_str("    }\n\n");
            }
        }
    }

    // Generate build() method
    code.push_str(&format!(
        "    pub fn build(self) -> Result<{}, &'static str> {{\n",
        struct_name
    ));
    code.push_str(&format!("        Ok({} {{\n", struct_name));

    for param in &params {
        let field_name = param_name_to_field_name(&param.name);

        match param.rust_type {
            RustType::String => {
                // Required String field
                code.push_str(&format!(
                    "            {}: self.{}.ok_or(\"{} is required\")?,\n",
                    field_name, field_name, param.name
                ));
            }

            RustType::OptionString => {
                // Optional String field
                if let Some(default) = &param.default_value {
                    let escaped_default = escape_rust_string(default);
                    code.push_str(&format!(
                        "            {}: self.{}.or(Some(\"{}\".to_string())),\n",
                        field_name, field_name, escaped_default
                    ));
                } else {
                    code.push_str(&format!("            {}: self.{},\n", field_name, field_name));
                }
            }

            RustType::VecString => {
                // Vec field - default to empty vec if not provided
                code.push_str(&format!(
                    "            {}: self.{}.unwrap_or_default(),\n",
                    field_name, field_name
                ));
            }
        }
    }

    code.push_str("        })\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Span;
    use crate::parser::*;
    use crate::semantic::{AnalyzedPrompt, ParameterInfo, RustType};
    use std::collections::HashMap;

    #[test]
    fn test_generate_builder_with_required_field() {
        let mut params = HashMap::new();
        params.insert(
            "name".to_string(),
            ParameterInfo {
                name: "name".to_string(),
                rust_type: RustType::String,
                is_required: true,
                default_value: None,
                render_type: None,
                first_occurrence: Span::zero(),
            },
        );

        let prompt_file = PromptFile::new("Test".to_string(), None, vec![], Span::zero());
        let analyzed = AnalyzedPrompt::new(prompt_file, params);

        let code = generate_builder(&analyzed);

        assert!(code.contains("pub struct TestBuilder"));
        assert!(code.contains("name: Option<String>"));
        assert!(code.contains("pub fn name(mut self, value: impl Into<String>) -> Self"));
        assert!(code.contains("pub fn build(self) -> Result<Test, &'static str>"));
        assert!(code.contains(r#"self.name.ok_or("name is required")?"#));
    }

    #[test]
    fn test_generate_builder_with_optional_field() {
        let mut params = HashMap::new();
        params.insert(
            "email".to_string(),
            ParameterInfo {
                name: "email".to_string(),
                rust_type: RustType::OptionString,
                is_required: false,
                default_value: None,
                render_type: None,
                first_occurrence: Span::zero(),
            },
        );

        let prompt_file = PromptFile::new("Test".to_string(), None, vec![], Span::zero());
        let analyzed = AnalyzedPrompt::new(prompt_file, params);

        let code = generate_builder(&analyzed);

        assert!(code.contains("pub fn email(mut self, value: impl Into<String>) -> Self"));
        assert!(code.contains("email: self.email,"));
    }

    #[test]
    fn test_generate_builder_with_default() {
        let mut params = HashMap::new();
        params.insert(
            "format".to_string(),
            ParameterInfo {
                name: "format".to_string(),
                rust_type: RustType::OptionString,
                is_required: false,
                default_value: Some("json".to_string()),
                render_type: None,
                first_occurrence: Span::zero(),
            },
        );

        let prompt_file = PromptFile::new("Test".to_string(), None, vec![], Span::zero());
        let analyzed = AnalyzedPrompt::new(prompt_file, params);

        let code = generate_builder(&analyzed);

        assert!(code.contains(r#"self.format.or(Some("json".to_string()))"#));
    }

    #[test]
    fn test_generate_builder_with_list() {
        let mut params = HashMap::new();
        params.insert(
            "items".to_string(),
            ParameterInfo {
                name: "items".to_string(),
                rust_type: RustType::VecString,
                is_required: true,
                default_value: None,
                render_type: Some(RenderType::List),
                first_occurrence: Span::zero(),
            },
        );

        let prompt_file = PromptFile::new("Test".to_string(), None, vec![], Span::zero());
        let analyzed = AnalyzedPrompt::new(prompt_file, params);

        let code = generate_builder(&analyzed);

        assert!(code.contains("items: Option<Vec<String>>"));
        assert!(code.contains("pub fn add_items(mut self, item: impl Into<String>) -> Self"));
        assert!(code.contains("self.items.get_or_insert_with(Vec::new).push(item.into())"));
    }
}
