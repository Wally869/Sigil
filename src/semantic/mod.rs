pub mod type_checker;

pub use type_checker::{ParameterInfo, RustType, TypeChecker};

use crate::error::Result;
use crate::parser::PromptFile;
use std::collections::HashMap;

/// Analyzed prompt file with type information
#[derive(Debug, Clone)]
pub struct AnalyzedPrompt {
    pub prompt_file: PromptFile,
    pub parameters: HashMap<String, ParameterInfo>,
}

impl AnalyzedPrompt {
    pub fn new(prompt_file: PromptFile, parameters: HashMap<String, ParameterInfo>) -> Self {
        Self {
            prompt_file,
            parameters,
        }
    }
}

/// Perform semantic analysis on a parsed prompt file
pub fn analyze(prompt_file: &PromptFile) -> Result<AnalyzedPrompt> {
    let mut type_checker = TypeChecker::new();

    // Analyze sections and parameters
    type_checker.analyze_sections(&prompt_file.sections)?;

    // Extract parameters from render attributes
    type_checker.extract_attribute_parameters(&prompt_file.sections)?;

    // Get analyzed parameter information
    let parameters = type_checker.get_parameters().clone();

    Ok(AnalyzedPrompt::new(prompt_file.clone(), parameters))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    use crate::parser;

    fn analyze_source(source: &str) -> Result<AnalyzedPrompt> {
        let tokens = lexer::lex(source)?;
        let ast = parser::parse(tokens, "test.sigil")?;
        analyze(&ast)
    }

    #[test]
    fn test_analyze_simple_prompt() {
        let source = r#"
@prompt Test

@section
Hello {name}
@end
"#;
        let analyzed = analyze_source(source).unwrap();

        assert_eq!(analyzed.parameters.len(), 1);
        assert!(analyzed.parameters.contains_key("name"));

        let param = &analyzed.parameters["name"];
        assert_eq!(param.rust_type, RustType::String);
        assert!(param.is_required);
    }

    #[test]
    fn test_analyze_optional_parameter() {
        let source = r#"
@prompt Test

@section[optional]
Hello {name}
@end
"#;
        let analyzed = analyze_source(source).unwrap();

        let param = &analyzed.parameters["name"];
        assert_eq!(param.rust_type, RustType::OptionString);
        assert!(!param.is_required);
    }

    #[test]
    fn test_analyze_parameter_with_default() {
        let source = r#"
@prompt Test

@section
Hello {name="World"}
@end
"#;
        let analyzed = analyze_source(source).unwrap();

        let param = &analyzed.parameters["name"];
        assert_eq!(param.rust_type, RustType::OptionString);
        assert!(!param.is_required);
        assert_eq!(param.default_value, Some("World".to_string()));
    }

    #[test]
    fn test_analyze_list_parameter() {
        let source = r#"
@prompt Test

@section
Items: {items:list}
@end
"#;
        let analyzed = analyze_source(source).unwrap();

        let param = &analyzed.parameters["items"];
        assert_eq!(param.rust_type, RustType::VecString);
    }

    #[test]
    fn test_analyze_type_conflict() {
        let source = r#"
@prompt Test

@section1
{data}
@end

@section2
{data:list}
@end
"#;
        let result = analyze_source(source);
        assert!(result.is_err());
    }
}
