//! Example demonstrating JSON usage in embedded IoT sensor scenarios
//!
//! This example shows patterns for:
//! - Creating sensor readings
//! - Batching data for efficient transmission
//! - Memory-efficient JSON construction
//! - Configuration management

use json_lib::{
    BufferDestination, BufferSource, Node, Numeric, ParserConfig,
    embedded::{ArrayBuilder, ObjectBuilder, config, memory, sensor},
    parse_with_config, stringify,
};

fn main() {
    println!("=== Embedded IoT Sensor Example ===\n");

    // Example 1: Simple temperature sensor reading
    println!("1. Simple Sensor Reading:");
    let temp_reading = sensor::simple_reading("temp_sensor_01", 23.5, 1699876543);
    print_json("Temperature Reading", &temp_reading);

    // Example 2: Multi-value environmental sensor
    println!("\n2. Multi-Value Sensor Reading:");
    let env_values = vec![
        ("temperature", 23.5),
        ("humidity", 45.2),
        ("pressure", 1013.25),
    ];
    let env_reading = sensor::multi_reading("env_sensor_01", &env_values, 1699876543);
    print_json("Environmental Reading", &env_reading);

    // Example 3: Batch sensor readings (efficient for low-power transmission)
    println!("\n3. Batched Sensor Readings:");
    let readings = vec![
        sensor::simple_reading("temp_01", 23.5, 1699876543),
        sensor::simple_reading("temp_01", 23.7, 1699876563),
        sensor::simple_reading("temp_01", 23.4, 1699876583),
    ];
    let batch = sensor::batch_readings("temp_01", readings);
    print_json("Batch of Readings", &batch);

    // Example 4: Memory-efficient construction
    println!("\n4. Memory-Efficient Construction:");
    let reading = ObjectBuilder::with_capacity(4)
        .add_str("device", "motion_01")
        .add_bool("detected", true)
        .add_i64("timestamp", 1699876543)
        .add_u32("count", 5)
        .build();

    let size = memory::estimate_node_size(&reading);
    println!("   Estimated memory: {} bytes", size);
    print_json("Motion Detection", &reading);

    // Example 5: Configuration management
    println!("\n5. Device Configuration:");
    let device_config = config::simple()
        .add_str("wifi_ssid", "IoT_Network")
        .add_str("mqtt_broker", "192.168.1.100")
        .add_i32("sample_interval_ms", 5000)
        .add_i32("transmit_interval_ms", 60000)
        .add_bool("deep_sleep_enabled", true)
        .add_f64("temperature_threshold", 30.0)
        .build();

    print_json("Device Config", &device_config);

    // Read configuration values safely
    println!("\n   Reading config values:");
    if let Some(ssid) = config::get_string(&device_config, "wifi_ssid") {
        println!("   WiFi SSID: {}", ssid);
    }
    if let Some(interval) = config::get_i32(&device_config, "sample_interval_ms") {
        println!("   Sample Interval: {}ms", interval);
    }
    if let Some(deep_sleep) = config::get_bool(&device_config, "deep_sleep_enabled") {
        println!("   Deep Sleep: {}", deep_sleep);
    }

    // Example 6: Parsing with memory limits (for constrained devices)
    println!("\n6. Parsing with Memory Limits:");
    let json_config = r#"{
        "device_id": "sensor_01",
        "location": "room_A",
        "enabled": true,
        "thresholds": {
            "min": 15.0,
            "max": 30.0
        }
    }"#;

    let config = ParserConfig::strict();
    let mut source = BufferSource::new(json_config.as_bytes());

    match parse_with_config(&mut source, &config) {
        Ok(parsed) => {
            println!("   ✓ Parsed configuration successfully");

            if let Some(device_id) = config::get_string(&parsed, "device_id") {
                println!("   Device ID: {}", device_id);
            }

            if let Some(thresholds) = parsed.get("thresholds") {
                if let Some(min) = config::get_f64(thresholds, "min") {
                    println!("   Min Threshold: {}", min);
                }
                if let Some(max) = config::get_f64(thresholds, "max") {
                    println!("   Max Threshold: {}", max);
                }
            }
        }
        Err(e) => println!("   ✗ Parse error: {}", e),
    }

    // Example 7: Array of sensor data
    println!("\n7. Sensor Data Array:");
    let sensor_array = ArrayBuilder::with_capacity(5)
        .add_f64(23.5)
        .add_f64(23.7)
        .add_f64(23.4)
        .add_f64(23.6)
        .add_f64(23.8)
        .build();

    print_json("Temperature History", &sensor_array);

    // Example 8: Complex sensor payload
    println!("\n8. Complex Sensor Payload:");
    let mut status_map = std::collections::HashMap::new();
    status_map.insert("battery".to_string(), Node::Number(Numeric::UInt32(87)));
    status_map.insert(
        "signal_strength".to_string(),
        Node::Number(Numeric::Int32(-65)),
    );
    status_map.insert("uptime".to_string(), Node::Number(Numeric::UInt32(3600)));

    let payload = ObjectBuilder::new()
        .add_str("device", "multi_sensor_01")
        .add_i64("timestamp", 1699876543)
        .add_object("status", status_map)
        .add_node("reading", sensor::simple_reading("temp", 24.1, 1699876543))
        .build();

    let size = memory::estimate_node_size(&payload);
    println!("   Payload size: {} bytes", size);
    print_json("Complex Payload", &payload);

    // Example 9: Calculating memory requirements
    println!("\n9. Memory Requirements Analysis:");
    println!("   Node size: {} bytes", memory::node_size());
    println!("   Numeric size: {} bytes", memory::numeric_size());

    let simple = sensor::simple_reading("test", 1.0, 0);
    println!(
        "   Simple reading: {} bytes",
        memory::estimate_node_size(&simple)
    );

    let multi = sensor::multi_reading("test", &[("a", 1.0), ("b", 2.0)], 0);
    println!(
        "   Multi reading: {} bytes",
        memory::estimate_node_size(&multi)
    );

    println!("\n=== Memory Optimization Tips ===");
    println!("1. Use with_capacity() when size is known");
    println!("2. Reuse buffers when possible");
    println!("3. Use ParserConfig::strict() for constrained devices");
    println!("4. Batch readings to reduce overhead");
    println!("5. Choose appropriate numeric types (i32 vs i64)");
}

fn print_json(label: &str, node: &Node) {
    let mut dest = BufferDestination::new();
    if stringify(node, &mut dest).is_ok() {
        println!("   {}: {}", label, dest.to_string());
    }
}
