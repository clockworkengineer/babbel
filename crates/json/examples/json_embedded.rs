//! Embedded Compatibility Test Example
//!
//! This example validates that babbel_json can be built and used for embedded systems.

use babbel_json::embedded::{sensor, ArrayBuilder, ObjectBuilder};
use babbel_json::{parse_with_config, stringify, BufferDestination, BufferSource, ParserConfig};

pub fn test_sensor_reading() {
    let _reading = sensor::simple_reading("temp_01", 23.5, 1234567890);
}

pub fn test_object_builder() {
    let _config = ObjectBuilder::with_capacity(3)
        .add_str("device", "sensor_01")
        .add_i32("rate", 1000)
        .add_bool("enabled", true)
        .build();
}

pub fn test_array_builder() {
    let _arr = ArrayBuilder::with_capacity(5)
        .add_i32(1)
        .add_i32(2)
        .add_i32(3)
        .add_i32(4)
        .add_i32(5)
        .build();
}

pub fn test_parsing() {
    let json = br#"{"id":"sensor_01","value":23.5}"#;
    let config = ParserConfig::strict();
    let mut source = BufferSource::new(json);
    let _ = parse_with_config(&mut source, &config);
}

pub fn test_stringify() {
    let node = ObjectBuilder::new()
        .add_str("test", "value")
        .build();
    let mut dest = BufferDestination::new();
    let _ = stringify(&node, &mut dest);
}

pub fn test_batch_readings() {
    let mut readings = Vec::new();
    for i in 0..10 {
        readings.push(sensor::simple_reading("sensor", 23.5, 1234567890 + i));
    }
    let _batch = sensor::batch_readings("device_01", readings);
}

fn main() {
    println!("Testing embedded utilities...");
    test_sensor_reading();
    test_object_builder();
    test_array_builder();
    test_parsing();
    test_stringify();
    test_batch_readings();
    println!("Embedded compatibility tests passed!");
}
