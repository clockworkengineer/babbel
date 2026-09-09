//! # TOML Error Handling & Validation Example
//!
//! Demonstrates catching syntax errors, extracting source line, column, and offset,
//! and verifying grammar compliance using `babbel_toml::from_str`.

use babbel_toml::from_str;

fn check_syntax(label: &str, input: &str) {
    println!("--- Testing: {} ---", label);
    println!("Input:\n{}\n", input.trim());

    match from_str(input) {
        Ok(_) => {
            println!("  Result: UNEXPECTED ACCEPT (valid syntax)");
        }
        Err(err) => {
            println!("  Result: REJECTED AS EXPECTED");
            println!("  Message:  {}", err.message);
            println!("  Line:     {}", err.line);
            println!("  Column:   {}", err.column);
            println!("  Offset:   {} bytes", err.position);
        }
    }
    println!();
}

fn main() {
    println!("=== TOML Error Handling & Syntax Diagnostics ===\n");

    // Case 1: Leading zeros in decimal integer
    check_syntax(
        "Leading Zero in Decimal Integer",
        "invalid_decimal = 0123\n",
    );

    // Case 2: Illegal underscore at start of number
    check_syntax(
        "Prefix Underscore in Number",
        "bad_number = _42\n",
    );

    // Case 3: Double consecutive underscores
    check_syntax(
        "Consecutive Underscores",
        "bad_number = 1__000\n",
    );

    // Case 4: Duplicate table definition
    check_syntax(
        "Duplicate Table Header",
        "[server]\nhost = \"localhost\"\n\n[server]\nhost = \"127.0.0.1\"\n",
    );

    // Case 5: Table conflicting with existing static key
    check_syntax(
        "Table Conflicting with Existing Key",
        "fruit = \"apple\"\n\n[fruit]\ncolor = \"red\"\n",
    );

    // Case 6: Whitespace between array-of-tables brackets
    check_syntax(
        "Separated Array Brackets",
        "[ [servers] ]\nname = \"web\"\n",
    );

    // Case 7: Unclosed multiline string
    check_syntax(
        "Unclosed Multiline String",
        "text = \"\"\"this string is never closed\n",
    );

    println!("All error handling diagnostics successfully demonstrated!");
}
