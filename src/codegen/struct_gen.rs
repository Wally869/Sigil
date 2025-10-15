use crate::semantic::AnalyzedPrompt;
use crate::util::param_name_to_field_name;

/// Generate the main struct definition
pub fn generate_struct(analyzed: &AnalyzedPrompt) -> String {
    let mut code = String::new();

    // Add doc comment if description exists
    if let Some(description) = &analyzed.prompt_file.description {
        code.push_str(&format!("/// {}\n", description));
    }

    // Struct definition
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str(&format!("pub struct {} {{\n", analyzed.prompt_file.prompt_name));

    // Sort parameters by name for consistent output
    let mut params: Vec<_> = analyzed.parameters.values().collect();
    params.sort_by(|a, b| a.name.cmp(&b.name));

    // Add fields
    for param in params {
        let field_name = param_name_to_field_name(&param.name);
        let type_str = param.rust_type.as_str();
        code.push_str(&format!("    pub {}: {},\n", field_name, type_str));
    }

    code.push_str("}\n\n");

    // Add impl with builder method
    code.push_str(&format!("impl {} {{\n", analyzed.prompt_file.prompt_name));
    code.push_str(&format!(
        "    pub fn builder() -> {}Builder {{\n",
        analyzed.prompt_file.prompt_name
    ));
    code.push_str(&format!(
        "        {}Builder::default()\n",
        analyzed.prompt_file.prompt_name
    ));
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
    fn test_generate_simple_struct() {
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

        let prompt_file = PromptFile::new(
            "TestPrompt".to_string(),
            Some("A test prompt".to_string()),
            vec![],
            Span::zero(),
        );

        let analyzed = AnalyzedPrompt::new(prompt_file, params);

        let code = generate_struct(&analyzed);

        assert!(code.contains("/// A test prompt"));
        assert!(code.contains("pub struct TestPrompt"));
        assert!(code.contains("pub name: String"));
        assert!(code.contains("pub fn builder() -> TestPromptBuilder"));
    }

    #[test]
    fn test_generate_struct_with_optional_field() {
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

        let code = generate_struct(&analyzed);

        assert!(code.contains("pub email: Option<String>"));
    }

    #[test]
    fn test_generate_struct_with_vec_field() {
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

        let code = generate_struct(&analyzed);

        assert!(code.contains("pub items: Vec<String>"));
    }
}
