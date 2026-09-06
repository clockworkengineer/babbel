//! Example demonstrating parser configuration for memory-constrained embedded systems
//!
//! This example shows how to use ParserConfig to control resource usage when parsing JSON.

use json_lib::{BufferSource, ParserConfig, parse_with_config};

fn main() {
    println!("=== JSON Parser Configuration Examples ===\n");

    // Example 1: Default configuration (embedded-friendly)
    println!("1. Default Configuration (embedded-friendly):");
    let config = ParserConfig::new();
    println!("   Max depth: {:?}", config.max_depth);
    println!("   Max string length: {:?}", config.max_string_length);
    println!("   Max array size: {:?}", config.max_array_size);
    println!("   Max object size: {:?}", config.max_object_size);

    let json = r#"{"name":"sensor","values":[1,2,3,4,5]}"#;
    let mut source = BufferSource::new(json.as_bytes());
    match parse_with_config(&mut source, &config) {
        Ok(node) => println!("   ✓ Parsed successfully: {:?}\n", node),
        Err(e) => println!("   ✗ Parse error: {}\n", e),
    }

    // Example 2: Strict configuration (highly constrained)
    println!("2. Strict Configuration (ultra-low memory):");
    let config = ParserConfig::strict();
    println!("   Max depth: {:?}", config.max_depth);
    println!("   Max string length: {:?}", config.max_string_length);
    println!("   Max array size: {:?}", config.max_array_size);
    println!("   Max object size: {:?}", config.max_object_size);

    let json = r#"{"sensor":"temp","value":23.5}"#;
    let mut source = BufferSource::new(json.as_bytes());
    match parse_with_config(&mut source, &config) {
        Ok(_node) => println!("   ✓ Parsed successfully\n"),
        Err(e) => println!("   ✗ Parse error: {}\n", e),
    }

    // Example 3: Depth limit exceeded
    println!("3. Demonstrating Depth Limit:");
    let config = ParserConfig::new().with_max_depth(Some(3));
    let deeply_nested = r#"{"a":{"b":{"c":{"d":1}}}}"#;
    let mut source = BufferSource::new(deeply_nested.as_bytes());
    match parse_with_config(&mut source, &config) {
        Ok(_) => println!("   ✓ Parsed (unexpected)"),
        Err(e) => println!("   ✗ Expected error: {}\n", e),
    }

    // Example 4: String length limit
    println!("4. Demonstrating String Length Limit:");
    let config = ParserConfig::new().with_max_string_length(Some(10));
    let long_string = r#"{"key":"this is a very long string that exceeds the limit"}"#;
    let mut source = BufferSource::new(long_string.as_bytes());
    match parse_with_config(&mut source, &config) {
        Ok(_) => println!("   ✓ Parsed (unexpected)"),
        Err(e) => println!("   ✗ Expected error: {}\n", e),
    }

    // Example 5: Array size limit
    println!("5. Demonstrating Array Size Limit:");
    let config = ParserConfig::new().with_max_array_size(Some(5));
    let large_array = r#"[1,2,3,4,5,6,7,8,9,10]"#;
    let mut source = BufferSource::new(large_array.as_bytes());
    match parse_with_config(&mut source, &config) {
        Ok(_) => println!("   ✓ Parsed (unexpected)"),
        Err(e) => println!("   ✗ Expected error: {}\n", e),
    }

    // Example 6: Object size limit
    println!("6. Demonstrating Object Size Limit:");
    let config = ParserConfig::new().with_max_object_size(Some(2));
    let large_object = r#"{"a":1,"b":2,"c":3,"d":4}"#;
    let mut source = BufferSource::new(large_object.as_bytes());
    match parse_with_config(&mut source, &config) {
        Ok(_) => println!("   ✓ Parsed (unexpected)"),
        Err(e) => println!("   ✗ Expected error: {}\n", e),
    }

    // Example 7: Custom configuration for specific use case
    println!("7. Custom Configuration (IoT sensor data):");
    let config = ParserConfig::new()
        .with_max_depth(Some(4)) // Sensor data rarely deeply nested
        .with_max_string_length(Some(64)) // Short identifiers
        .with_max_array_size(Some(100)) // Moderate sensor readings
        .with_max_object_size(Some(20)); // Few fields per reading

    let sensor_data = r#"{
        "device": "temp_sensor_01",
        "timestamp": 1699876543,
        "readings": [
            {"temp": 23.5, "humidity": 45.2},
            {"temp": 23.7, "humidity": 45.1}
        ]
    }"#;
    let mut source = BufferSource::new(sensor_data.as_bytes());
    match parse_with_config(&mut source, &config) {
        Ok(node) => {
            println!("   ✓ Parsed sensor data successfully");

            // Demonstrate safe access methods
            if let Some(device) = node.get("device").and_then(|n| n.as_str()) {
                println!("   Device: {}", device);
            }

            if let Some(readings) = node.get("readings").and_then(|n| n.as_array()) {
                println!("   Readings count: {}", readings.len());

                // Use safe array access
                if let Some(first) = readings.first() {
                    if let Some(temp) = first.get("temp") {
                        println!("   First reading temperature: {:?}", temp);
                    }
                }
            }
            println!();
        }
        Err(e) => println!("   ✗ Parse error: {}\n", e),
    }

    // Example 8: Unlimited configuration (not recommended for embedded)
    println!("8. Unlimited Configuration (desktop/server use):");
    let config = ParserConfig::unlimited();
    println!("   Max depth: {:?}", config.max_depth);
    println!("   Max string length: {:?}", config.max_string_length);
    println!("   WARNING: Not recommended for embedded systems!");
    println!("   Use this only on systems with abundant memory.\n");

    // Example 9: Memory usage estimation
    println!("9. Memory Usage Estimation:");
    println!(
        "   Node enum size: {} bytes",
        std::mem::size_of::<json_lib::Node>()
    );
    println!("   With strict config:");
    println!("     Max string: 256 bytes");
    println!("     Max array: 64 × 56 bytes = 3,584 bytes");
    println!("     Max object: 32 × 80 bytes ≈ 2,560 bytes");
    println!("     Per level: ~6.4 KB");
    println!("     Total (16 levels): ~102 KB");
    println!();

    println!("   With default config:");
    println!("     Max string: 4,096 bytes");
    println!("     Max array: 1,024 × 56 bytes = 57,344 bytes");
    println!("     Max object: 256 × 80 bytes ≈ 20,480 bytes");
    println!("     Per level: ~81.9 KB");
    println!("     Total (32 levels): ~2.6 MB");
    println!();

    println!("=== Recommendations for Different Platforms ===\n");

    println!("Ultra-Low Memory (<16KB RAM):");
    println!("  ParserConfig::strict().with_max_depth(Some(8))");
    println!();

    println!("Low Memory (16-64KB RAM):");
    println!("  ParserConfig::strict()");
    println!();

    println!("Medium Memory (64-256KB RAM):");
    println!("  ParserConfig::new().with_max_depth(Some(16))");
    println!();

    println!("Higher Memory (>256KB RAM):");
    println!("  ParserConfig::new() // Use defaults");
    println!();
}
