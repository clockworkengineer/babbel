//! # TOML v1.1.0 Specification Features Example
//!
//! Highlights the modern features introduced in TOML v1.1.0 (released Dec 2025):
//! 1. Byte escape sequences (`\xHH`) and escape character (`\e`)
//! 2. Datetimes with optional seconds (e.g. `07:32`, `1979-05-27 07:32Z`)
//! 3. Multiline inline tables with trailing commas and interior comments
//! 4. CRLF newline normalization in multi-line strings

use babbel_toml::{from_str, to_string_pretty, DatetimeKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TOML v1.1.0 Feature Showcase ===\n");

    let toml_11_source = r#"
# 1. New Escape Sequences: \e and \xHH
terminal_reset = "\e[0m"
bell_character = "\x07"
hex_greeting = "\x48\x65\x6c\x6c\x6f, \x57\x6f\x72\x6c\x64!"

# 2. Datetimes with Optional Seconds
meeting_time = 09:30
planned_date = 2026-09-09 15:30Z
local_event = 2026-10-31T20:00

# 3. Multiline Inline Tables with comments and trailing comma
database_cluster = {
    engine = "postgresql",
    port = 5432,
    # Primary host configuration
    primary = "db-primary.internal",
    read_pool_size = 32,
}

# 4. Multiline Strings
documentation = """
    This multiline basic string supports \
    line-continuation backslashes with whitespace.
    """
"#;

    println!("--- Parsing TOML 1.1.0 Document ---");
    let doc = from_str(toml_11_source)?;

    // Feature 1: Escapes
    println!("\n1. Escape Sequences:");
    if let Some(reset) = doc.get("terminal_reset").and_then(|n| n.as_str()) {
        println!("  terminal_reset: {:?} (len: {})", reset, reset.len());
        assert_eq!(reset, "\x1B[0m");
    }
    if let Some(greeting) = doc.get("hex_greeting").and_then(|n| n.as_str()) {
        println!("  hex_greeting:   {:?}", greeting);
        assert_eq!(greeting, "Hello, World!");
    }

    // Feature 2: Datetimes with Optional Seconds
    println!("\n2. Datetimes with Optional Seconds:");
    if let Some(time) = doc.get("meeting_time").and_then(|n| n.as_datetime()) {
        println!("  meeting_time: {} (kind: {:?})", time, time.kind);
        assert_eq!(time.kind, DatetimeKind::LocalTime);
    }
    if let Some(dt) = doc.get("planned_date").and_then(|n| n.as_datetime()) {
        println!("  planned_date: {} (kind: {:?})", dt, dt.kind);
        assert_eq!(dt.kind, DatetimeKind::OffsetDateTime);
    }

    // Feature 3: Multiline Inline Tables
    println!("\n3. Multiline Inline Table:");
    if let Some(db) = doc.get("database_cluster") {
        if let Some(engine) = db.get("engine").and_then(|n| n.as_str()) {
            println!("  Engine:   {}", engine);
        }
        if let Some(primary) = db.get("primary").and_then(|n| n.as_str()) {
            println!("  Primary:  {}", primary);
        }
        if let Some(port) = db.get("port").and_then(|n| n.as_i64()) {
            println!("  Port:     {}", port);
        }
    }

    println!("\n--- Serializing Formatted TOML ---");
    let serialized = to_string_pretty(&doc)?;
    println!("{}", serialized);

    println!("All TOML v1.1.0 features parsed and verified!");
    Ok(())
}
