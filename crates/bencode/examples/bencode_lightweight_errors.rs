//! Lightweight error handling example for embedded systems
//!
//! **Best Practice:** Use `BencodeError` for error handling in embedded or size-sensitive applications.
//! This error type is stack-only, requires no heap allocation, and is ideal for no_std and deterministic environments.
//!
//! Demonstrates the use of BencodeError enum which doesn't require heap allocation,
//! making it ideal for no_std and embedded environments with limited memory.

use bencode_lib::{parse_bytes, BencodeError};

fn main() {
    println!("=== Lightweight Error Handling for Embedded Systems ===\n");

    // Example 1: Basic error handling with BencodeError
    println!("1. Basic error handling:");
    let invalid_data = b"i42"; // Missing 'e' terminator

    match parse_bytes(invalid_data) {
        Ok(node) => println!("   Parsed: {}", node),
        Err(e) => {
            // Convert String error to BencodeError
            let bencode_err: BencodeError = e.into();
            println!("   Error: {}", bencode_err);
            println!("   Error code: {}", bencode_err.code());
            println!("   Static string: {}", bencode_err.as_str());
        }
    }

    // Example 2: Pattern matching on error variants
    println!("\n2. Pattern matching on errors:");
    let test_cases = vec![
        ("Empty input", b"" as &[u8]),
        ("Invalid integer", b"i-0e"),
        ("Unterminated integer", b"i42"),
        ("Invalid string length", b"10:short"),
        ("Unterminated list", b"li1ei2e"),
        ("Unterminated dict", b"d3:key5:value"),
        ("Unordered keys", b"d3:zzz5:value3:aaa5:valuee"),
    ];

    for (description, data) in test_cases {
        match parse_bytes(data) {
            Ok(_) => println!("   {}: OK", description),
            Err(e) => {
                let err: BencodeError = e.into();
                match err {
                    BencodeError::EmptyInput => {
                        println!("   {}: Empty (code={})", description, err.code())
                    }
                    BencodeError::InvalidInteger => {
                        println!("   {}: Bad integer (code={})", description, err.code())
                    }
                    BencodeError::UnterminatedInteger => println!(
                        "   {}: No 'e' terminator (code={})",
                        description,
                        err.code()
                    ),
                    BencodeError::InvalidStringLength => println!(
                        "   {}: String length issue (code={})",
                        description,
                        err.code()
                    ),
                    BencodeError::UnterminatedList => {
                        println!("   {}: List not closed (code={})", description, err.code())
                    }
                    BencodeError::UnterminatedDictionary => {
                        println!("   {}: Dict not closed (code={})", description, err.code())
                    }
                    BencodeError::DictKeysOutOfOrder => {
                        println!("   {}: Keys not sorted (code={})", description, err.code())
                    }
                    _ => println!("   {}: Other error (code={})", description, err.code()),
                }
            }
        }
    }

    // Example 3: Error codes for compact reporting
    println!("\n3. Compact error reporting (useful for embedded systems):");
    println!("   Using error codes instead of strings saves memory:");

    let errors = vec![
        BencodeError::EmptyInput,
        BencodeError::InvalidInteger,
        BencodeError::UnterminatedInteger,
        BencodeError::InvalidStringLength,
        BencodeError::UnterminatedList,
        BencodeError::UnterminatedDictionary,
        BencodeError::DictKeysOutOfOrder,
    ];

    for err in errors {
        println!("   {} -> code {}", err.as_str(), err.code());
    }

    // Example 4: No allocation error handling
    println!("\n4. Zero-allocation error handling:");
    println!(
        "   BencodeError size: {} bytes",
        core::mem::size_of::<BencodeError>()
    );
    println!(
        "   String error size: {} bytes",
        core::mem::size_of::<String>()
    );
    println!(
        "   Result<(), BencodeError> size: {} bytes",
        core::mem::size_of::<Result<(), BencodeError>>()
    );

    // Example 5: Using error codes in embedded context
    println!("\n5. Simulated embedded error reporting:");
    simulate_embedded_error_reporting();

    println!("\n=== Advantages of Lightweight Errors ===");
    println!("✓ No heap allocation - BencodeError is stack-only");
    println!("✓ Small size - fits in a single byte (as error code)");
    println!("✓ Fast comparison - enum variants are efficient");
    println!("✓ no_std compatible - works without standard library");
    println!("✓ Deterministic - no allocation failures possible");
    println!("✓ Easy serialization - error codes are simple integers");
}

/// Simulates error reporting in an embedded system with limited resources
fn simulate_embedded_error_reporting() {
    // In embedded systems, you might log error codes instead of strings
    let test_data = vec![
        b"i42e" as &[u8], // Valid
        b"i42",           // Invalid
        b"",              // Empty
        b"d3:foo3:bare",  // Valid
        b"d3:foo3:bar",   // Invalid (unterminated)
    ];

    let mut error_log: [u8; 16] = [0; 16]; // Fixed-size error log
    let mut error_count = 0;

    for (i, data) in test_data.iter().enumerate() {
        match parse_bytes(data) {
            Ok(_) => {
                println!("   Entry {}: OK", i);
            }
            Err(e) => {
                let err: BencodeError = e.into();
                if error_count < error_log.len() {
                    error_log[error_count] = err.code();
                    error_count += 1;
                }
                println!("   Entry {}: ERROR (code={})", i, err.code());
            }
        }
    }

    println!(
        "   Error log: {:?} ({} errors)",
        &error_log[..error_count],
        error_count
    );
}
