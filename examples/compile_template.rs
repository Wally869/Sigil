// Step 1: Compile a .sigil file and write the generated Rust code to a file
// Usage: cargo run --example compile_template <input.sigil> <output.rs>

use sigil;
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input.sigil> <output.rs>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    println!("Compiling {} to {}", input_path, output_path);

    // Compile the .sigil file
    let generated_code = sigil::compile_sigil_file(input_path)?;

    // Write to output file
    fs::write(output_path, generated_code)?;

    println!("✓ Generated {} bytes", fs::metadata(output_path)?.len());

    Ok(())
}
