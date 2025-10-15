use crate::parser::{ContentItem, Parameter, ParameterKind, RenderAttrValue, RenderType};
use crate::semantic::{AnalyzedPrompt, RustType};
use crate::util::{
    escape_rust_string, param_name_to_field_name, snake_case_to_title_case, snake_case_to_upper,
};

/// Generate all three render methods (XML, Markdown, Plain)
pub fn generate_render_methods(analyzed: &AnalyzedPrompt) -> String {
    let mut code = String::new();
    let struct_name = &analyzed.prompt_file.prompt_name;

    code.push_str(&format!("impl {} {{\n", struct_name));

    // Generate XML renderer
    code.push_str("    pub fn render_xml(&self) -> String {\n");
    code.push_str("        let mut output = String::new();\n");
    code.push_str(&generate_render_body(analyzed, RenderFormat::Xml));
    code.push_str("        output.trim_end().to_string()\n");
    code.push_str("    }\n\n");

    // Generate Markdown renderer
    code.push_str("    pub fn render_markdown(&self) -> String {\n");
    code.push_str("        let mut output = String::new();\n");
    code.push_str(&generate_render_body(analyzed, RenderFormat::Markdown));
    code.push_str("        output.trim_end().to_string()\n");
    code.push_str("    }\n\n");

    // Generate Plain renderer
    code.push_str("    pub fn render_plain(&self) -> String {\n");
    code.push_str("        let mut output = String::new();\n");
    code.push_str(&generate_render_body(analyzed, RenderFormat::Plain));
    code.push_str("        output.trim_end().to_string()\n");
    code.push_str("    }\n");

    code.push_str("}\n\n");

    code
}

#[derive(Debug, Clone, Copy)]
enum RenderFormat {
    Xml,
    Markdown,
    Plain,
}

fn generate_render_body(analyzed: &AnalyzedPrompt, format: RenderFormat) -> String {
    let mut code = String::new();

    for section in &analyzed.prompt_file.sections {
        let section_name = &section.name;

        // Check if section is optional
        if section.is_optional() {
            // Generate conditional check for optional sections
            // A section is rendered if any of its parameters has a value
            code.push_str("        if ");

            let mut conditions = Vec::new();
            for item in &section.content.items {
                if let ContentItem::Parameter(param) = item {
                    let field_name = param_name_to_field_name(&param.name);
                    if let Some(param_info) = analyzed.parameters.get(&param.name) {
                        match param_info.rust_type {
                            RustType::OptionString => {
                                conditions.push(format!("self.{}.is_some()", field_name));
                            }
                            RustType::VecString => {
                                conditions.push(format!("!self.{}.is_empty()", field_name));
                            }
                            _ => {}
                        }
                    }
                }
            }

            if !conditions.is_empty() {
                code.push_str(&conditions.join(" || "));
                code.push_str(" {\n");
            } else {
                // If no parameters, always render
                code.push_str("true {\n");
            }
        }

        // Section header
        match format {
            RenderFormat::Xml => {
                code.push_str(&format!(
                    "        output.push_str(\"<{}>\");\n",
                    section_name
                ));
            }
            RenderFormat::Markdown => {
                let title = snake_case_to_title_case(section_name);
                code.push_str(&format!("        output.push_str(\"# {}\\n\\n\");\n", title));
            }
            RenderFormat::Plain => {
                let upper = snake_case_to_upper(section_name);
                code.push_str(&format!("        output.push_str(\"{}:\\n\");\n", upper));
            }
        }

        // Section content
        code.push_str(&generate_section_content(
            &section.content.items,
            analyzed,
            format,
        ));

        // Section footer
        match format {
            RenderFormat::Xml => {
                code.push_str(&format!(
                    "        output.push_str(\"</{}>\\n\\n\");\n",
                    section_name
                ));
            }
            RenderFormat::Markdown | RenderFormat::Plain => {
                // Content already ends with \n (ensured above), add one more for blank line separator
                code.push_str("        output.push_str(\"\\n\");\n");
            }
        }

        if section.is_optional() {
            code.push_str("        }\n");
        }
    }

    code
}

fn generate_section_content(
    items: &[ContentItem],
    analyzed: &AnalyzedPrompt,
    format: RenderFormat,
) -> String {
    let mut code = String::new();

    for item in items {
        match item {
            ContentItem::Text(text) => {
                let escaped = escape_rust_string(text);
                code.push_str(&format!("        output.push_str(\"{}\");\n", escaped));
            }
            ContentItem::Parameter(param) => {
                code.push_str(&generate_parameter_substitution(param, analyzed, format));
            }
        }
    }

    // Ensure content ends with exactly one newline for consistent section spacing
    match format {
        RenderFormat::Markdown | RenderFormat::Plain => {
            code.push_str("        if !output.ends_with('\\n') {\n");
            code.push_str("            output.push_str(\"\\n\");\n");
            code.push_str("        }\n");
        }
        _ => {}
    }

    code
}

fn generate_parameter_substitution(
    param: &Parameter,
    analyzed: &AnalyzedPrompt,
    format: RenderFormat,
) -> String {
    let field_name = param_name_to_field_name(&param.name);
    let param_info = analyzed
        .parameters
        .get(&param.name)
        .expect("Parameter should exist in analyzed parameters");

    match &param.kind {
        ParameterKind::Plain => generate_plain_parameter(&field_name, param_info, format),

        ParameterKind::WithDefault(default) => {
            let escaped_default = escape_rust_string(default);
            let mut code = String::new();

            match param_info.rust_type {
                RustType::OptionString => {
                    code.push_str(&format!(
                        "        output.push_str(self.{}.as_deref().unwrap_or(\"{}\"));\n",
                        field_name, escaped_default
                    ));
                }
                _ => {
                    code.push_str(&format!("        output.push_str(&self.{});\n", field_name));
                }
            }

            code
        }

        ParameterKind::WithRenderType {
            render_type,
            attributes,
        } => generate_rendered_parameter(&field_name, param_info, render_type, attributes, format, analyzed),
    }
}

fn generate_plain_parameter(
    field_name: &str,
    param_info: &crate::semantic::ParameterInfo,
    #[allow(unused_variables)] _format: RenderFormat,
) -> String {
    let mut code = String::new();

    match param_info.rust_type {
        RustType::String => {
            code.push_str(&format!("        output.push_str(&self.{});\n", field_name));
        }
        RustType::OptionString => {
            code.push_str(&format!(
                "        if let Some(ref value) = self.{} {{\n",
                field_name
            ));
            code.push_str("            output.push_str(value);\n");
            code.push_str("        }\n");
        }
        RustType::VecString => {
            // This shouldn't happen for plain parameters
            code.push_str(&format!("        // Unexpected VecString for {}\n", field_name));
        }
    }

    code
}

fn generate_rendered_parameter(
    field_name: &str,
    _param_info: &crate::semantic::ParameterInfo,
    render_type: &RenderType,
    attributes: &[crate::parser::RenderAttribute],
    format: RenderFormat,
    analyzed: &AnalyzedPrompt,
) -> String {
    let mut code = String::new();

    match render_type {
        RenderType::CodeBlock => {
            // Extract language attribute
            let language = attributes
                .iter()
                .find(|attr| attr.name == "language")
                .map(|attr| match &attr.value {
                    RenderAttrValue::Literal(s) => format!("\"{}\"", escape_rust_string(s)),
                    RenderAttrValue::ParamRef { name, default } => {
                        let param_field = param_name_to_field_name(name);
                        // Check the actual parameter type from analyzed
                        let param_type = analyzed.parameters.get(name)
                            .map(|p| &p.rust_type);

                        if let Some(def) = default {
                            format!(
                                "self.{}.as_deref().unwrap_or(\"{}\")",
                                param_field,
                                escape_rust_string(def)
                            )
                        } else if matches!(param_type, Some(RustType::OptionString)) {
                            // Parameter is optional, need to unwrap
                            if let Some(p) = analyzed.parameters.get(name) {
                                if let Some(default_val) = &p.default_value {
                                    format!(
                                        "self.{}.as_deref().unwrap_or(\"{}\")",
                                        param_field,
                                        escape_rust_string(default_val)
                                    )
                                } else {
                                    format!("self.{}.as_deref().unwrap_or(\"\")", param_field)
                                }
                            } else {
                                format!("&self.{}", param_field)
                            }
                        } else {
                            format!("&self.{}", param_field)
                        }
                    }
                });

            match format {
                RenderFormat::Xml | RenderFormat::Markdown => {
                    if let Some(lang_expr) = language {
                        code.push_str(&format!("        output.push_str(\"```\");\n"));
                        code.push_str(&format!("        output.push_str({});\n", lang_expr));
                        code.push_str(&format!("        output.push_str(\"\\n\");\n"));
                    } else {
                        code.push_str(&format!("        output.push_str(\"```\\n\");\n"));
                    }
                    code.push_str(&format!("        output.push_str(&self.{});\n", field_name));
                    code.push_str(&format!("        output.push_str(\"\\n```\\n\");\n"));
                }
                RenderFormat::Plain => {
                    code.push_str(&format!("        output.push_str(&self.{});\n", field_name));
                    code.push_str(&format!("        output.push_str(\"\\n\");\n"));
                }
            }
        }

        RenderType::List => {
            match format {
                RenderFormat::Xml | RenderFormat::Markdown | RenderFormat::Plain => {
                    code.push_str(&format!(
                        "        for item in &self.{} {{\n",
                        field_name
                    ));
                    code.push_str("            output.push_str(\"- \");\n");
                    code.push_str("            output.push_str(item);\n");
                    code.push_str("            output.push_str(\"\\n\");\n");
                    code.push_str("        }\n");
                }
            }
        }

        RenderType::Json => {
            match format {
                RenderFormat::Xml | RenderFormat::Markdown => {
                    code.push_str(&format!("        output.push_str(\"```json\\n\");\n"));
                    code.push_str(&format!("        output.push_str(&self.{});\n", field_name));
                    code.push_str(&format!("        output.push_str(\"\\n```\\n\");\n"));
                }
                RenderFormat::Plain => {
                    code.push_str(&format!("        output.push_str(&self.{});\n", field_name));
                    code.push_str(&format!("        output.push_str(\"\\n\");\n"));
                }
            }
        }

        RenderType::Xml => {
            match format {
                RenderFormat::Xml | RenderFormat::Markdown => {
                    code.push_str(&format!("        output.push_str(\"```xml\\n\");\n"));
                    code.push_str(&format!("        output.push_str(&self.{});\n", field_name));
                    code.push_str(&format!("        output.push_str(\"\\n```\\n\");\n"));
                }
                RenderFormat::Plain => {
                    code.push_str(&format!("        output.push_str(&self.{});\n", field_name));
                }
            }
        }

        RenderType::Plain => {
            code.push_str(&format!("        output.push_str(&self.{});\n", field_name));
        }
    }

    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Span;
    use crate::parser::*;
    use crate::semantic::{AnalyzedPrompt, ParameterInfo};
    use std::collections::HashMap;

    #[test]
    fn test_generate_render_methods() {
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

        let section = Section::new(
            "greeting".to_string(),
            vec![],
            SectionContent::new(vec![
                ContentItem::Text("Hello, ".to_string()),
                ContentItem::Parameter(Parameter::new(
                    "name".to_string(),
                    ParameterKind::Plain,
                    Span::zero(),
                )),
                ContentItem::Text("!".to_string()),
            ]),
            Span::zero(),
        );

        let prompt_file = PromptFile::new(
            "Test".to_string(),
            None,
            vec![section],
            Span::zero(),
        );

        let analyzed = AnalyzedPrompt::new(prompt_file, params);
        let code = generate_render_methods(&analyzed);

        assert!(code.contains("pub fn render_xml(&self) -> String"));
        assert!(code.contains("pub fn render_markdown(&self) -> String"));
        assert!(code.contains("pub fn render_plain(&self) -> String"));
        assert!(code.contains("output.push_str(\"<greeting>\")"));  // No newline after opening tag
        assert!(code.contains("output.push_str(\"# Greeting\\n\\n\")"));
        assert!(code.contains("output.push_str(\"GREETING:\\n\")"));
        assert!(code.contains("output.trim_end().to_string()"));  // Trimming trailing whitespace
    }
}
