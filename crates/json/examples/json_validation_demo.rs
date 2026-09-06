//! JSON Validation Demo
//!
//! Demonstrates fast JSON validation without memory allocation.
//! Useful for embedded systems to reject invalid data early.

use json_lib::{validate_json, BufferSource, ParserConfig};

fn main() {
    println!("=== JSON Validation Demo ===\n");

    // Example 1: Valid simple JSON
    println!("1. Validating simple JSON:");
    let json1 = br#"{"name": "Alice", "age": 30, "active": true}"#;
    match validate_simple(json1) {
        Ok(_) => println!("   ✓ Valid JSON\n"),
        Err(e) => println!("   ✗ Invalid: {}\n", e),
    }

    // Example 2: Valid nested JSON
    println!("2. Validating nested structure:");
    let json2 = br#"{
        "user": {
            "name": "Bob",
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        },
        "devices": ["phone", "tablet", "laptop"]
    }"#;
    match validate_simple(json2) {
        Ok(_) => println!("   ✓ Valid JSON\n"),
        Err(e) => println!("   ✗ Invalid: {}\n", e),
    }

    // Example 3: Invalid JSON - missing quote
    println!("3. Detecting invalid JSON (missing quote):");
    let json3 = br#"{"name: "Alice"}"#;
    match validate_simple(json3) {
        Ok(_) => println!("   ✓ Valid JSON\n"),
        Err(e) => println!("   ✗ Invalid: {}\n", e),
    }

    // Example 4: Invalid JSON - trailing comma
    println!("4. Detecting trailing comma:");
    let json4 = br#"{"a": 1, "b": 2,}"#;
    match validate_simple(json4) {
        Ok(_) => println!("   ✓ Valid JSON\n"),
        Err(e) => println!("   ✗ Invalid: {}\n", e),
    }

    // Example 5: Configuration limits - depth
    println!("5. Checking depth limits:");
    let deep_json = br#"{"a":{"b":{"c":{"d":{"e":1}}}}}"#;
    let mut source = BufferSource::new(deep_json);
    let shallow_config = ParserConfig::new().with_max_depth(Some(3));

    match validate_json(&mut source, &shallow_config) {
        Ok(_) => println!("   ✓ Within depth limit\n"),
        Err(e) => println!("   ✗ Exceeds limit: {}\n", e),
    }

    // Example 6: Configuration limits - string length
    println!("6. Checking string length limits:");
    let long_string = format!(r#"{{"data": "{}"}}"#, "x".repeat(300));
    let mut source = BufferSource::new(long_string.as_bytes());
    let strict_config = ParserConfig::strict(); // max 256 bytes

    match validate_json(&mut source, &strict_config) {
        Ok(_) => println!("   ✓ Within string limit\n"),
        Err(e) => println!("   ✗ Exceeds limit: {}\n", e),
    }

    // Example 7: Valid number formats
    println!("7. Validating various number formats:");
    let numbers: Vec<(&[u8], &str)> = vec![
        (b"123", "integer"),
        (b"-456", "negative"),
        (b"3.14", "decimal"),
        (b"1e10", "exponential"),
        (b"2.5e-3", "scientific"),
    ];

    for (json, desc) in numbers {
        let mut source = BufferSource::new(json);
        let config = ParserConfig::new();
        match validate_json(&mut source, &config) {
            Ok(_) => println!("   ✓ {} ({})", std::str::from_utf8(json).unwrap(), desc),
            Err(e) => println!("   ✗ {}: {}", desc, e),
        }
    }
    println!();

    // Example 8: Invalid number formats
    println!("8. Detecting invalid numbers:");
    let invalid_numbers: Vec<(&[u8], &str)> = vec![
        (b"01", "leading zero"),
        (b"-.5", "no digit before decimal"),
        (b"1.", "no digit after decimal"),
    ];

    for (json, desc) in invalid_numbers {
        let mut source = BufferSource::new(json);
        let config = ParserConfig::new();
        match validate_json(&mut source, &config) {
            Ok(_) => println!("   ✗ Should have failed: {}", desc),
            Err(e) => println!("   ✓ Correctly rejected {}: {}", desc, e),
        }
    }
    println!();

    // Example 9: Large array validation
    println!("9. Validating array size limits:");
    let large_array = format!(
        "[{}]",
        (0..100)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    let mut source = BufferSource::new(large_array.as_bytes());
    let small_config = ParserConfig::strict().with_max_array_size(Some(50));

    match validate_json(&mut source, &small_config) {
        Ok(_) => println!("   ✓ Within array limit\n"),
        Err(e) => println!("   ✗ Exceeds limit: {}\n", e),
    }

    // Example 10: Embedded use case - sensor data validation
    println!("10. Embedded use case - validating sensor data:");
    let sensor_readings: Vec<&[u8]> = vec![
        br#"{"id":"temp_01","value":23.5,"ts":1234567890}"#.as_slice(),
        br#"{"id":"hum_02","value":45.2,"ts":1234567891}"#.as_slice(),
        br#"{"id":"bad","value":,"ts":123}"#.as_slice(), // Invalid
    ];

    let config = ParserConfig::strict();
    for (i, reading) in sensor_readings.iter().enumerate() {
        let mut source = BufferSource::new(*reading);
        match validate_json(&mut source, &config) {
            Ok(_) => println!("   ✓ Reading {} is valid", i + 1),
            Err(e) => println!("   ✗ Reading {} invalid: {}", i + 1, e),
        }
    }

    println!("\n=== Validation complete ===");
}

fn validate_simple(json: &[u8]) -> Result<(), String> {
    let mut source = BufferSource::new(json);
    let config = ParserConfig::new();
    validate_json(&mut source, &config)
}
