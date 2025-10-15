use crate::error::SourceLocation;

/// A cursor over the source text for lexical analysis
///
/// Tracks the current position in the source and provides
/// methods for peeking and advancing through characters.
pub struct Cursor<'a> {
    source: &'a str,
    chars: std::str::Chars<'a>,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Cursor<'a> {
    /// Create a new cursor from source text
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Peek at the current character without consuming it
    pub fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    /// Peek at the character n positions ahead (0 = next char after current)
    pub fn peek_ahead(&self, n: usize) -> Option<char> {
        let mut chars = self.chars.clone();
        chars.next()?; // Skip current
        for _ in 0..n {
            chars.next()?;
        }
        chars.next()
    }

    /// Advance the cursor by one character
    pub fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        self.position += ch.len_utf8();

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        Some(ch)
    }

    /// Check if we're at the end of the source
    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// Get the current source location
    pub fn location(&self) -> SourceLocation {
        SourceLocation::new(self.line, self.column)
    }

    /// Get the current position in bytes
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the current line number
    pub fn line(&self) -> usize {
        self.line
    }

    /// Get the current column number
    pub fn column(&self) -> usize {
        self.column
    }

    /// Get the remaining source text
    pub fn remaining(&self) -> &'a str {
        self.chars.as_str()
    }

    /// Skip whitespace (spaces and tabs) but not newlines
    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip a single-line comment (// until end of line)
    pub fn skip_comment(&mut self) {
        // Assume we're at '//'
        self.advance(); // first '/'
        self.advance(); // second '/'

        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.advance();
        }
    }

    /// Check if the next characters match a given string
    pub fn starts_with(&self, s: &str) -> bool {
        self.chars.as_str().starts_with(s)
    }

    /// Consume characters while a predicate holds
    pub fn take_while<F>(&mut self, mut predicate: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if predicate(ch) {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        result
    }

    /// Get a slice of the source from start position to current position
    pub fn slice_from(&self, start: usize) -> &'a str {
        &self.source[start..self.position]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_basic() {
        let source = "hello";
        let mut cursor = Cursor::new(source);

        assert_eq!(cursor.peek(), Some('h'));
        assert_eq!(cursor.advance(), Some('h'));
        assert_eq!(cursor.peek(), Some('e'));
        assert_eq!(cursor.position(), 1);
    }

    #[test]
    fn test_cursor_location() {
        let source = "line1\nline2";
        let mut cursor = Cursor::new(source);

        assert_eq!(cursor.line(), 1);
        assert_eq!(cursor.column(), 1);

        for _ in 0..5 {
            cursor.advance();
        }
        assert_eq!(cursor.column(), 6);

        cursor.advance(); // newline
        assert_eq!(cursor.line(), 2);
        assert_eq!(cursor.column(), 1);
    }

    #[test]
    fn test_cursor_peek_ahead() {
        let source = "hello";
        let cursor = Cursor::new(source);

        assert_eq!(cursor.peek_ahead(0), Some('e'));
        assert_eq!(cursor.peek_ahead(1), Some('l'));
        assert_eq!(cursor.peek_ahead(4), None);
    }

    #[test]
    fn test_cursor_skip_whitespace() {
        let source = "   \t  hello";
        let mut cursor = Cursor::new(source);

        cursor.skip_whitespace();
        assert_eq!(cursor.peek(), Some('h'));
    }

    #[test]
    fn test_cursor_skip_comment() {
        let source = "// comment\ncode";
        let mut cursor = Cursor::new(source);

        cursor.skip_comment();
        assert_eq!(cursor.peek(), Some('\n'));
    }

    #[test]
    fn test_cursor_starts_with() {
        let source = "hello world";
        let cursor = Cursor::new(source);

        assert!(cursor.starts_with("hello"));
        assert!(!cursor.starts_with("world"));
    }

    #[test]
    fn test_cursor_take_while() {
        let source = "abc123def";
        let mut cursor = Cursor::new(source);

        let result = cursor.take_while(|ch| ch.is_alphabetic());
        assert_eq!(result, "abc");
        assert_eq!(cursor.peek(), Some('1'));
    }

    #[test]
    fn test_cursor_utf8() {
        let source = "hello 世界";
        let mut cursor = Cursor::new(source);

        for _ in 0..6 {
            cursor.advance();
        }
        assert_eq!(cursor.peek(), Some('世'));
        cursor.advance();
        assert_eq!(cursor.peek(), Some('界'));
    }

    #[test]
    fn test_cursor_eof() {
        let source = "a";
        let mut cursor = Cursor::new(source);

        assert!(!cursor.is_eof());
        cursor.advance();
        assert!(cursor.is_eof());
    }
}
