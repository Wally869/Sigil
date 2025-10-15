// Sigil - A Domain-Specific Language for Type-Safe LLM Prompt Templates
//
// This library provides a compile-time DSL for creating type-safe prompt templates
// with multiple output formats (XML, Markdown, Plain Text).

pub mod error;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod codegen;
pub mod util;

use std::fs;
use std::path::Path;

pub use error::{SigilError, Result, SourceLocation, Span};

/// Main entry point for compiling a Sigil file
///
/// # Arguments
/// * `path` - Path to the .sigil file
///
/// # Returns
/// * `Ok(String)` - Generated Rust code
/// * `Err(SigilError)` - Compilation error
///
/// # Example
/// ```ignore
/// let generated_code = sigil::compile_sigil_file("prompts/example.sigil")?;
/// ```
pub fn compile_sigil_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    let source = fs::read_to_string(path)?;
    let filename = path.to_string_lossy().to_string();

    compile_sigil(&source, &filename)
}

/// Compiles Sigil source code to Rust code
///
/// # Arguments
/// * `source` - The Sigil source code
/// * `filename` - Filename for error reporting
///
/// # Returns
/// * `Ok(String)` - Generated Rust code
/// * `Err(SigilError)` - Compilation error
pub fn compile_sigil(source: &str, filename: &str) -> Result<String> {
    // Step 1: Lexical analysis
    let tokens = lexer::lex(source)?;

    // Step 2: Parse into AST
    let ast = parser::parse(tokens, filename)?;

    // Step 3: Semantic analysis and type checking
    let analyzed = semantic::analyze(&ast)?;

    // Step 4: Generate Rust code
    let generated_code = codegen::generate(&analyzed)?;

    Ok(generated_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_prompt() {
        let source = r#"
@prompt Greeting
@description "A simple greeting"

@message
Hello, {name}!
@end
"#;

        let result = compile_sigil(source, "test.sigil");
        assert!(result.is_ok(), "Should compile successfully");

        let code = result.unwrap();
        assert!(code.contains("struct Greeting"), "Should generate Greeting struct");
        assert!(code.contains("pub fn builder()"), "Should generate builder method");
    }
}
