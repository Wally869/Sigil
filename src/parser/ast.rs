use crate::error::Span;

/// Root node representing a complete Sigil prompt file
#[derive(Debug, Clone, PartialEq)]
pub struct PromptFile {
    pub prompt_name: String,
    pub description: Option<String>,
    pub sections: Vec<Section>,
    pub span: Span,
}

impl PromptFile {
    pub fn new(prompt_name: String, description: Option<String>, sections: Vec<Section>, span: Span) -> Self {
        Self {
            prompt_name,
            description,
            sections,
            span,
        }
    }
}

/// A section in the prompt
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub name: String,
    pub attributes: Vec<SectionAttribute>,
    pub content: SectionContent,
    pub span: Span,
}

impl Section {
    pub fn new(name: String, attributes: Vec<SectionAttribute>, content: SectionContent, span: Span) -> Self {
        Self {
            name,
            attributes,
            content,
            span,
        }
    }

    pub fn is_optional(&self) -> bool {
        self.attributes.iter().any(|attr| matches!(attr, SectionAttribute::Optional))
    }
}

/// Attributes that can be applied to a section
#[derive(Debug, Clone, PartialEq)]
pub enum SectionAttribute {
    Optional,
}

/// Content of a section, composed of text and parameters
#[derive(Debug, Clone, PartialEq)]
pub struct SectionContent {
    pub items: Vec<ContentItem>,
}

impl SectionContent {
    pub fn new(items: Vec<ContentItem>) -> Self {
        Self { items }
    }

    pub fn empty() -> Self {
        Self { items: Vec::new() }
    }
}

/// An item in section content - either text or a parameter
#[derive(Debug, Clone, PartialEq)]
pub enum ContentItem {
    Text(String),
    Parameter(Parameter),
}

/// A parameter placeholder in the content
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub kind: ParameterKind,
    pub span: Span,
}

impl Parameter {
    pub fn new(name: String, kind: ParameterKind, span: Span) -> Self {
        Self { name, kind, span }
    }
}

/// Different kinds of parameters
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterKind {
    /// Plain parameter: {name}
    Plain,

    /// Parameter with default value: {name="default"}
    WithDefault(String),

    /// Parameter with render type: {name:render_type[...]}
    WithRenderType {
        render_type: RenderType,
        attributes: Vec<RenderAttribute>,
    },
}

/// Types of special rendering for parameters
#[derive(Debug, Clone, PartialEq)]
pub enum RenderType {
    CodeBlock,
    List,
    Json,
    Xml,
    Plain,
}

impl RenderType {
    pub fn as_str(&self) -> &str {
        match self {
            RenderType::CodeBlock => "code_block",
            RenderType::List => "list",
            RenderType::Json => "json",
            RenderType::Xml => "xml",
            RenderType::Plain => "plain",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "code_block" => Some(RenderType::CodeBlock),
            "list" => Some(RenderType::List),
            "json" => Some(RenderType::Json),
            "xml" => Some(RenderType::Xml),
            "plain" => Some(RenderType::Plain),
            _ => None,
        }
    }
}

/// Attribute for render types (e.g., language="rust")
#[derive(Debug, Clone, PartialEq)]
pub struct RenderAttribute {
    pub name: String,
    pub value: RenderAttrValue,
    pub span: Span,
}

impl RenderAttribute {
    pub fn new(name: String, value: RenderAttrValue, span: Span) -> Self {
        Self { name, value, span }
    }
}

/// Value of a render attribute
#[derive(Debug, Clone, PartialEq)]
pub enum RenderAttrValue {
    /// A string literal: "value"
    Literal(String),

    /// A parameter reference: {param}
    ParamRef {
        name: String,
        default: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SourceLocation;

    #[test]
    fn test_render_type_conversions() {
        assert_eq!(RenderType::CodeBlock.as_str(), "code_block");
        assert_eq!(RenderType::from_str("list"), Some(RenderType::List));
        assert_eq!(RenderType::from_str("invalid"), None);
    }

    #[test]
    fn test_section_is_optional() {
        let span = Span::zero();

        let optional_section = Section::new(
            "test".to_string(),
            vec![SectionAttribute::Optional],
            SectionContent::empty(),
            span,
        );
        assert!(optional_section.is_optional());

        let required_section = Section::new(
            "test".to_string(),
            vec![],
            SectionContent::empty(),
            span,
        );
        assert!(!required_section.is_optional());
    }

    #[test]
    fn test_parameter_kinds() {
        let span = Span::zero();

        let plain = Parameter::new("test".to_string(), ParameterKind::Plain, span);
        assert!(matches!(plain.kind, ParameterKind::Plain));

        let with_default = Parameter::new(
            "test".to_string(),
            ParameterKind::WithDefault("default".to_string()),
            span,
        );
        match with_default.kind {
            ParameterKind::WithDefault(ref d) => assert_eq!(d, "default"),
            _ => panic!("Expected WithDefault"),
        }
    }

    #[test]
    fn test_render_attr_value() {
        let literal = RenderAttrValue::Literal("test".to_string());
        assert!(matches!(literal, RenderAttrValue::Literal(_)));

        let param_ref = RenderAttrValue::ParamRef {
            name: "lang".to_string(),
            default: Some("rust".to_string()),
        };
        match param_ref {
            RenderAttrValue::ParamRef { name, default } => {
                assert_eq!(name, "lang");
                assert_eq!(default, Some("rust".to_string()));
            }
            _ => panic!("Expected ParamRef"),
        }
    }

    #[test]
    fn test_content_item() {
        let text = ContentItem::Text("hello".to_string());
        assert!(matches!(text, ContentItem::Text(_)));

        let param = ContentItem::Parameter(Parameter::new(
            "name".to_string(),
            ParameterKind::Plain,
            Span::zero(),
        ));
        assert!(matches!(param, ContentItem::Parameter(_)));
    }

    #[test]
    fn test_prompt_file_structure() {
        let span = Span::zero();
        let file = PromptFile::new(
            "TestPrompt".to_string(),
            Some("A test prompt".to_string()),
            vec![],
            span,
        );

        assert_eq!(file.prompt_name, "TestPrompt");
        assert_eq!(file.description, Some("A test prompt".to_string()));
        assert!(file.sections.is_empty());
    }
}
