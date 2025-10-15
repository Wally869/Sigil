# Sigil Language Specification v0.1

**A Domain-Specific Language for Type-Safe LLM Prompt Templates**

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Lexical Structure](#2-lexical-structure)
3. [Syntax Specification](#3-syntax-specification)
4. [Semantic Rules](#4-semantic-rules)
5. [Type System](#5-type-system)
6. [Code Generation](#6-code-generation)
7. [Rendering Targets](#7-rendering-targets)
8. [Error Handling](#8-error-handling)
9. [Examples](#9-examples)
10. [Implementation Notes](#10-implementation-notes)

---

## 1. Introduction

### 1.1 Purpose

Sigil is a domain-specific language for defining LLM prompt templates with compile-time type safety. It generates idiomatic Rust code with builder patterns, enabling type-checked prompt construction and multiple output format rendering.

### 1.2 Design Goals

- **Type Safety**: Catch prompt errors at compile time
- **Multiple Output Formats**: Render to XML, Markdown, or plain text
- **Ergonomic API**: Generate fluent Rust builders
- **Zero Runtime Overhead**: All parsing happens at build time
- **Extensibility**: Support custom render types

### 1.3 File Format

- **Extension**: `.sigil`
- **Encoding**: UTF-8
- **Line Endings**: LF (`\n`) or CRLF (`\r\n`)

---

## 2. Lexical Structure

### 2.1 Character Set

Sigil source files MUST be valid UTF-8.

### 2.2 Whitespace

Whitespace characters are:
- Space (U+0020)
- Tab (U+0009)
- Line Feed (U+000A)
- Carriage Return (U+000D)

Whitespace is significant for:
- Separating tokens
- Line-based parsing of section content

Whitespace is NOT significant for:
- The number of spaces/tabs (except in content)
- Blank lines between directives

### 2.3 Comments

```
// Single-line comment syntax
// Comments extend to end of line
```

Comments are ignored by the parser and do not appear in generated code or output.

**Syntax:**
- Starts with `//`
- Continues until end of line
- Can appear anywhere except inside string literals

**Example:**
```sigil
// This is a comment
@prompt Example  // Inline comment
```

### 2.4 Keywords

Reserved keywords:
- `@prompt`
- `@description`
- `@end`
- `optional`

Render type keywords:
- `code_block`
- `list`
- `json`
- `xml`
- `plain`

### 2.5 Identifiers

**Syntax:**
```
identifier ::= [a-zA-Z_][a-zA-Z0-9_]*
```

**Rules:**
- Must start with letter or underscore
- Can contain letters, digits, underscores
- Case-sensitive
- Cannot be a keyword

**Conventions:**
- Prompt names: `PascalCase`
- Section names: `snake_case`
- Parameter names: `snake_case`

### 2.6 String Literals

String literals are enclosed in double quotes:
```
string_literal ::= '"' character* '"'
```

**Escape Sequences:**
- `\"` - Double quote
- `\\` - Backslash
- `\n` - Newline
- `\r` - Carriage return
- `\t` - Tab

---

## 3. Syntax Specification

### 3.1 Formal Grammar

```ebnf
(* Top Level *)
prompt_file ::= prompt_directive description_directive? section*

prompt_directive ::= '@prompt' identifier EOL

description_directive ::= '@description' string_literal EOL

(* Sections *)
section ::= section_header section_content '@end' EOL

section_header ::= '@' identifier section_attributes? EOL

section_attributes ::= '[' attribute_list ']'

attribute_list ::= attribute (',' attribute)*

attribute ::= 'optional'

section_content ::= (text_line | parameter_line)*

(* Parameters *)
parameter ::= '{' parameter_body '}'

parameter_body ::= plain_parameter
                 | parameter_with_default
                 | parameter_with_render

plain_parameter ::= identifier

parameter_with_default ::= identifier '=' string_literal

parameter_with_render ::= identifier ':' render_type render_attributes?

render_type ::= 'code_block' | 'list' | 'json' | 'xml' | 'plain'

render_attributes ::= '[' render_attr_list ']'

render_attr_list ::= render_attr (',' render_attr)*

render_attr ::= identifier '=' (string_literal | parameter_reference)

parameter_reference ::= '{' identifier ('=' string_literal)? '}'

(* Lexical *)
identifier ::= [a-zA-Z_][a-zA-Z0-9_]*

string_literal ::= '"' [^"]* '"'

text_line ::= [^\n]* EOL

EOL ::= '\n' | '\r\n'
```

### 3.2 Prompt Directive

**Syntax:**
```sigil
@prompt PromptName
```

**Semantics:**
- MUST be the first directive in the file
- MUST appear exactly once
- Defines the name of the generated Rust struct
- Name MUST be a valid Rust identifier in PascalCase

**Example:**
```sigil
@prompt CodeReview
@prompt DataAnalysis
@prompt ContentGenerator
```

### 3.3 Description Directive

**Syntax:**
```sigil
@description "A human-readable description"
```

**Semantics:**
- OPTIONAL
- MUST appear after `@prompt` and before any sections
- Generates documentation comment in Rust code
- String MUST be a valid string literal

**Example:**
```sigil
@description "Analyzes code for quality and security issues"
```

### 3.4 Section Directive

**Syntax:**
```sigil
@section_name[optional]
content with {parameters}
@end
```

**Components:**

1. **Section Header**: `@section_name[attributes]`
   - Name: Valid identifier in snake_case
   - Attributes: Optional comma-separated list

2. **Section Content**: 
   - Free-form text with parameter placeholders
   - Preserves formatting and whitespace
   - Can span multiple lines

3. **Section Terminator**: `@end`
   - MUST appear on its own line
   - Closes the section

**Attributes:**
- `optional`: Section can be omitted if parameters not provided

**Semantics:**
- Sections are required by default
- Optional sections are rendered only if at least one parameter is provided
- Section names become XML tags or Markdown headers in output
- Multiple sections with same name is an error

**Example:**
```sigil
@system
You are a helpful assistant.
@end

@context[optional]
Additional context: {context_info}
@end
```

### 3.5 Parameters

Parameters are placeholders for values that will be substituted at runtime.

#### 3.5.1 Plain Parameters

**Syntax:**
```sigil
{parameter_name}
```

**Semantics:**
- Simple string substitution
- Parameter is required unless in optional section
- Type: `String` or `Option<String>`

**Example:**
```sigil
@greeting
Hello, {name}!
@end
```

#### 3.5.2 Parameters with Defaults

**Syntax:**
```sigil
{parameter_name="default_value"}
```

**Semantics:**
- If parameter not provided, use default value
- Makes parameter optional
- Type: `Option<String>` with default in builder

**Example:**
```sigil
@output
Format: {format="markdown"}
@end
```

#### 3.5.3 Parameters with Render Types

**Syntax:**
```sigil
{parameter_name:render_type[attr=value, ...]}
```

**Render Types:**

1. **`code_block`**: Renders as fenced code block
   - Attributes:
     - `language`: Programming language for syntax highlighting
     - `file_path`: Optional file path
   - Type: `String`

2. **`list`**: Renders as bulleted list
   - Attributes: None
   - Type: `Vec<String>`
   - Each item becomes a list item

3. **`json`**: Renders as JSON code block
   - Attributes: None
   - Type: `String`
   - Content wrapped in ```json

4. **`xml`**: Renders as XML block
   - Attributes: None
   - Type: `String`

5. **`plain`**: No special rendering
   - Attributes: None
   - Type: `String`

**Examples:**

```sigil
@code
{source_code:code_block[language={lang="rust"}]}
@end

@items
{todo_items:list}
@end

@data
{json_data:json}
@end
```

#### 3.5.4 Render Attributes

Render attributes provide metadata for rendering.

**Syntax:**
```
attribute_name=value
```

**Value Types:**
1. **String Literal**: `"value"`
2. **Parameter Reference**: `{param_name}`
3. **Parameter with Default**: `{param_name="default"}`

**Examples:**
```sigil
{code:code_block[language="rust"]}
{code:code_block[language={lang}]}
{code:code_block[language={lang="python"}, file_path={path="unknown"}]}
```

---

## 4. Semantic Rules

### 4.1 Parameter Resolution

**Rule 1: Unique Parameter Names**
- Parameter names MUST be unique across the entire prompt
- Same parameter can appear in multiple sections
- All occurrences refer to the same value

**Rule 2: Parameter Types**
- Each parameter has exactly one type
- Type is inferred from first declaration
- Subsequent uses must match the type

**Rule 3: Required vs Optional**
- Parameter is required if it appears in any required section without default
- Parameter is optional if:
  - It only appears in optional sections, OR
  - It has a default value

### 4.2 Section Rendering

**Rule 1: Required Sections**
- Always rendered
- All required parameters must be provided

**Rule 2: Optional Sections**
- Rendered if any parameter has a value
- Check is generated in output code

**Rule 3: Section Order**
- Sections are rendered in declaration order
- Order is preserved across all rendering targets

### 4.3 Default Values

**Rule 1: Default Propagation**
- Default value declared once applies everywhere
- Multiple defaults for same parameter is an error

**Rule 2: Default Types**
- Defaults must be string literals
- Applied at build time in generated code

### 4.4 Content Whitespace

**Rule 1: Leading/Trailing Whitespace**
- Leading blank lines in section content are stripped
- Trailing blank lines in section content are stripped
- Interior blank lines are preserved

**Rule 2: Line Indentation**
- Indentation is preserved exactly as written
- No automatic dedenting

**Rule 3: Line Endings**
- Normalized to `\n` in output
- Platform-specific line endings handled by renderer

---

## 5. Type System

### 5.1 Parameter Types

Sigil has a simple type system focused on string manipulation:

```rust
// Generated Rust types
String              // Required plain parameter
Option<String>      // Optional plain parameter or parameter with default
Vec<String>         // List parameter
```

### 5.2 Type Inference

**Algorithm:**

1. Scan all parameter declarations
2. For each unique parameter name:
   - If any declaration has `list` render type → `Vec<String>`
   - Else if any declaration in required section without default → `String`
   - Else → `Option<String>`

**Examples:**

```sigil
@section1
{name}                    // String (required, no default)
@end

@section2[optional]
{email}                   // Option<String> (only in optional section)
@end

@section3
{format="json"}           // Option<String> (has default)
@end

@section4
{items:list}              // Vec<String> (list type)
@end
```

### 5.3 Type Compatibility

**Rule: Single Type Per Parameter**
- Each parameter name maps to exactly one type
- Conflicting type declarations are errors

**Error Examples:**
```sigil
// ERROR: 'data' used as both plain and list
@section1
{data}
@end

@section2
{data:list}
@end
```

---

## 6. Code Generation

### 6.1 Generated Artifacts

For each `.sigil` file, generate:

1. **Struct Definition**: Holds parameter values
2. **Struct Implementation**: Rendering methods
3. **Builder Struct**: Fluent API for construction
4. **Builder Implementation**: Setter methods and `build()`

### 6.2 Struct Generation

**Input:**
```sigil
@prompt Example
@system
Hello {name}
@end
```

**Output:**
```rust
#[derive(Debug, Clone)]
pub struct Example {
    pub name: String,
}
```

**Rules:**
- Struct name matches prompt name
- Fields derived from parameters
- Public visibility
- Derives: `Debug`, `Clone`

### 6.3 Render Method Generation

**Signature:**
```rust
impl Example {
    pub fn render_xml(&self) -> String { /* ... */ }
    pub fn render_markdown(&self) -> String { /* ... */ }
}
```

**Logic:**
1. Create empty output string
2. For each section:
   - If optional: wrap in conditional check
   - Render section header (format-specific)
   - Substitute parameters in content
   - Render parameters with special types
   - Render section footer (format-specific)
3. Return output

**Parameter Substitution:**
- Plain parameters: Direct string interpolation
- Parameters with defaults: Use `.unwrap_or()` 
- Code blocks: Wrap in fenced code with language
- Lists: Iterate and render each item with bullet

### 6.4 Builder Generation

**Structure:**
```rust
#[derive(Default)]
pub struct ExampleBuilder {
    name: Option<String>,
}

impl ExampleBuilder {
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }
    
    pub fn build(self) -> Result<Example, &'static str> {
        Ok(Example {
            name: self.name.ok_or("name is required")?,
        })
    }
}
```

**Rules:**
- Builder struct holds `Option<T>` for each field
- Setter methods:
  - Take `self` by value (move semantics)
  - Accept `impl Into<String>` for ergonomics
  - Return `Self` for chaining
- List parameters get `add_item` method instead
- `build()` method:
  - Returns `Result<Prompt, &'static str>`
  - Validates required fields
  - Applies defaults
  - Constructs prompt struct

**List Parameter Handling:**
```rust
// For {items:list}
pub fn add_item(mut self, item: impl Into<String>) -> Self {
    self.items.get_or_insert_with(Vec::new).push(item.into());
    self
}
```

### 6.5 Error Messages

**Required Field Missing:**
```rust
self.name.ok_or("name is required")?
```

**Format:**
- Simple string literal
- Contains field name
- No additional context needed (compile-time check)

---

## 7. Rendering Targets

### 7.1 XML Format

**Purpose:** Optimized for Claude (Anthropic) and models that prefer structured prompts

**Rules:**

1. **Section Headers:**
   ```xml
   <section_name>
   ```

2. **Section Footers:**
   ```xml
   </section_name>
   ```

3. **Content:**
   - Rendered as plain text inside tags
   - Special characters escaped (if needed)

4. **Code Blocks:**
   ```xml
   <section>
   ```language
   code
   ```
   </section>
   ```

5. **Lists:**
   ```xml
   <items>
   - item 1
   - item 2
   </items>
   ```

**Example Output:**
```xml
<system>
You are a helpful assistant.
</system>

<context>
Additional context: User is working on Rust project
</context>

<code>
```rust
fn main() {}
```
</code>
```

### 7.2 Markdown Format

**Purpose:** Optimized for GPT-4 and human-readable prompts

**Rules:**

1. **Section Headers:**
   ```markdown
   # Section Name
   ```
   - Convert `snake_case` to Title Case
   - Use H1 (`#`) for all sections

2. **Content:**
   - Plain text after header
   - Blank line after header

3. **Code Blocks:**
   ```markdown
   ```language
   code
   ```
   ```

4. **Lists:**
   ```markdown
   - item 1
   - item 2
   ```

**Example Output:**
```markdown
# System

You are a helpful assistant.

# Context

Additional context: User is working on Rust project

# Code

```rust
fn main() {}
```
```

### 7.3 Plain Text Format

**Purpose:** Minimal formatting for simple models or debugging

**Rules:**

1. **Section Headers:**
   ```
   SECTION_NAME:
   ```
   - Convert to UPPERCASE
   - Followed by colon

2. **Content:**
   - Plain text
   - No special formatting

3. **Code Blocks:**
   - No fences, just raw text
   - Optional language label

4. **Lists:**
   - Simple dashes or numbers

**Example Output:**
```
SYSTEM:
You are a helpful assistant.

CONTEXT:
Additional context: User is working on Rust project

CODE:
fn main() {}
```

### 7.4 Format Selection

Users select format at render time:

```rust
let prompt = Example::builder().build()?;

let xml = prompt.render_xml();        // For Claude
let markdown = prompt.render_markdown();  // For GPT-4
let plain = prompt.render_plain();    // For debugging
```

---

## 8. Error Handling

### 8.1 Parse Errors

**Error Types:**

1. **Missing @prompt Directive**
   - Error: "Missing @prompt directive"
   - Fatal: Yes

2. **Duplicate @prompt**
   - Error: "Multiple @prompt directives found"
   - Fatal: Yes

3. **Missing @end**
   - Error: "Section 'name' missing @end terminator"
   - Fatal: Yes

4. **Invalid Identifier**
   - Error: "Invalid identifier 'name'"
   - Fatal: Yes

5. **Unclosed String Literal**
   - Error: "Unclosed string literal"
   - Fatal: Yes

6. **Unknown Render Type**
   - Error: "Unknown render type 'type_name'"
   - Fatal: Yes

### 8.2 Semantic Errors

**Error Types:**

1. **Type Conflict**
   - Error: "Parameter 'name' used with conflicting types"
   - Fatal: Yes

2. **Multiple Defaults**
   - Error: "Parameter 'name' has multiple default values"
   - Fatal: Yes

3. **Duplicate Section**
   - Error: "Section 'name' defined multiple times"
   - Fatal: Yes

### 8.3 Build-Time Errors

Generated Rust code produces compile errors for:

1. **Missing Required Field**
   ```rust
   // Compile error: method `required_param` not called
   ```

2. **Type Mismatch**
   ```rust
   // Compile error: expected String, found i32
   ```

### 8.4 Runtime Errors

**Error Types:**

1. **Build Validation Failure**
   ```rust
   Err("field_name is required")
   ```
   - Returned from `build()` method
   - Indicates required field not set

**Error Handling:**
```rust
let prompt = Example::builder()
    .name("value")
    .build()?;  // Returns Result
```

---

## 9. Examples

### 9.1 Simple Prompt

**Input (`greeting.sigil`):**
```sigil
@prompt Greeting
@description "A simple greeting prompt"

@greeting
Hello, {name}! Welcome to {place="Earth"}.
@end
```

**Generated Code:**
```rust
/// A simple greeting prompt
#[derive(Debug, Clone)]
pub struct Greeting {
    pub name: String,
    pub place: Option<String>,
}

impl Greeting {
    pub fn builder() -> GreetingBuilder {
        GreetingBuilder::default()
    }
    
    pub fn render_xml(&self) -> String {
        let mut output = String::new();
        output.push_str("<greeting>\n");
        output.push_str(&format!("Hello, {}! Welcome to {}.\n",
            self.name,
            self.place.as_deref().unwrap_or("Earth")
        ));
        output.push_str("</greeting>\n");
        output
    }
}

#[derive(Default)]
pub struct GreetingBuilder {
    name: Option<String>,
    place: Option<String>,
}

impl GreetingBuilder {
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }
    
    pub fn place(mut self, value: impl Into<String>) -> Self {
        self.place = Some(value.into());
        self
    }
    
    pub fn build(self) -> Result<Greeting, &'static str> {
        Ok(Greeting {
            name: self.name.ok_or("name is required")?,
            place: self.place,
        })
    }
}
```

**Usage:**
```rust
let greeting = Greeting::builder()
    .name("Alice")
    .place("Wonderland")
    .build()?;

println!("{}", greeting.render_xml());
```

### 9.2 Code Review Prompt

**Input (`code_review.sigil`):**
```sigil
@prompt CodeReview
@description "Reviews code for quality and security"

@system
You are an expert code reviewer specializing in {language="rust"}.
@end

@context[optional]
Project: {project_info}
@end

@code
{source_code:code_block[language={language}]}
@end

@focus_areas[optional]
Pay attention to:
{areas:list}
@end

@output
Provide analysis in {format="markdown"} format.
@end
```

**Usage:**
```rust
let review = CodeReview::builder()
    .language("rust")
    .source_code("fn main() {\n    println!(\"Hello\");\n}")
    .project_info("CLI tool")
    .add_area("error handling")
    .add_area("memory safety")
    .format("json")
    .build()?;

let prompt = review.render_xml();
```

### 9.3 Data Analysis Prompt

**Input (`analysis.sigil`):**
```sigil
@prompt DataAnalysis

@instructions
Analyze the following dataset and provide insights.
@end

@dataset
{data:code_block[language="csv"]}
@end

@questions[optional]
Answer these specific questions:
{questions:list}
@end

@constraints
- Be concise
- Use {tone="professional"} tone
- Provide {num_insights="3"} key insights
@end
```

**Usage:**
```rust
let analysis = DataAnalysis::builder()
    .data("name,age,city\nAlice,30,NYC\nBob,25,LA")
    .add_question("What is the average age?")
    .add_question("Which city is most common?")
    .tone("casual")
    .num_insights("5")
    .build()?;
```

---

## 10. Implementation Notes

### 10.1 Parser Implementation

**Recommended Approach:**

1. **Lexical Analysis:**
   - Character-by-character scanning
   - Track line numbers for error reporting
   - Handle comments before parsing

2. **Parsing Strategy:**
   - Line-oriented parsing for directives
   - Block parsing for section content
   - Recursive descent for parameter expressions

3. **AST Construction:**
   - Build complete AST before code generation
   - Validate during construction
   - Store source locations for errors

### 10.2 Code Generator Implementation

**Recommended Approach:**

1. **Template-Based Generation:**
   - Use string templates for boilerplate
   - Parameterize with prompt data
   - Format with rustfmt if available

2. **Name Hygiene:**
   - Escape Rust keywords if used as identifiers
   - Generate unique names for temporaries
   - Use fully qualified paths where needed

3. **Optimization:**
   - Pre-compute parameter metadata
   - Cache type information
   - Generate efficient string concatenation

### 10.3 Build Script Integration

**Cargo Build Script (`build.rs`):**

```rust
fn main() {
    println!("cargo:rerun-if-changed=prompts/");
    
    // Parse all .sigil files in prompts/
    // Generate code
    // Write to OUT_DIR/generated_prompts.rs
}
```

**Integration:**

```rust
// In lib.rs or main.rs
include!(concat!(env!("OUT_DIR"), "/generated_prompts.rs"));
```

### 10.4 Error Reporting

**Best Practices:**

1. **Include Source Location:**
   ```
   error: missing @end terminator
     --> prompts/example.sigil:10:1
   ```

2. **Provide Context:**
   ```
   error: parameter 'name' used with conflicting types
     --> prompts/example.sigil:5:3
      |
    5 |     {name}
      |     ^^^^^^ used as String here
      |
     --> prompts/example.sigil:10:3
      |
   10 |     {name:list}
      |     ^^^^^^^^^^^ used as Vec<String> here
   ```

3. **Suggest Fixes:**
   ```
   error: unknown render type 'code'
     --> prompts/example.sigil:8:12
      |
    8 |     {code:code}
      |            ^^^^ 
      |
      = help: did you mean 'code_block'?
   ```

### 10.5 Testing Strategy

**Unit Tests:**
- Lexer: Token recognition
- Parser: AST construction
- Generator: Code output

**Integration Tests:**
- Parse → Generate → Compile cycle
- Multiple prompts in one project
- Error recovery

**Example Tests:**
- All examples in spec compile
- Generated code follows Rust conventions
- Rendered output matches expected format

### 10.6 Future Extensions

**Potential Features:**

1. **Custom Render Types:**
   ```sigil
   {data:custom_type[plugin="my_renderer"]}
   ```

2. **Include Directives:**
   ```sigil
   @include common/system_prompt.sigil
   ```

3. **Conditional Sections:**
   ```sigil
   @section[optional, if={debug}]
   Debug info here
   @end
   ```

4. **Validation Rules:**
   ```sigil
   {email:validate[regex=".*@.*"]}
   ```

5. **Multiple Render Methods:**
   ```sigil
   @render xml, markdown, json
   ```

---

## Appendix A: Complete Grammar (EBNF)

```ebnf
(* Sigil Language Grammar *)

prompt_file       = prompt_directive, 
                    [ description_directive ],
                    { section };

prompt_directive  = "@prompt", ws, identifier, eol;

description_directive = "@description", ws, string_literal, eol;

section           = section_header,
                    section_content,
                    "@end", eol;

section_header    = "@", identifier, [ section_attrs ], eol;

section_attrs     = "[", attr_list, "]";

attr_list         = "optional", { ",", "optional" };

section_content   = { content_line };

content_line      = { character | parameter }, eol;

parameter         = "{", param_body, "}";

param_body        = plain_param
                  | param_default
                  | param_render;

plain_param       = identifier;

param_default     = identifier, "=", string_literal;

param_render      = identifier, ":", render_type, [ render_attrs ];

render_type       = "code_block" | "list" | "json" | "xml" | "plain";

render_attrs      = "[", render_attr_list, "]";

render_attr_list  = render_attr, { ",", render_attr };

render_attr       = identifier, "=", ( string_literal | param_ref );

param_ref         = "{", identifier, [ "=", string_literal ], "}";

identifier        = letter, { letter | digit | "_" };

string_literal    = '"', { string_char }, '"';

string_char       = ? any character except '"' or '\' ?
                  | escape_sequence;

escape_sequence   = "\", ( '"' | "\" | "n" | "r" | "t" );

letter            = "a" | "b" | ... | "z" | "A" | "B" | ... | "Z" | "_";

digit             = "0" | "1" | "2" | ... | "9";

ws                = { " " | "\t" };

eol               = "\n" | "\r\n";

character         = ? any UTF-8 character ?;
```

---

## Appendix B: Reserved Words

**Keywords:**
- `@prompt`
- `@description`
- `@end`
- `optional`

**Render Types:**
- `code_block`
- `list`
- `json`
- `xml`
- `plain`

**Future Reserved:**
- `@include`
- `@validate`
- `@render`
- `required`
- `if`
- `unless`

---

## Appendix C: Naming Conventions

**Prompt Names:**
- PascalCase
- Examples: `CodeReview`, `DataAnalysis`, `ContentGenerator`

**Section Names:**
- snake_case
- Examples: `system`, `code_review`, `focus_areas`

**Parameter Names:**
- snake_case
- Examples: `source_code`, `language`, `project_info`

**File Names:**
- snake_case with `.sigil` extension
- Examples: `code_review.sigil`, `data_analysis.sigil`

---

## Appendix D: Comparison with Similar Languages

**vs Handlebars/Mustache:**
- ✓ Type-safe (compile-time checking)
- ✓ Multiple output formats
- ✗ No logic in templates (by design)
- ✗ Less runtime flexibility

**vs Jinja2:**
- ✓ Compile-time code generation
- ✓ Zero runtime overhead
- ✗ No template inheritance
- ✗ No filters or functions

**vs Custom String Templates:**
- ✓ Structured sections
- ✓ Multiple render targets
- ✓ Type safety
- ✓ IDE support (via generated code)

---

## Version History

- **v0.1** (2025-01-15): Initial specification
  - Core syntax defined
  - Three render targets: XML, Markdown, Plain
  - Basic parameter types: String, Option, Vec
  - Builder pattern code generation

---

**End of Specification**