use crate::error::Span;

/// Token types in the Sigil language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Prompt,         // @prompt
    Description,    // @description
    End,            // @end
    Optional,       // optional

    // Render types
    CodeBlock,      // code_block
    List,           // list
    Json,           // json
    Xml,            // xml
    Plain,          // plain

    // Identifiers and literals
    Identifier(String),
    StringLiteral(String),
    SectionName(String),  // @identifier (section header)
    Text(String),         // Arbitrary text (for section content)

    // Symbols
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Equals,         // =
    Colon,          // :
    Comma,          // ,

    // Whitespace and structural
    Newline,        // \n or \r\n

    // End of file
    Eof,
}

impl TokenKind {
    /// Check if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Prompt
                | TokenKind::Description
                | TokenKind::End
                | TokenKind::Optional
        )
    }

    /// Check if this token is a render type
    pub fn is_render_type(&self) -> bool {
        matches!(
            self,
            TokenKind::CodeBlock
                | TokenKind::List
                | TokenKind::Json
                | TokenKind::Xml
                | TokenKind::Plain
        )
    }

    /// Get the string representation of a token kind
    pub fn as_str(&self) -> &str {
        match self {
            TokenKind::Prompt => "@prompt",
            TokenKind::Description => "@description",
            TokenKind::End => "@end",
            TokenKind::Optional => "optional",
            TokenKind::CodeBlock => "code_block",
            TokenKind::List => "list",
            TokenKind::Json => "json",
            TokenKind::Xml => "xml",
            TokenKind::Plain => "plain",
            TokenKind::Identifier(_) => "identifier",
            TokenKind::StringLiteral(_) => "string literal",
            TokenKind::SectionName(_) => "section name",
            TokenKind::Text(_) => "text",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::LeftBracket => "[",
            TokenKind::RightBracket => "]",
            TokenKind::Equals => "=",
            TokenKind::Colon => ":",
            TokenKind::Comma => ",",
            TokenKind::Newline => "newline",
            TokenKind::Eof => "end of file",
        }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Identifier(s) => write!(f, "identifier '{}'", s),
            TokenKind::StringLiteral(s) => write!(f, "string \"{}\"", s),
            TokenKind::SectionName(s) => write!(f, "section @{}", s),
            TokenKind::Text(s) => write!(f, "text '{}'", s),
            _ => write!(f, "{}", self.as_str()),
        }
    }
}

/// A token with its kind and location in the source
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn eof(span: Span) -> Self {
        Self {
            kind: TokenKind::Eof,
            span,
        }
    }
}

/// Parse a keyword or identifier
pub fn parse_keyword_or_identifier(word: &str) -> TokenKind {
    match word {
        "prompt" => TokenKind::Prompt,
        "description" => TokenKind::Description,
        "end" => TokenKind::End,
        "optional" => TokenKind::Optional,
        "code_block" => TokenKind::CodeBlock,
        "list" => TokenKind::List,
        "json" => TokenKind::Json,
        "xml" => TokenKind::Xml,
        "plain" => TokenKind::Plain,
        _ => TokenKind::Identifier(word.to_string()),
    }
}

/// Check if a character can start an identifier
pub fn is_identifier_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

/// Check if a character can continue an identifier
pub fn is_identifier_continue(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SourceLocation;

    #[test]
    fn test_token_kind_is_keyword() {
        assert!(TokenKind::Prompt.is_keyword());
        assert!(TokenKind::End.is_keyword());
        assert!(!TokenKind::CodeBlock.is_keyword());
    }

    #[test]
    fn test_token_kind_is_render_type() {
        assert!(TokenKind::CodeBlock.is_render_type());
        assert!(TokenKind::List.is_render_type());
        assert!(!TokenKind::Prompt.is_render_type());
    }

    #[test]
    fn test_parse_keywords() {
        assert_eq!(parse_keyword_or_identifier("prompt"), TokenKind::Prompt);
        assert_eq!(parse_keyword_or_identifier("end"), TokenKind::End);
        assert_eq!(parse_keyword_or_identifier("code_block"), TokenKind::CodeBlock);
    }

    #[test]
    fn test_parse_identifier() {
        match parse_keyword_or_identifier("my_var") {
            TokenKind::Identifier(s) => assert_eq!(s, "my_var"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_identifier_validation() {
        assert!(is_identifier_start('a'));
        assert!(is_identifier_start('_'));
        assert!(!is_identifier_start('1'));

        assert!(is_identifier_continue('a'));
        assert!(is_identifier_continue('1'));
        assert!(is_identifier_continue('_'));
        assert!(!is_identifier_continue('@'));
    }

    #[test]
    fn test_token_display() {
        let span = Span::new(SourceLocation::new(1, 1), SourceLocation::new(1, 5));

        let token = Token::new(TokenKind::Prompt, span);
        assert_eq!(format!("{}", token.kind), "@prompt");

        let token = Token::new(TokenKind::Identifier("test".to_string()), span);
        assert_eq!(format!("{}", token.kind), "identifier 'test'");
    }
}
