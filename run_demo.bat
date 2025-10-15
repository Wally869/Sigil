@echo off
REM Complete end-to-end demo of Sigil workflow

echo ========================================
echo Sigil End-to-End Workflow Demo
echo ========================================
echo.

echo Step 1: Compiling .sigil file to Rust code...
cargo run --example compile_template prompts/greeting.sigil target/generated_prompt.rs
echo.

echo Step 2: Showing generated code (first 20 lines)...
powershell -Command "Get-Content target/generated_prompt.rs -TotalCount 20"
echo ... (truncated)
echo.

echo Step 3: Using generated code to render prompts...
echo.
cargo run --example use_generated
echo.

echo ========================================
echo Complete workflow finished!
echo ========================================
echo.
echo What happened:
echo   1. prompts/greeting.sigil -^> Rust code (via compile_template)
echo   2. Generated code written to target/generated_prompt.rs
echo   3. use_generated example includes and uses that code
echo   4. Type-safe builder creates Greeting instances
echo   5. Rendered to XML/Markdown/Plain formats
echo.
echo In production, step 1 would happen in build.rs
