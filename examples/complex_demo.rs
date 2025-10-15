// Comprehensive demo showing all Sigil features with a complex real-world example

use std::fs;

// Include the generated code from AI code reviewer
include!("../target/generated_ai_reviewer.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Comprehensive Sigil Demo ===\n");
    println!("Features demonstrated:");
    println!("  • Multiple sections (6 sections)");
    println!("  • Optional sections with conditional rendering");
    println!("  • List parameters with add_item methods");
    println!("  • Code block rendering with language attribute");
    println!("  • Parameter references in attributes");
    println!("  • Default values for multiple types");
    println!("  • Complex nested parameter usage");
    println!("  • XML/Markdown/Plain output formats\n");

    // Create output directory
    fs::create_dir_all("output")?;

    println!("=== Example 1: Minimal Required Parameters ===\n");

    let minimal = AICodeReviewer::builder()
        .add_expertise("code quality")
        .file_path("src/main.rs")
        .source_code("fn main() {\n    println!(\"Hello, world!\");\n}")
        .build()?;

    println!("Built with only required parameters");
    println!("(Uses defaults: role=Senior Software Engineer, language=Rust, etc.)\n");

    let xml1 = minimal.render_xml();
    fs::write("output/review_minimal.xml", &xml1)?;
    println!("XML output (first 400 chars):");
    println!("{}", &xml1[..400.min(xml1.len())]);
    println!("...\n");

    println!("=== Example 2: Full Featured Review ===\n");

    let full_review = AICodeReviewer::builder()
        .role("Principal Engineer")
        .language("python")
        .years("15")
        .add_expertise("security auditing")
        .add_expertise("performance optimization")
        .add_expertise("API design")
        .add_expertise("concurrent programming")
        .project_name("FastAPI Microservice")
        .repo_url("https://github.com/example/api")
        .branch("feature/auth-refactor")
        .additional_context("This is a critical authentication module used by 1M+ users")
        .file_path("app/auth/jwt_handler.py")
        .source_code(r#"import jwt
from datetime import datetime, timedelta

class JWTHandler:
    def __init__(self, secret):
        self.secret = secret

    def create_token(self, user_id):
        payload = {
            'user_id': user_id,
            'exp': datetime.utcnow() + timedelta(hours=24)
        }
        return jwt.encode(payload, self.secret, algorithm='HS256')

    def verify_token(self, token):
        try:
            return jwt.decode(token, self.secret, algorithms=['HS256'])
        except:
            return None
"#)
        .add_focus_areas("JWT security best practices")
        .add_focus_areas("Error handling and logging")
        .add_focus_areas("Token expiration strategy")
        .add_focus_areas("Secret management")
        .output_format("json")
        .severity_levels("critical,high,medium")
        .include_suggestions("true")
        .build()?;

    println!("Built comprehensive review with:");
    println!("  - Custom role and language");
    println!("  - 4 expertise areas");
    println!("  - Project context");
    println!("  - Python code block");
    println!("  - 4 focus areas");
    println!("  - Custom output format\n");

    // Render to all formats
    let xml = full_review.render_xml();
    let markdown = full_review.render_markdown();
    let plain = full_review.render_plain();

    fs::write("output/review_full.xml", &xml)?;
    fs::write("output/review_full.md", &markdown)?;
    fs::write("output/review_full.txt", &plain)?;

    println!("--- XML Format (for Claude) ---");
    println!("{}\n", xml);

    println!("--- Markdown Format (for GPT-4) ---");
    println!("{}\n", &markdown[..800.min(markdown.len())]);
    println!("...\n");

    println!("=== Example 3: Partial Optional Sections ===\n");

    let partial = AICodeReviewer::builder()
        .language("rust")
        .add_expertise("systems programming")
        .add_expertise("memory safety")
        .file_path("src/parser.rs")
        .source_code("pub fn parse(input: &str) -> Result<AST> { todo!() }")
        .add_focus_areas("error handling")
        // Note: context section omitted, will not render
        .build()?;

    println!("Built with:");
    println!("  - review_focus section included (has focus_areas)");
    println!("  - context section omitted (no context params provided)\n");

    let xml3 = partial.render_xml();
    fs::write("output/review_partial.xml", &xml3)?;
    println!("XML output shows conditional section rendering:\n");
    println!("{}\n", xml3);

    println!("=== Summary ===\n");
    println!("Generated files:");
    println!("  • output/review_minimal.xml  - Minimal required params");
    println!("  • output/review_full.xml     - Full featured XML");
    println!("  • output/review_full.md      - Full featured Markdown");
    println!("  • output/review_full.txt     - Full featured Plain text");
    println!("  • output/review_partial.xml  - Partial optional sections\n");

    println!("Key Sigil Features Demonstrated:");
    println!("  ✓ Type-safe builders prevent missing required params");
    println!("  ✓ add_expertise/add_focus_areas for list params");
    println!("  ✓ Default values (role, language, branch, etc.)");
    println!("  ✓ Optional sections only render when params provided");
    println!("  ✓ Code blocks with language from parameter");
    println!("  ✓ Multiple output formats from one template");
    println!("  ✓ Parameter references in render attributes");
    println!("  ✓ Complex nested parameter substitution\n");

    println!("This template generates ~12KB of type-safe Rust code!");
    println!("Zero runtime overhead - all validation at compile time.\n");

    Ok(())
}
