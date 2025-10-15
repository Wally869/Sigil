use std::fmt;

/// Represents a location in the source file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn zero() -> Self {
        Self { line: 0, column: 0 }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Represents a span of source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl Span {
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        Self { start, end }
    }

    pub fn zero() -> Self {
        Self {
            start: SourceLocation::zero(),
            end: SourceLocation::zero(),
        }
    }

    pub fn from_single(loc: SourceLocation) -> Self {
        Self {
            start: loc,
            end: loc,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start.line == self.end.line {
            write!(f, "{}:{}-{}", self.start.line, self.start.column, self.end.column)
        } else {
            write!(f, "{} to {}", self.start, self.end)
        }
    }
}

/// Main error type for Sigil compiler
#[derive(Debug, Clone)]
pub enum SigilError {
    // Lexer errors
    UnexpectedCharacter { ch: char, location: SourceLocation },
    UnclosedStringLiteral { location: SourceLocation },
    InvalidEscapeSequence { sequence: String, location: SourceLocation },

    // Parser errors
    UnexpectedToken { expected: String, found: String, span: Span },
    MissingPromptDirective { location: SourceLocation },
    DuplicatePromptDirective { first: Span, second: Span },
    MissingEndTerminator { section_name: String, start: Span },
    InvalidIdentifier { name: String, location: SourceLocation },
    UnknownRenderType { render_type: String, location: SourceLocation },
    MalformedParameter { message: String, span: Span },

    // Semantic errors
    TypeConflict {
        param_name: String,
        first_type: String,
        first_span: Span,
        second_type: String,
        second_span: Span,
    },
    MultipleDefaults {
        param_name: String,
        first_span: Span,
        second_span: Span
    },
    DuplicateSection {
        section_name: String,
        first_span: Span,
        second_span: Span
    },

    // Generic errors
    IoError { message: String },
    Other { message: String },
}

impl fmt::Display for SigilError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Lexer errors
            SigilError::UnexpectedCharacter { ch, location } => {
                write!(f, "error: unexpected character '{}' at {}", ch, location)
            }
            SigilError::UnclosedStringLiteral { location } => {
                write!(f, "error: unclosed string literal at {}", location)
            }
            SigilError::InvalidEscapeSequence { sequence, location } => {
                write!(f, "error: invalid escape sequence '{}' at {}", sequence, location)
            }

            // Parser errors
            SigilError::UnexpectedToken { expected, found, span } => {
                write!(f, "error: expected {}, found {} at {}", expected, found, span)
            }
            SigilError::MissingPromptDirective { location } => {
                write!(f, "error: missing @prompt directive at {}", location)
            }
            SigilError::DuplicatePromptDirective { first, second } => {
                write!(f, "error: multiple @prompt directives found\n  first at {}\n  second at {}", first, second)
            }
            SigilError::MissingEndTerminator { section_name, start } => {
                write!(f, "error: section '{}' missing @end terminator (started at {})", section_name, start)
            }
            SigilError::InvalidIdentifier { name, location } => {
                write!(f, "error: invalid identifier '{}' at {}", name, location)
            }
            SigilError::UnknownRenderType { render_type, location } => {
                write!(f, "error: unknown render type '{}' at {}\n  = help: valid types are 'code_block', 'list', 'json', 'xml', 'plain'", render_type, location)
            }
            SigilError::MalformedParameter { message, span } => {
                write!(f, "error: malformed parameter at {}: {}", span, message)
            }

            // Semantic errors
            SigilError::TypeConflict { param_name, first_type, first_span, second_type, second_span } => {
                write!(
                    f,
                    "error: parameter '{}' used with conflicting types\n  {} used as {} at {}\n  {} used as {} at {}",
                    param_name, param_name, first_type, first_span, param_name, second_type, second_span
                )
            }
            SigilError::MultipleDefaults { param_name, first_span, second_span } => {
                write!(
                    f,
                    "error: parameter '{}' has multiple default values\n  first default at {}\n  second default at {}",
                    param_name, first_span, second_span
                )
            }
            SigilError::DuplicateSection { section_name, first_span, second_span } => {
                write!(
                    f,
                    "error: section '{}' defined multiple times\n  first at {}\n  second at {}",
                    section_name, first_span, second_span
                )
            }

            // Generic errors
            SigilError::IoError { message } => {
                write!(f, "error: I/O error: {}", message)
            }
            SigilError::Other { message } => {
                write!(f, "error: {}", message)
            }
        }
    }
}

impl std::error::Error for SigilError {}

impl From<std::io::Error> for SigilError {
    fn from(err: std::io::Error) -> Self {
        SigilError::IoError {
            message: err.to_string(),
        }
    }
}

/// Type alias for Results using SigilError
pub type Result<T> = std::result::Result<T, SigilError>;

/// Diagnostic reporter for enhanced error output
pub struct DiagnosticReporter {
    source: String,
    filename: String,
}

impl DiagnosticReporter {
    pub fn new(source: String, filename: String) -> Self {
        Self { source, filename }
    }

    /// Generate a detailed error report with source context
    pub fn report(&self, error: &SigilError) -> String {
        let mut output = String::new();

        // Add the error message
        output.push_str(&format!("{}\n", error));

        // Add source context if we have location information
        match error {
            SigilError::UnexpectedCharacter { location, .. }
            | SigilError::UnclosedStringLiteral { location }
            | SigilError::InvalidEscapeSequence { location, .. }
            | SigilError::MissingPromptDirective { location }
            | SigilError::InvalidIdentifier { location, .. }
            | SigilError::UnknownRenderType { location, .. } => {
                self.add_context(&mut output, location, location);
            }
            SigilError::UnexpectedToken { span, .. }
            | SigilError::MalformedParameter { span, .. }
            | SigilError::MissingEndTerminator { start: span, .. } => {
                self.add_context(&mut output, &span.start, &span.end);
            }
            SigilError::DuplicatePromptDirective { first, second }
            | SigilError::MultipleDefaults { first_span: first, second_span: second, .. }
            | SigilError::DuplicateSection { first_span: first, second_span: second, .. } => {
                self.add_context(&mut output, &first.start, &first.end);
                output.push_str("  ...\n");
                self.add_context(&mut output, &second.start, &second.end);
            }
            SigilError::TypeConflict { first_span, second_span, .. } => {
                self.add_context(&mut output, &first_span.start, &first_span.end);
                output.push_str("  ...\n");
                self.add_context(&mut output, &second_span.start, &second_span.end);
            }
            _ => {}
        }

        output
    }

    fn add_context(&self, output: &mut String, start: &SourceLocation, _end: &SourceLocation) {
        let lines: Vec<&str> = self.source.lines().collect();

        if start.line == 0 || start.line > lines.len() {
            return;
        }

        let line = lines[start.line - 1];
        output.push_str(&format!("  --> {}:{}:{}\n", self.filename, start.line, start.column));
        output.push_str(&format!("   |\n"));
        output.push_str(&format!("{:3} | {}\n", start.line, line));
        output.push_str(&format!("   | {}", " ".repeat(start.column.saturating_sub(1))));
        output.push_str("^\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation::new(10, 5);
        assert_eq!(format!("{}", loc), "10:5");
    }

    #[test]
    fn test_span_display_same_line() {
        let span = Span::new(
            SourceLocation::new(5, 10),
            SourceLocation::new(5, 20),
        );
        assert_eq!(format!("{}", span), "5:10-20");
    }

    #[test]
    fn test_span_display_different_lines() {
        let span = Span::new(
            SourceLocation::new(5, 10),
            SourceLocation::new(7, 5),
        );
        assert_eq!(format!("{}", span), "5:10 to 7:5");
    }

    #[test]
    fn test_error_display() {
        let err = SigilError::UnexpectedCharacter {
            ch: '$',
            location: SourceLocation::new(1, 5),
        };
        assert!(format!("{}", err).contains("unexpected character"));
        assert!(format!("{}", err).contains("'$'"));
    }

    #[test]
    fn test_diagnostic_reporter() {
        let source = "line 1\nline 2\nline 3".to_string();
        let reporter = DiagnosticReporter::new(source, "test.sigil".to_string());

        let error = SigilError::UnexpectedCharacter {
            ch: '@',
            location: SourceLocation::new(2, 3),
        };

        let report = reporter.report(&error);
        assert!(report.contains("test.sigil"));
        assert!(report.contains("line 2"));
    }
}
