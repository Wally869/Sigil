mod cursor;
mod token;

pub use token::{Token, TokenKind};

use crate::error::{Result, SigilError, SourceLocation, Span};
use cursor::Cursor;
use token::{is_identifier_continue, is_identifier_start, parse_keyword_or_identifier};

/// Main lexer for Sigil language
pub struct Lexer<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer from source text
    pub fn new(source: &'a str) -> Self {
        Self {
            cursor: Cursor::new(source),
        }
    }

    /// Tokenize the entire source
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    /// Get the next token
    fn next_token(&mut self) -> Result<Token> {
        let start_loc = self.cursor.location();

        match self.cursor.peek() {
            // Handle spaces as Text tokens (spaces are significant in section content)
            Some(' ') | Some('\t') => {
                let ws_char = self.cursor.peek().unwrap();
                self.cursor.advance();
                let end_loc = self.cursor.location();
                Ok(Token::new(
                    TokenKind::Text(ws_char.to_string()),
                    Span::new(start_loc, end_loc),
                ))
            }
            None => Ok(Token::eof(Span::from_single(start_loc))),

            Some('\n') => {
                self.cursor.advance();
                let end_loc = self.cursor.location();
                Ok(Token::new(
                    TokenKind::Newline,
                    Span::new(start_loc, end_loc),
                ))
            }

            Some('\r') => {
                self.cursor.advance();
                // Handle \r\n
                if self.cursor.peek() == Some('\n') {
                    self.cursor.advance();
                }
                let end_loc = self.cursor.location();
                Ok(Token::new(
                    TokenKind::Newline,
                    Span::new(start_loc, end_loc),
                ))
            }

            Some('/') if self.cursor.peek_ahead(0) == Some('/') => {
                self.cursor.skip_comment();
                // After skipping comment, get next token
                self.next_token()
            }

            Some('@') => {
                self.cursor.advance(); // consume '@'
                self.lex_directive_or_section()
            }

            Some('{') => {
                self.cursor.advance();
                let end_loc = self.cursor.location();
                Ok(Token::new(
                    TokenKind::LeftBrace,
                    Span::new(start_loc, end_loc),
                ))
            }

            Some('}') => {
                self.cursor.advance();
                let end_loc = self.cursor.location();
                Ok(Token::new(
                    TokenKind::RightBrace,
                    Span::new(start_loc, end_loc),
                ))
            }

            Some('[') => {
                self.cursor.advance();
                let end_loc = self.cursor.location();
                Ok(Token::new(
                    TokenKind::LeftBracket,
                    Span::new(start_loc, end_loc),
                ))
            }

            Some(']') => {
                self.cursor.advance();
                let end_loc = self.cursor.location();
                Ok(Token::new(
                    TokenKind::RightBracket,
                    Span::new(start_loc, end_loc),
                ))
            }

            Some('=') => {
                self.cursor.advance();
                let end_loc = self.cursor.location();
                Ok(Token::new(TokenKind::Equals, Span::new(start_loc, end_loc)))
            }

            Some(':') => {
                self.cursor.advance();
                let end_loc = self.cursor.location();
                Ok(Token::new(TokenKind::Colon, Span::new(start_loc, end_loc)))
            }

            Some(',') => {
                self.cursor.advance();
                let end_loc = self.cursor.location();
                Ok(Token::new(TokenKind::Comma, Span::new(start_loc, end_loc)))
            }

            Some('"') => self.lex_string_literal(),

            Some(ch) if is_identifier_start(ch) => self.lex_identifier(),

            Some(ch) => {
                // Any other character is valid in section content
                self.cursor.advance();
                let end_loc = self.cursor.location();
                Ok(Token::new(
                    TokenKind::Text(ch.to_string()),
                    Span::new(start_loc, end_loc),
                ))
            }
        }
    }

    /// Lex a directive (@prompt, @description, @end) or section header (@section_name)
    fn lex_directive_or_section(&mut self) -> Result<Token> {
        let start_loc = SourceLocation::new(
            self.cursor.line(),
            self.cursor.column().saturating_sub(1),
        );

        // Read the identifier after @
        if self.cursor.peek().map(is_identifier_start).unwrap_or(false) {
            let identifier = self.cursor.take_while(is_identifier_continue);
            let end_loc = self.cursor.location();
            let span = Span::new(start_loc, end_loc);

            let kind = match identifier.as_str() {
                "prompt" => TokenKind::Prompt,
                "description" => TokenKind::Description,
                "end" => TokenKind::End,
                _ => TokenKind::SectionName(identifier),
            };

            Ok(Token::new(kind, span))
        } else {
            Err(SigilError::InvalidIdentifier {
                name: "@".to_string(),
                location: start_loc,
            })
        }
    }

    /// Lex an identifier or keyword
    fn lex_identifier(&mut self) -> Result<Token> {
        let start_loc = self.cursor.location();
        let identifier = self.cursor.take_while(is_identifier_continue);
        let end_loc = self.cursor.location();

        let kind = parse_keyword_or_identifier(&identifier);
        Ok(Token::new(kind, Span::new(start_loc, end_loc)))
    }

    /// Lex a string literal
    fn lex_string_literal(&mut self) -> Result<Token> {
        let start_loc = self.cursor.location();

        self.cursor.advance(); // consume opening "

        let mut string_value = String::new();

        loop {
            match self.cursor.peek() {
                None | Some('\n') | Some('\r') => {
                    return Err(SigilError::UnclosedStringLiteral {
                        location: start_loc,
                    });
                }

                Some('"') => {
                    self.cursor.advance(); // consume closing "
                    break;
                }

                Some('\\') => {
                    self.cursor.advance(); // consume backslash
                    match self.cursor.peek() {
                        Some('"') => {
                            string_value.push('"');
                            self.cursor.advance();
                        }
                        Some('\\') => {
                            string_value.push('\\');
                            self.cursor.advance();
                        }
                        Some('n') => {
                            string_value.push('\n');
                            self.cursor.advance();
                        }
                        Some('r') => {
                            string_value.push('\r');
                            self.cursor.advance();
                        }
                        Some('t') => {
                            string_value.push('\t');
                            self.cursor.advance();
                        }
                        Some(ch) => {
                            return Err(SigilError::InvalidEscapeSequence {
                                sequence: format!("\\{}", ch),
                                location: self.cursor.location(),
                            });
                        }
                        None => {
                            return Err(SigilError::UnclosedStringLiteral {
                                location: start_loc,
                            });
                        }
                    }
                }

                Some(ch) => {
                    string_value.push(ch);
                    self.cursor.advance();
                }
            }
        }

        let end_loc = self.cursor.location();
        Ok(Token::new(
            TokenKind::StringLiteral(string_value),
            Span::new(start_loc, end_loc),
        ))
    }
}

/// Convenience function to lex source code
pub fn lex(source: &str) -> Result<Vec<Token>> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_keywords() {
        let source = "@prompt\n@description\n@end";  // Use newlines instead of spaces
        let tokens = lex(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Prompt);
        assert_eq!(tokens[1].kind, TokenKind::Newline);
        assert_eq!(tokens[2].kind, TokenKind::Description);
        assert_eq!(tokens[3].kind, TokenKind::Newline);
        assert_eq!(tokens[4].kind, TokenKind::End);
    }

    #[test]
    fn test_lex_section_name() {
        let source = "@system";
        let tokens = lex(source).unwrap();

        match &tokens[0].kind {
            TokenKind::SectionName(name) => assert_eq!(name, "system"),
            _ => panic!("Expected section name"),
        }
    }

    #[test]
    fn test_lex_identifiers() {
        let source = "optional\ncode_block\nlist";  // Use newlines instead of spaces
        let tokens = lex(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Optional);
        assert_eq!(tokens[1].kind, TokenKind::Newline);
        assert_eq!(tokens[2].kind, TokenKind::CodeBlock);
        assert_eq!(tokens[3].kind, TokenKind::Newline);
        assert_eq!(tokens[4].kind, TokenKind::List);
    }

    #[test]
    fn test_lex_string_literal() {
        let source = r#""hello world""#;
        let tokens = lex(source).unwrap();

        match &tokens[0].kind {
            TokenKind::StringLiteral(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_lex_string_with_escapes() {
        let source = r#""line1\nline2\t\"quoted\"""#;
        let tokens = lex(source).unwrap();

        match &tokens[0].kind {
            TokenKind::StringLiteral(s) => {
                assert_eq!(s, "line1\nline2\t\"quoted\"");
            }
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_lex_symbols() {
        let source = "{}[]=:,";  // No spaces - spaces are now Text tokens
        let tokens = lex(source).unwrap();

        assert_eq!(tokens[0].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[1].kind, TokenKind::RightBrace);
        assert_eq!(tokens[2].kind, TokenKind::LeftBracket);
        assert_eq!(tokens[3].kind, TokenKind::RightBracket);
        assert_eq!(tokens[4].kind, TokenKind::Equals);
        assert_eq!(tokens[5].kind, TokenKind::Colon);
        assert_eq!(tokens[6].kind, TokenKind::Comma);
    }

    #[test]
    fn test_lex_newlines() {
        let source = "a\nb\r\nc";
        let tokens = lex(source).unwrap();

        match &tokens[0].kind {
            TokenKind::Identifier(s) => assert_eq!(s, "a"),
            _ => panic!("Expected identifier"),
        }
        assert_eq!(tokens[1].kind, TokenKind::Newline);
        match &tokens[2].kind {
            TokenKind::Identifier(s) => assert_eq!(s, "b"),
            _ => panic!("Expected identifier"),
        }
        assert_eq!(tokens[3].kind, TokenKind::Newline);
    }

    #[test]
    fn test_lex_comments() {
        let source = "a// comment\nb";  // No space before //
        let tokens = lex(source).unwrap();

        match &tokens[0].kind {
            TokenKind::Identifier(s) => assert_eq!(s, "a"),
            _ => panic!("Expected identifier"),
        }
        assert_eq!(tokens[1].kind, TokenKind::Newline);
        match &tokens[2].kind {
            TokenKind::Identifier(s) => assert_eq!(s, "b"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_lex_unclosed_string() {
        let source = r#""unclosed"#;
        let result = lex(source);

        assert!(result.is_err());
        match result.unwrap_err() {
            SigilError::UnclosedStringLiteral { .. } => {}
            _ => panic!("Expected unclosed string literal error"),
        }
    }

    #[test]
    fn test_lex_invalid_escape() {
        let source = r#""bad\xescape""#;
        let result = lex(source);

        assert!(result.is_err());
        match result.unwrap_err() {
            SigilError::InvalidEscapeSequence { .. } => {}
            _ => panic!("Expected invalid escape sequence error"),
        }
    }

    #[test]
    fn test_lex_complex_example() {
        let source = r#"
@prompt CodeReview
@description "A code review prompt"

@system
@end
"#;
        let tokens = lex(source).unwrap();

        // Should successfully tokenize without errors
        assert!(!tokens.is_empty());
        assert!(matches!(tokens.last().unwrap().kind, TokenKind::Eof));
    }
}
