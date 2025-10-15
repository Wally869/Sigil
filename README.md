# Sigil - Type-Safe LLM Prompt Templates

A domain-specific language for defining LLM prompt templates with compile-time type safety. Sigil generates idiomatic Rust code with builder patterns, enabling type-checked prompt construction and multiple output format rendering.

## Features

- **Type Safety**: Catch prompt errors at compile time
- **Multiple Output Formats**: Render to XML (Claude), Markdown (GPT-4), or Plain Text
- **Ergonomic API**: Fluent builder pattern with IDE autocomplete
- **Zero Runtime Overhead**: All parsing and validation happens at build time
- **Rich Type System**: Support for required/optional parameters, defaults, and lists

## Quick Start

### 1. Create a `.sigil` file

```sigil
@prompt Greeting
@description "A simple greeting prompt"

@greeting
Hello, {name}! Welcome to {place="Earth"}.
@end
```

### 2. Compile it to Rust

```rust
use sigil;

let code = sigil::compile_sigil_file("greeting.sigil")?;
// Generates a Greeting struct with builder pattern
```

### 3. Use the generated code

```rust
let greeting = Greeting::builder()
    .name("Alice")
    .place("Wonderland")  // optional
    .build()?;

// Render to different formats:
let xml = greeting.render_xml();        // For Claude
let markdown = greeting.render_markdown();  // For GPT-4
let plain = greeting.render_plain();    // For debugging
```

## Examples

Run the demo examples:

```bash
# Simple code generation demo
cargo run --example demo

# End-to-end workflow: compile and use generated code
cargo run --example use_generated

# Comprehensive feature demonstration
cargo run --example complex_demo

# Quick demo script (compiles + renders)
./run_demo.sh    # Mac/Linux
run_demo.bat     # Windows
```

## Syntax Features

### Parameters

```sigil
{name}                    // Required parameter
{lang="rust"}            // Parameter with default value
{items:list}             // List parameter (Vec<String>)
{code:code_block[language={lang}]}  // Special rendering
```

### Optional Sections

```sigil
@context[optional]
Additional info: {info}
@end
```

### Render Types

- `code_block` - Fenced code blocks with syntax highlighting
- `list` - Bulleted lists
- `json` - JSON code blocks
- `xml` - XML blocks
- `plain` - No special formatting

## Output Formats

All three formats are generated from the same `.sigil` template:

### XML (for Claude/Anthropic)

```xml
<greeting>Hello, Alice! Welcome to Wonderland.</greeting>
```

### Markdown (for GPT-4/OpenAI)

```markdown
# Greeting

Hello, Alice! Welcome to Wonderland.
```

### Plain Text

```
GREETING:
Hello, Alice! Welcome to Wonderland.
```

## Project Structure

```
sigil/
├── src/
│   ├── error.rs          # Error types & diagnostics
│   ├── lexer/            # Tokenization
│   ├── parser/           # Parsing & AST
│   ├── semantic/         # Type checking
│   ├── codegen/          # Code generation
│   └── util.rs           # Utilities
├── examples/             # Demo programs
│   ├── demo.rs           # Basic code generation
│   ├── use_generated.rs  # End-to-end example
│   ├── complex_demo.rs   # Feature showcase
│   └── compile_template.rs  # CLI compiler
├── prompts/              # Example .sigil templates
│   ├── greeting.sigil
│   └── ai_code_reviewer.sigil
├── output/               # Generated output samples
├── docs/SPECS.md         # Full language specification
└── run_demo.sh/bat       # Quick demo scripts
```

## Documentation

See [docs/SPECS.md](docs/SPECS.md) for the complete language specification.

## License

See LICENSE file for details.
