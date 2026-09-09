//! # TOML Streaming Pull Parser Example
//!
//! Demonstrates the event-driven, zero-allocation pull parser (`TomlPullParser`).
//! Ideal for embedded systems, microcontrollers (ARM Cortex-M, ESP32, RISC-V),
//! or processing massive TOML streams with constant O(1) working memory.

use babbel_toml::{TomlPullEvent, TomlPullParser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TOML Zero-Allocation Streaming Pull Parser Example ===\n");

    let toml_stream = r#"
# Embedded system sensor telemetry
[device]
serial = "ESP32-S3-0042"
firmware_version = "2.1.0"
active = true

[sensors.dht22]
temperature = 23.8
humidity = 45.2
interval_sec = 10

[telemetry]
endpoint = "https://iot.babbel.io/v1/stream"
retry_count = 3
tags = [ "edge", "warehouse", "zone-4" ]
"#;

    println!("--- Initializing TomlPullParser over input string ---");
    let mut parser = TomlPullParser::new(toml_stream);

    let mut event_count = 0;
    let mut active_table = String::from("root");
    let mut current_key = String::new();

    println!("{:<6} {:<24} {:<34}", "Step", "Event Kind", "Value / Context");
    println!("{:-<68}", "");

    while let Some(event) = parser.next_event()? {
        event_count += 1;

        match &event {
            TomlPullEvent::StartDocument => {
                println!("{:<6} {:<24} [Start of Document]", event_count, "StartDocument");
            }
            TomlPullEvent::TableHeader(path) => {
                active_table = path.clone();
                println!("{:<6} {:<24} [{}]", event_count, "TableHeader", active_table);
            }
            TomlPullEvent::ArrayOfTablesHeader(path) => {
                active_table = path.clone();
                println!("{:<6} {:<24} [[{}]]", event_count, "ArrayOfTablesHeader", active_table);
            }
            TomlPullEvent::Key(k) => {
                current_key = k.clone();
                println!("{:<6} {:<24} \"{}\"", event_count, "Key", current_key);
            }
            TomlPullEvent::ValueString(s) => {
                println!("{:<6} {:<24} \"{}\" (key: {}, table: {})", event_count, "ValueString", s, current_key, active_table);
            }
            TomlPullEvent::ValueInteger(i) => {
                println!("{:<6} {:<24} {} (key: {}, table: {})", event_count, "ValueInteger", i, current_key, active_table);
            }
            TomlPullEvent::ValueFloat(f) => {
                println!("{:<6} {:<24} {:.2} (table: {})", event_count, "ValueFloat", f, active_table);
            }
            TomlPullEvent::ValueBoolean(b) => {
                println!("{:<6} {:<24} {} (table: {})", event_count, "ValueBoolean", b, active_table);
            }
            TomlPullEvent::ValueDatetime(dt) => {
                println!("{:<6} {:<24} {} (table: {})", event_count, "ValueDatetime", dt, active_table);
            }
            TomlPullEvent::StartArray => {
                println!("{:<6} {:<24} Begin [", event_count, "StartArray");
            }
            TomlPullEvent::EndArray => {
                println!("{:<6} {:<24} End ]", event_count, "EndArray");
            }
            TomlPullEvent::StartInlineTable => {
                println!("{:<6} {:<24} Begin {{", event_count, "StartInlineTable");
            }
            TomlPullEvent::EndInlineTable => {
                println!("{:<6} {:<24} End }}", event_count, "EndInlineTable");
            }
            TomlPullEvent::EndDocument => {
                println!("{:<6} {:<24} [End of Document]", event_count, "EndDocument");
            }
        }
    }

    println!("{:-<68}", "");
    println!("Processed {} streaming events with zero document-level allocations!", event_count);

    Ok(())
}
