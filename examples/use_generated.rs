// Step 2: Use the generated code to actually render prompts
// This includes the generated code from step 1

use std::fs;

// Include the generated code
include!("../target/generated_prompt.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    fs::create_dir_all("output")?;
    println!("=== Using Generated Sigil Code ===\n");

    // Build a prompt with the type-safe builder
    println!("Building prompt with:");
    println!("  name: \"Alice\"");
    println!("  place: \"Wonderland\"\n");

    let prompt = Greeting::builder()
        .name("Alice")
        .place("Wonderland")
        .build()?;

    println!("=== RENDERED OUTPUT ===\n");

    // Render to XML (for Claude)
    println!("--- XML Format (for Claude API) ---");
    let xml = prompt.render_xml();
    println!("{}", xml);
    fs::write("output/greeting_alice.xml", &xml)?;

    // Render to Markdown (for GPT-4)
    println!("--- Markdown Format (for GPT-4 API) ---");
    let markdown = prompt.render_markdown();
    println!("{}", markdown);
    fs::write("output/greeting_alice.md", &markdown)?;

    // Render to Plain (for debugging)
    println!("--- Plain Text Format ---");
    let plain = prompt.render_plain();
    println!("{}", plain);
    fs::write("output/greeting_alice.txt", &plain)?;

    println!("✓ Saved to output/ directory");

    // More examples
    println!("=== Additional Examples ===\n");

    // Using default value
    let default_prompt = Greeting::builder()
        .name("Bob")
        .build()?;

    println!("With default place='Earth':");
    print!("{}", default_prompt.render_xml());

    // Batch rendering
    println!("Batch rendering multiple prompts:");
    for (name, place) in &[("Charlie", "Mars"), ("Diana", "Venus"), ("Eve", "Jupiter")] {
        let p = Greeting::builder()
            .name(*name)
            .place(*place)
            .build()?;
        let output = p.render_plain();
        println!("  {}", output.lines().nth(1).unwrap_or("").trim());
    }

    // Save batch examples
    let mut batch_output = String::new();
    batch_output.push_str("Batch Rendering Examples\n");
    batch_output.push_str("========================\n\n");
    for (name, place) in &[("Charlie", "Mars"), ("Diana", "Venus"), ("Eve", "Jupiter")] {
        let p = Greeting::builder()
            .name(*name)
            .place(*place)
            .build()?;
        batch_output.push_str(&format!("{}:\n{}\n", name, p.render_plain()));
    }
    fs::write("output/greeting_batch.txt", &batch_output)?;

    println!("\n✓ All prompts rendered successfully!");
    println!("\nGenerated files:");
    println!("  output/greeting_alice.xml");
    println!("  output/greeting_alice.md");
    println!("  output/greeting_alice.txt");
    println!("  output/greeting_batch.txt");

    Ok(())
}
