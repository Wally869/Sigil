/// Convert snake_case to Title Case
///
/// Example: "code_review" -> "Code Review"
pub fn snake_case_to_title_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert snake_case to UPPER_CASE
///
/// Example: "code_review" -> "CODE_REVIEW"
pub fn snake_case_to_upper(s: &str) -> String {
    s.to_uppercase()
}

/// Check if a string is in PascalCase
pub fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // First character must be uppercase
    if !first.is_uppercase() {
        return false;
    }

    // Rest can be alphanumeric or underscore
    for ch in chars {
        if !ch.is_alphanumeric() && ch != '_' {
            return false;
        }
    }

    true
}

/// Check if a string is in snake_case
pub fn is_snake_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // First character must be lowercase or underscore
    if !first.is_lowercase() && first != '_' {
        return false;
    }

    // Rest must be lowercase, digits, or underscore
    for ch in chars {
        if !ch.is_lowercase() && !ch.is_numeric() && ch != '_' {
            return false;
        }
    }

    true
}

/// Escape a string for use in Rust code
pub fn escape_rust_string(s: &str) -> String {
    let mut escaped = String::new();

    for ch in s.chars() {
        match ch {
            '"' => escaped.push_str(r#"\""#),
            '\\' => escaped.push_str(r"\\"),
            '\n' => escaped.push_str(r"\n"),
            '\r' => escaped.push_str(r"\r"),
            '\t' => escaped.push_str(r"\t"),
            _ => escaped.push(ch),
        }
    }

    escaped
}

/// Check if an identifier is a Rust keyword
pub fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

/// Escape a Rust identifier if it's a keyword by adding r# prefix
pub fn escape_rust_identifier(s: &str) -> String {
    if is_rust_keyword(s) {
        format!("r#{}", s)
    } else {
        s.to_string()
    }
}

/// Convert a parameter name to a valid Rust field name
pub fn param_name_to_field_name(s: &str) -> String {
    escape_rust_identifier(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case_to_title_case() {
        assert_eq!(snake_case_to_title_case("hello_world"), "Hello World");
        assert_eq!(snake_case_to_title_case("code_review"), "Code Review");
        assert_eq!(snake_case_to_title_case("system"), "System");
        assert_eq!(
            snake_case_to_title_case("my_long_section_name"),
            "My Long Section Name"
        );
    }

    #[test]
    fn test_snake_case_to_upper() {
        assert_eq!(snake_case_to_upper("hello_world"), "HELLO_WORLD");
        assert_eq!(snake_case_to_upper("system"), "SYSTEM");
    }

    #[test]
    fn test_is_pascal_case() {
        assert!(is_pascal_case("HelloWorld"));
        assert!(is_pascal_case("CodeReview"));
        assert!(is_pascal_case("A"));
        assert!(!is_pascal_case("helloWorld"));
        assert!(!is_pascal_case("hello_world"));
        assert!(!is_pascal_case(""));
    }

    #[test]
    fn test_is_snake_case() {
        assert!(is_snake_case("hello_world"));
        assert!(is_snake_case("code_review"));
        assert!(is_snake_case("a"));
        assert!(is_snake_case("_private"));
        assert!(!is_snake_case("HelloWorld"));
        assert!(!is_snake_case("helloWorld"));
        assert!(!is_snake_case(""));
    }

    #[test]
    fn test_escape_rust_string() {
        assert_eq!(escape_rust_string("hello"), "hello");
        assert_eq!(escape_rust_string("hello\"world"), r#"hello\"world"#);
        assert_eq!(escape_rust_string("line1\nline2"), r"line1\nline2");
        assert_eq!(escape_rust_string("tab\there"), r"tab\there");
    }

    #[test]
    fn test_is_rust_keyword() {
        assert!(is_rust_keyword("fn"));
        assert!(is_rust_keyword("struct"));
        assert!(is_rust_keyword("return"));
        assert!(is_rust_keyword("async"));
        assert!(!is_rust_keyword("hello"));
        assert!(!is_rust_keyword("my_var"));
    }

    #[test]
    fn test_escape_rust_identifier() {
        assert_eq!(escape_rust_identifier("hello"), "hello");
        assert_eq!(escape_rust_identifier("fn"), "r#fn");
        assert_eq!(escape_rust_identifier("type"), "r#type");
    }

    #[test]
    fn test_param_name_to_field_name() {
        assert_eq!(param_name_to_field_name("my_field"), "my_field");
        assert_eq!(param_name_to_field_name("type"), "r#type");
    }
}
