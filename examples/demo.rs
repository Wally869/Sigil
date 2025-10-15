use sigil;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Sigil Compiler Demo ===\n");

    // Example 1: Compile greeting.sigil
    println!("1. Compiling examples/greeting.sigil...");
    let greeting_code = sigil::compile_sigil_file("examples/greeting.sigil")?;
    println!("   ✓ Generated {} bytes of Rust code\n", greeting_code.len());

    // Example 2: Compile code_review.sigil
    println!("2. Compiling examples/code_review.sigil...");
    let review_code = sigil::compile_sigil_file("examples/code_review.sigil")?;
    println!("   ✓ Generated {} bytes of Rust code\n", review_code.len());

    // Show a snippet of generated code
    println!("3. Sample of generated code (first 500 chars):");
    println!("   {}\n   ...\n", &greeting_code[..greeting_code.len().min(500)]);

    println!("✓ All examples compiled successfully!");
    println!("\nThe generated code includes:");
    println!("  • Struct definitions with typed fields");
    println!("  • Builder pattern for construction");
    println!("  • render_xml(), render_markdown(), render_plain() methods");

    Ok(())
}
