//! # TOML Date and Time Handling Example
//!
//! Comprehensive guide to working with all 4 TOML RFC 3339 datetime representations:
//! 1. Offset Date-Time (UTC 'Z' or numeric offset)
//! 2. Local Date-Time (calendar date and wall clock time)
//! 3. Local Date (calendar date only)
//! 4. Local Time (wall clock time with optional seconds)

use babbel_toml::{from_str, to_string_pretty, DatetimeKind, Node, TomlDatetime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TOML Date & Time Showcase ===\n");

    let datetime_source = r#"
# 1. Offset Date-Time
utc_zulu = 1979-05-27T07:32:00Z
pacific_offset = 1979-05-27T00:32:00-07:00
space_separated = 1979-05-27 07:32:00Z
with_fraction = 1979-05-27T00:32:00.999999-07:00

# 2. Local Date-Time
local_iso = 1979-05-27T07:32:00
local_space = 1979-05-27 07:32:00.999999

# 3. Local Date
birth_date = 1979-05-27

# 4. Local Time
exact_time = 07:32:00
nanosecond_time = 07:32:00.999999999
optional_seconds_time = 07:32
"#;

    println!("--- 1. Parsing TOML Datetime Variants ---");
    let doc = from_str(datetime_source)?;

    let fields = [
        ("utc_zulu", DatetimeKind::OffsetDateTime),
        ("pacific_offset", DatetimeKind::OffsetDateTime),
        ("space_separated", DatetimeKind::OffsetDateTime),
        ("with_fraction", DatetimeKind::OffsetDateTime),
        ("local_iso", DatetimeKind::LocalDateTime),
        ("local_space", DatetimeKind::LocalDateTime),
        ("birth_date", DatetimeKind::LocalDate),
        ("exact_time", DatetimeKind::LocalTime),
        ("nanosecond_time", DatetimeKind::LocalTime),
        ("optional_seconds_time", DatetimeKind::LocalTime),
    ];

    println!("{:<24} {:<18} {:<32}", "Field Key", "Kind", "Raw Formatted");
    println!("{:-<74}", "");

    for (key, expected_kind) in fields {
        if let Some(dt) = doc.get(key).and_then(|n| n.as_datetime()) {
            println!("{:<24} {:<18?} {:<32}", key, dt.kind, dt.raw);
            assert_eq!(dt.kind, expected_kind, "Mismatched kind for {}", key);
        } else {
            panic!("Field {} missing or not a datetime!", key);
        }
    }

    println!("\n--- 2. Programmatically Creating Datetimes ---");
    let mut schedule = Node::new_table();

    // Create a new event with Local Date and Local Time
    let event_date = TomlDatetime::parse("2026-09-09").expect("valid date");
    assert_eq!(event_date.kind, DatetimeKind::LocalDate);

    let event_start = TomlDatetime::parse("14:30:00").expect("valid time");
    assert_eq!(event_start.kind, DatetimeKind::LocalTime);

    let event_end_zulu = TomlDatetime::parse("2026-09-09T16:00:00Z").expect("valid offset dt");
    assert_eq!(event_end_zulu.kind, DatetimeKind::OffsetDateTime);

    schedule.insert("conference_date".to_string(), Node::Datetime(event_date));
    schedule.insert("start_time".to_string(), Node::Datetime(event_start));
    schedule.insert("end_timestamp".to_string(), Node::Datetime(event_end_zulu));

    println!("Constructed Schedule Document:\n");
    println!("{}", to_string_pretty(&schedule)?);

    println!("All date and time representations verified!");
    Ok(())
}
