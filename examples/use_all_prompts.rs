// Example showing how to use all prompts from generated/

// Include all generated prompts
include!("../src/generated/mod.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Using All Generated Prompts ===\n");

    // Use Greeting prompt
    let greeting = Greeting::builder()
        .name("Alice")
        .build()?;

    println!("Greeting:");
    println!("{}\n", greeting.render_plain());

    // Use CodeReview prompt
    let review = CodeReview::builder()
        .language("rust")
        .source_code("fn main() { println!(\"test\"); }")
        .build()?;

    println!("Code Review:");
    println!("{}\n", review.render_plain());

    // Use AICodeReviewer prompt
    let ai_review = AICodeReviewer::builder()
        .add_expertise("Rust systems programming")
        .file_path("src/main.rs")
        .source_code("fn main() { println!(\"Hello\"); }")
        .build()?;

    println!("AI Code Reviewer:");
    println!("{}\n", ai_review.render_plain());

    println!("✓ All prompts working via generated/mod.rs!");

    Ok(())
}
