//! JSON Library Performance Benchmarks
//!
//! This benchmark suite measures:
//! - Parsing speed for various JSON structures
//! - Memory usage during parsing and construction
//! - Impact of parser configuration limits
//! - Performance with different features enabled
//!
//! Run with: cargo run --release --example json_benchmarks

use json_lib::{
    BufferDestination, BufferSource, ParserConfig,
    embedded::{ArrayBuilder, ObjectBuilder, memory, sensor},
    parse, parse_with_config, stringify,
};
use std::time::{Duration, Instant};

/// Benchmark result
struct BenchmarkResult {
    name: String,
    duration: Duration,
    iterations: u32,
    ops_per_sec: f64,
    memory_estimate: Option<usize>,
}

impl BenchmarkResult {
    fn new(name: String, duration: Duration, iterations: u32) -> Self {
        let ops_per_sec = iterations as f64 / duration.as_secs_f64();
        Self {
            name,
            duration,
            iterations,
            ops_per_sec,
            memory_estimate: None,
        }
    }

    #[allow(dead_code)]
    fn with_memory(mut self, bytes: usize) -> Self {
        self.memory_estimate = Some(bytes);
        self
    }
}

fn main() {
    println!("=== JSON Library Performance Benchmarks ===\n");
    println!("Running benchmarks...\n");

    // Parsing benchmarks
    println!("--- Parsing Benchmarks ---");
    bench_parse_small();
    bench_parse_medium();
    bench_parse_large();
    bench_parse_nested();
    println!();

    // Parser config benchmarks
    println!("--- Parser Config Benchmarks ---");
    bench_config_default();
    bench_config_strict();
    bench_config_unlimited();
    println!();

    // Construction benchmarks
    println!("--- Construction Benchmarks ---");
    bench_construct_object();
    bench_construct_array();
    bench_construct_sensor();
    println!();

    // Stringify benchmarks
    println!("--- Stringify Benchmarks ---");
    bench_stringify_small();
    bench_stringify_medium();
    bench_stringify_large();
    println!();

    // Memory benchmarks
    println!("--- Memory Benchmarks ---");
    bench_memory_small();
    bench_memory_medium();
    bench_memory_large();
    println!();

    println!("Benchmarks complete!");
}

/// Small JSON: ~50 bytes
fn bench_parse_small() {
    let json = r#"{"id":"sensor_01","value":23.5,"timestamp":1234567890}"#;
    let result = benchmark_parse("Parse Small JSON (50 bytes)", json, 50_000);
    print_result(&result);
}

/// Medium JSON: ~500 bytes
fn bench_parse_medium() {
    let json = r#"
    {
        "device": "sensor_01",
        "location": "room_A",
        "readings": [
            {"sensor": "temp", "value": 23.5, "unit": "C"},
            {"sensor": "humidity", "value": 45.2, "unit": "%"},
            {"sensor": "pressure", "value": 1013.25, "unit": "hPa"}
        ],
        "metadata": {
            "firmware": "v2.1.0",
            "battery": 85,
            "signal": -67
        },
        "timestamp": 1234567890
    }
    "#;
    let result = benchmark_parse("Parse Medium JSON (500 bytes)", json, 10_000);
    print_result(&result);
}

/// Large JSON: ~2KB
fn bench_parse_large() {
    let json = format!(
        r#"{{
        "device": "sensor_01",
        "batch": [{}]
    }}"#,
        (0..50)
            .map(|i| {
                format!(
                    r#"{{"id":"reading_{}","temp":23.5,"humidity":45.2,"ts":{}}}"#,
                    i,
                    1234567890 + i
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    );
    let result = benchmark_parse("Parse Large JSON (2KB)", &json, 5_000);
    print_result(&result);
}

/// Deeply nested JSON
fn bench_parse_nested() {
    let json = r#"{"a":{"b":{"c":{"d":{"e":{"f":{"g":{"h":"value"}}}}}}}}"#;
    let result = benchmark_parse("Parse Nested JSON (8 levels)", json, 20_000);
    print_result(&result);
}

/// Parse with default config
fn bench_config_default() {
    let json = r#"{"data":[1,2,3,4,5,6,7,8,9,10]}"#;
    let config = ParserConfig::new();
    let result = benchmark_parse_with_config("Parse with Default Config", json, &config, 30_000);
    print_result(&result);
}

/// Parse with strict config
fn bench_config_strict() {
    let json = r#"{"data":[1,2,3,4,5,6,7,8,9,10]}"#;
    let config = ParserConfig::strict();
    let result = benchmark_parse_with_config("Parse with Strict Config", json, &config, 30_000);
    print_result(&result);
}

/// Parse with unlimited config
fn bench_config_unlimited() {
    let json = r#"{"data":[1,2,3,4,5,6,7,8,9,10]}"#;
    let config = ParserConfig::unlimited();
    let result = benchmark_parse_with_config("Parse with Unlimited Config", json, &config, 30_000);
    print_result(&result);
}

/// Construct object with builder
fn bench_construct_object() {
    let iterations = 100_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _node = ObjectBuilder::with_capacity(5)
            .add_str("id", "sensor_01")
            .add_f64("value", 23.5)
            .add_i64("timestamp", 1234567890)
            .add_bool("enabled", true)
            .add_null("error")
            .build();
    }

    let duration = start.elapsed();
    let result = BenchmarkResult::new(
        "Construct Object (5 fields)".to_string(),
        duration,
        iterations,
    );
    print_result(&result);
}

/// Construct array with builder
fn bench_construct_array() {
    let iterations = 100_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _node = ArrayBuilder::with_capacity(10)
            .add_i32(1)
            .add_i32(2)
            .add_i32(3)
            .add_i32(4)
            .add_i32(5)
            .add_i32(6)
            .add_i32(7)
            .add_i32(8)
            .add_i32(9)
            .add_i32(10)
            .build();
    }

    let duration = start.elapsed();
    let result = BenchmarkResult::new(
        "Construct Array (10 elements)".to_string(),
        duration,
        iterations,
    );
    print_result(&result);
}

/// Construct sensor reading
fn bench_construct_sensor() {
    let iterations = 100_000u32;
    let start = Instant::now();

    for i in 0..iterations {
        let _reading =
            sensor::simple_reading("temp_01", 23.5 + (i as f64 * 0.01), 1234567890 + (i as i64));
    }

    let duration = start.elapsed();
    let result = BenchmarkResult::new("Construct Sensor Reading".to_string(), duration, iterations);
    print_result(&result);
}

/// Stringify small structure
fn bench_stringify_small() {
    let node = ObjectBuilder::with_capacity(3)
        .add_str("id", "s1")
        .add_i32("value", 42)
        .add_i64("ts", 1234567890)
        .build();

    let iterations = 50_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
    }

    let duration = start.elapsed();
    let result = BenchmarkResult::new("Stringify Small Object".to_string(), duration, iterations);
    print_result(&result);
}

/// Stringify medium structure
fn bench_stringify_medium() {
    let readings = ArrayBuilder::with_capacity(3)
        .add_node(sensor::simple_reading("temp", 23.5, 1234567890))
        .add_node(sensor::simple_reading("humidity", 45.2, 1234567891))
        .add_node(sensor::simple_reading("pressure", 1013.25, 1234567892))
        .build();

    let node = ObjectBuilder::with_capacity(3)
        .add_str("device", "sensor_01")
        .add_node("readings", readings)
        .add_i64("timestamp", 1234567890)
        .build();

    let iterations = 10_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
    }

    let duration = start.elapsed();
    let result = BenchmarkResult::new("Stringify Medium Object".to_string(), duration, iterations);
    print_result(&result);
}

/// Stringify large structure
fn bench_stringify_large() {
    let mut readings = Vec::new();
    for i in 0..50 {
        readings.push(sensor::simple_reading(
            "sensor",
            23.5 + (i as f64 * 0.1),
            1234567890 + i,
        ));
    }
    let node = sensor::batch_readings("device_01", readings);

    let iterations = 2_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
    }

    let duration = start.elapsed();
    let result = BenchmarkResult::new("Stringify Large Batch".to_string(), duration, iterations);
    print_result(&result);
}

/// Memory usage for small structure
fn bench_memory_small() {
    let node = ObjectBuilder::with_capacity(3)
        .add_str("id", "sensor_01")
        .add_f64("value", 23.5)
        .add_i64("ts", 1234567890)
        .build();

    let size = memory::estimate_node_size(&node);
    println!("Small Object Memory: {} bytes", size);
}

/// Memory usage for medium structure
fn bench_memory_medium() {
    let node = sensor::multi_reading(
        "sensor_01",
        &[
            ("temperature", 23.5),
            ("humidity", 45.2),
            ("pressure", 1013.25),
        ],
        1234567890,
    );

    let size = memory::estimate_node_size(&node);
    println!("Medium Sensor Reading: {} bytes", size);
}

/// Memory usage for large structure
fn bench_memory_large() {
    let mut readings = Vec::new();
    for i in 0..50 {
        readings.push(sensor::simple_reading("sensor", 23.5, 1234567890 + i));
    }
    let node = sensor::batch_readings("device_01", readings);

    let size = memory::estimate_node_size(&node);
    println!("Large Batch (50 readings): {} bytes", size);
}

// Helper functions

fn benchmark_parse(name: &str, json: &str, iterations: u32) -> BenchmarkResult {
    let start = Instant::now();

    for _ in 0..iterations {
        let mut source = BufferSource::new(json.as_bytes());
        let _ = parse(&mut source).unwrap();
    }

    let duration = start.elapsed();
    BenchmarkResult::new(name.to_string(), duration, iterations)
}

fn benchmark_parse_with_config(
    name: &str,
    json: &str,
    config: &ParserConfig,
    iterations: u32,
) -> BenchmarkResult {
    let start = Instant::now();

    for _ in 0..iterations {
        let mut source = BufferSource::new(json.as_bytes());
        let _ = parse_with_config(&mut source, config).unwrap();
    }

    let duration = start.elapsed();
    BenchmarkResult::new(name.to_string(), duration, iterations)
}

fn print_result(result: &BenchmarkResult) {
    println!("{}", result.name);
    println!("  Iterations: {}", result.iterations);
    println!("  Duration: {:.3}s", result.duration.as_secs_f64());
    println!("  Throughput: {:.0} ops/sec", result.ops_per_sec);
    if let Some(mem) = result.memory_estimate {
        println!("  Memory: {} bytes", mem);
    }
    println!();
}
