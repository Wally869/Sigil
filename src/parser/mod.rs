pub mod ast;

pub use ast::*;

use crate::error::{Result, SigilError, Span};
use crate::lexer::{Token, TokenKind};

/// Parser for Sigil language
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    filename: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, filename: String) -> Self {
        Self {
            tokens,
            current: 0,
            filename,
        }
    }

    /// Parse the tokens into an AST
    pub fn parse(&mut self) -> Result<PromptFile> {
        self.skip_newlines();

        // Parse @prompt directive (required, must be first)
        let (prompt_name, prompt_span) = self.parse_prompt_directive()?;

        self.skip_newlines();

        // Parse @description directive (optional)
        let description = self.parse_description_directive()?;

        self.skip_newlines();

        // Parse sections
        let mut sections = Vec::new();
        while !self.is_at_end() && !matches!(self.peek().kind, TokenKind::Eof) {
            let section = self.parse_section()?;
            sections.push(section);
            self.skip_newlines();
        }

        let end_span = self.previous().span;
        let full_span = Span::new(prompt_span.start, end_span.end);

        Ok(PromptFile::new(prompt_name, description, sections, full_span))
    }

    /// Parse @prompt directive
    fn parse_prompt_directive(&mut self) -> Result<(String, Span)> {
        let token = self.advance();

        if !matches!(token.kind, TokenKind::Prompt) {
            return Err(SigilError::MissingPromptDirective {
                location: token.span.start,
            });
        }

        let prompt_span = token.span;

        self.skip_whitespace_tokens();

        // Expect identifier (prompt name)
        let name_token = self.advance();
        let prompt_name = match &name_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                return Err(SigilError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: name_token.kind.to_string(),
                    span: name_token.span,
                });
            }
        };

        // Expect newline
        self.expect_newline()?;

        Ok((prompt_name, prompt_span))
    }

    /// Parse @description directive (optional)
    fn parse_description_directive(&mut self) -> Result<Option<String>> {
        if !matches!(self.peek().kind, TokenKind::Description) {
            return Ok(None);
        }

        self.advance(); // consume @description

        self.skip_whitespace_tokens();

        // Expect string literal
        let desc_token = self.advance();
        let description = match &desc_token.kind {
            TokenKind::StringLiteral(s) => s.clone(),
            _ => {
                return Err(SigilError::UnexpectedToken {
                    expected: "string literal".to_string(),
                    found: desc_token.kind.to_string(),
                    span: desc_token.span,
                });
            }
        };

        self.expect_newline()?;

        Ok(Some(description))
    }

    /// Parse a section
    fn parse_section(&mut self) -> Result<Section> {
        // Parse section header (@section_name[optional])
        let start_span = self.peek().span;

        let section_name = match &self.advance().kind {
            TokenKind::SectionName(name) => name.clone(),
            TokenKind::End => {
                return Err(SigilError::UnexpectedToken {
                    expected: "section name".to_string(),
                    found: "@end".to_string(),
                    span: start_span,
                });
            }
            other => {
                return Err(SigilError::UnexpectedToken {
                    expected: "section name".to_string(),
                    found: other.to_string(),
                    span: start_span,
                });
            }
        };

        // Parse optional attributes [optional]
        let attributes = self.parse_section_attributes()?;

        self.expect_newline()?;

        // Parse section content until @end
        let content = self.parse_section_content()?;

        // Expect @end
        let end_token = self.advance();
        if !matches!(end_token.kind, TokenKind::End) {
            return Err(SigilError::MissingEndTerminator {
                section_name: section_name.clone(),
                start: start_span,
            });
        }

        let end_span = end_token.span;

        self.expect_newline()?;
        let full_span = Span::new(start_span.start, end_span.end);

        Ok(Section::new(section_name, attributes, content, full_span))
    }

    /// Parse section attributes [optional]
    fn parse_section_attributes(&mut self) -> Result<Vec<SectionAttribute>> {
        if !matches!(self.peek().kind, TokenKind::LeftBracket) {
            return Ok(Vec::new());
        }

        self.advance(); // consume [

        let mut attributes = Vec::new();

        loop {
            if matches!(self.peek().kind, TokenKind::RightBracket) {
                self.advance(); // consume ]
                break;
            }

            let attr_token = self.advance();
            match attr_token.kind {
                TokenKind::Optional => attributes.push(SectionAttribute::Optional),
                _ => {
                    return Err(SigilError::UnexpectedToken {
                        expected: "optional or ]".to_string(),
                        found: attr_token.kind.to_string(),
                        span: attr_token.span,
                    });
                }
            }

            // Check for comma (optional, allows trailing comma)
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
            }
        }

        Ok(attributes)
    }

    /// Parse section content (text and parameters until @end)
    fn parse_section_content(&mut self) -> Result<SectionContent> {
        let mut items = Vec::new();
        let mut current_text = String::new();

        loop {
            let token = self.peek();

            match &token.kind {
                TokenKind::End | TokenKind::Eof => {
                    // Flush any pending text
                    if !current_text.is_empty() {
                        items.push(ContentItem::Text(current_text.clone()));
                        current_text.clear();
                    }
                    break;
                }

                TokenKind::Newline => {
                    current_text.push('\n');
                    self.advance();
                }

                TokenKind::LeftBrace => {
                    // Flush text before parameter
                    if !current_text.is_empty() {
                        items.push(ContentItem::Text(current_text.clone()));
                        current_text.clear();
                    }

                    // Parse parameter
                    let param = self.parse_parameter()?;
                    items.push(ContentItem::Parameter(param));
                }

                TokenKind::Identifier(s) => {
                    current_text.push_str(s);
                    self.advance();
                }

                TokenKind::StringLiteral(s) => {
                    current_text.push('"');
                    current_text.push_str(s);
                    current_text.push('"');
                    self.advance();
                }

                TokenKind::SectionName(s) => {
                    current_text.push('@');
                    current_text.push_str(s);
                    self.advance();
                }

                TokenKind::Text(s) => {
                    current_text.push_str(s);
                    self.advance();
                }

                // Add other tokens as text
                _ => {
                    current_text.push_str(token.kind.as_str());
                    self.advance();
                }
            }
        }

        // Trim leading and trailing blank lines from content
        let content = Self::trim_content(items);

        Ok(SectionContent::new(content))
    }

    /// Trim leading and trailing blank lines from content
    fn trim_content(items: Vec<ContentItem>) -> Vec<ContentItem> {
        if items.is_empty() {
            return items;
        }

        let mut trimmed = items;

        // Trim leading newlines
        while !trimmed.is_empty() {
            if let Some(ContentItem::Text(text)) = trimmed.first() {
                let trimmed_text = text.trim_start_matches('\n');
                if trimmed_text.is_empty() {
                    trimmed.remove(0);
                } else if trimmed_text != text {
                    trimmed[0] = ContentItem::Text(trimmed_text.to_string());
                    break;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Trim trailing newlines
        while !trimmed.is_empty() {
            if let Some(ContentItem::Text(text)) = trimmed.last() {
                let trimmed_text = text.trim_end_matches('\n');
                if trimmed_text.is_empty() {
                    trimmed.pop();
                } else if trimmed_text != text {
                    let last_idx = trimmed.len() - 1;
                    trimmed[last_idx] = ContentItem::Text(trimmed_text.to_string());
                    break;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        trimmed
    }

    /// Parse a parameter {name}, {name="default"}, {name:type[...]}
    fn parse_parameter(&mut self) -> Result<Parameter> {
        let start_span = self.peek().span;

        self.expect(TokenKind::LeftBrace)?;

        self.skip_whitespace_tokens();

        let name_token = self.advance();
        let param_name = match &name_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                return Err(SigilError::MalformedParameter {
                    message: format!("expected identifier, found {}", name_token.kind),
                    span: name_token.span,
                });
            }
        };

        self.skip_whitespace_tokens();

        let kind = match self.peek().kind {
            TokenKind::RightBrace => ParameterKind::Plain,

            TokenKind::Equals => {
                self.advance(); // consume =
                self.skip_whitespace_tokens();
                let default_token = self.advance();
                match &default_token.kind {
                    TokenKind::StringLiteral(value) => ParameterKind::WithDefault(value.clone()),
                    _ => {
                        return Err(SigilError::MalformedParameter {
                            message: "expected string literal after =".to_string(),
                            span: default_token.span,
                        });
                    }
                }
            }

            TokenKind::Colon => {
                self.advance(); // consume :
                self.skip_whitespace_tokens();
                let (render_type, attributes) = self.parse_render_type_and_attributes()?;
                ParameterKind::WithRenderType {
                    render_type,
                    attributes,
                }
            }

            _ => {
                return Err(SigilError::MalformedParameter {
                    message: format!("unexpected token {}", self.peek().kind),
                    span: self.peek().span,
                });
            }
        };

        self.expect(TokenKind::RightBrace)?;

        let end_span = self.previous().span;
        let full_span = Span::new(start_span.start, end_span.end);

        Ok(Parameter::new(param_name, kind, full_span))
    }

    /// Parse render type and its attributes
    fn parse_render_type_and_attributes(&mut self) -> Result<(RenderType, Vec<RenderAttribute>)> {
        let type_token = self.advance();
        let render_type = match &type_token.kind {
            TokenKind::CodeBlock => RenderType::CodeBlock,
            TokenKind::List => RenderType::List,
            TokenKind::Json => RenderType::Json,
            TokenKind::Xml => RenderType::Xml,
            TokenKind::Plain => RenderType::Plain,
            TokenKind::Identifier(s) => {
                if let Some(rt) = RenderType::from_str(s) {
                    rt
                } else {
                    return Err(SigilError::UnknownRenderType {
                        render_type: s.clone(),
                        location: type_token.span.start,
                    });
                }
            }
            _ => {
                return Err(SigilError::UnknownRenderType {
                    render_type: type_token.kind.to_string(),
                    location: type_token.span.start,
                });
            }
        };

        let attributes = if matches!(self.peek().kind, TokenKind::LeftBracket) {
            self.parse_render_attributes()?
        } else {
            Vec::new()
        };

        Ok((render_type, attributes))
    }

    /// Parse render attributes [key=value, ...]
    fn parse_render_attributes(&mut self) -> Result<Vec<RenderAttribute>> {
        self.expect(TokenKind::LeftBracket)?;

        let mut attributes = Vec::new();

        loop {
            if matches!(self.peek().kind, TokenKind::RightBracket) {
                self.advance(); // consume ]
                break;
            }

            let start_span = self.peek().span;

            // Parse attribute name
            let name_token = self.advance();
            let attr_name = match &name_token.kind {
                TokenKind::Identifier(name) => name.clone(),
                _ => {
                    return Err(SigilError::UnexpectedToken {
                        expected: "identifier".to_string(),
                        found: name_token.kind.to_string(),
                        span: name_token.span,
                    });
                }
            };

            self.expect(TokenKind::Equals)?;

            // Parse attribute value (string literal or {param})
            let value = if matches!(self.peek().kind, TokenKind::LeftBrace) {
                self.parse_render_attr_param_ref()?
            } else {
                let value_token = self.advance();
                match &value_token.kind {
                    TokenKind::StringLiteral(s) => RenderAttrValue::Literal(s.clone()),
                    _ => {
                        return Err(SigilError::UnexpectedToken {
                            expected: "string literal or {param}".to_string(),
                            found: value_token.kind.to_string(),
                            span: value_token.span,
                        });
                    }
                }
            };

            let end_span = self.previous().span;
            let attr_span = Span::new(start_span.start, end_span.end);

            attributes.push(RenderAttribute::new(attr_name, value, attr_span));

            // Check for comma
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
            }
        }

        Ok(attributes)
    }

    /// Parse a parameter reference in render attribute: {param} or {param="default"}
    fn parse_render_attr_param_ref(&mut self) -> Result<RenderAttrValue> {
        self.expect(TokenKind::LeftBrace)?;

        let name_token = self.advance();
        let param_name = match &name_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                return Err(SigilError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: name_token.kind.to_string(),
                    span: name_token.span,
                });
            }
        };

        let default = if matches!(self.peek().kind, TokenKind::Equals) {
            self.advance(); // consume =
            let default_token = self.advance();
            match &default_token.kind {
                TokenKind::StringLiteral(value) => Some(value.clone()),
                _ => {
                    return Err(SigilError::UnexpectedToken {
                        expected: "string literal".to_string(),
                        found: default_token.kind.to_string(),
                        span: default_token.span,
                    });
                }
            }
        } else {
            None
        };

        self.expect(TokenKind::RightBrace)?;

        Ok(RenderAttrValue::ParamRef {
            name: param_name,
            default,
        })
    }

    // Helper methods

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof) || self.current >= self.tokens.len()
    }

    fn expect(&mut self, kind: TokenKind) -> Result<()> {
        if std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(&kind) {
            self.advance();
            Ok(())
        } else {
            Err(SigilError::UnexpectedToken {
                expected: kind.as_str().to_string(),
                found: self.peek().kind.to_string(),
                span: self.peek().span,
            })
        }
    }

    fn expect_newline(&mut self) -> Result<()> {
        if matches!(self.peek().kind, TokenKind::Newline | TokenKind::Eof) {
            if !matches!(self.peek().kind, TokenKind::Eof) {
                self.advance();
            }
            Ok(())
        } else {
            Err(SigilError::UnexpectedToken {
                expected: "newline".to_string(),
                found: self.peek().kind.to_string(),
                span: self.peek().span,
            })
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }
    }

    fn skip_whitespace_tokens(&mut self) {
        loop {
            match &self.peek().kind {
                TokenKind::Text(s) if s == " " || s == "\t" => {
                    self.advance();
                }
                _ => break,
            }
        }
    }
}

/// Parse tokens into an AST
pub fn parse(tokens: Vec<Token>, filename: &str) -> Result<PromptFile> {
    let mut parser = Parser::new(tokens, filename.to_string());
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    fn parse_source(source: &str) -> Result<PromptFile> {
        let tokens = lexer::lex(source)?;
        parse(tokens, "test.sigil")
    }

    #[test]
    fn test_parse_minimal_prompt() {
        let source = r#"
@prompt Simple

@section
Hello, {name}!
@end
"#;
        let ast = parse_source(source).unwrap();

        assert_eq!(ast.prompt_name, "Simple");
        assert_eq!(ast.description, None);
        assert_eq!(ast.sections.len(), 1);
        assert_eq!(ast.sections[0].name, "section");
    }

    #[test]
    fn test_parse_with_description() {
        let source = r#"
@prompt Test
@description "A test prompt"

@section
Content
@end
"#;
        let ast = parse_source(source).unwrap();

        assert_eq!(ast.prompt_name, "Test");
        assert_eq!(ast.description, Some("A test prompt".to_string()));
    }

    #[test]
    fn test_parse_optional_section() {
        let source = r#"
@prompt Test

@section[optional]
Optional content
@end
"#;
        let ast = parse_source(source).unwrap();

        assert!(ast.sections[0].is_optional());
    }

    #[test]
    fn test_parse_parameters() {
        let source = r#"
@prompt Test

@section
Plain: {name}
Default: {lang="rust"}
Render: {code:code_block[language="python"]}
@end
"#;
        let ast = parse_source(source).unwrap();

        let items = &ast.sections[0].content.items;

        // Find parameters in the items
        let params: Vec<&Parameter> = items
            .iter()
            .filter_map(|item| match item {
                ContentItem::Parameter(p) => Some(p),
                _ => None,
            })
            .collect();

        assert_eq!(params.len(), 3);
        assert!(matches!(params[0].kind, ParameterKind::Plain));
        assert!(matches!(params[1].kind, ParameterKind::WithDefault(_)));
    }

    #[test]
    fn test_parse_missing_prompt() {
        let source = r#"
@section
Content
@end
"#;
        let result = parse_source(source);
        assert!(result.is_err());
    }
}
