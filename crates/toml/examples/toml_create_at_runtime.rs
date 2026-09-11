//! # TOML Create At Runtime Example
//!
//! Demonstrates constructing a complete TOML document dynamically at runtime
//! using `Node::new_table()`, `Node::new_array()`, nested tables, and array-of-tables.

use babbel_toml::{to_string_pretty, DatetimeKind, Node, TomlDatetime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TOML Runtime DOM Creation Example ===\n");

    // 1. Root document table
    let mut root = Node::new_table();

    // Top-level metadata
    root.insert("project".to_string(), Node::from("Babbel"));
    root.insert("version".to_string(), Node::from("0.2.0"));
    root.insert("release_channel".to_string(), Node::from("stable"));
    root.insert("uptime_hours".to_string(), Node::from(144));
    root.insert("load_average".to_string(), Node::from(0.42));
    root.insert("is_production".to_string(), Node::from(true));

    // Release timestamp
    let release_dt = TomlDatetime::parse("2026-09-09T15:00:00+01:00").expect("valid datetime");
    assert_eq!(release_dt.kind, DatetimeKind::OffsetDateTime);
    root.insert("released_at".to_string(), Node::Datetime(release_dt));

    // 2. Nested table: [author]
    let mut author_table = Node::new_table();
    author_table.insert("name".to_string(), Node::from("ClockworkEngineer"));
    author_table.insert("email".to_string(), Node::from("dev@babbel.io"));
    author_table.insert("active".to_string(), Node::from(true));
    root.insert("author".to_string(), author_table);

    // 3. Nested table with array: [features]
    let mut features_table = Node::new_table();
    let mut supported_formats = Node::new_array();
    supported_formats.push(Node::from("json"));
    supported_formats.push(Node::from("yaml"));
    supported_formats.push(Node::from("xml"));
    supported_formats.push(Node::from("bencode"));
    supported_formats.push(Node::from("toml"));
    features_table.insert("formats".to_string(), supported_formats);
    features_table.insert("zero_copy".to_string(), Node::from(true));
    features_table.insert("no_std_support".to_string(), Node::from(true));
    root.insert("features".to_string(), features_table);

    // 4. Array of tables: [[services]]
    let mut services_array = Node::new_array();

    // Service 1: API Gateway
    let mut svc_api = Node::new_table();
    svc_api.insert("name".to_string(), Node::from("api-gateway"));
    svc_api.insert("host".to_string(), Node::from("10.0.1.10"));
    svc_api.insert("port".to_string(), Node::from(443));
    svc_api.insert("protocols".to_string(), {
        let mut p = Node::new_array();
        p.push(Node::from("https"));
        p.push(Node::from("http2"));
        p
    });
    services_array.push(svc_api);

    // Service 2: Worker Daemon
    let mut svc_worker = Node::new_table();
    svc_worker.insert("name".to_string(), Node::from("queue-worker"));
    svc_worker.insert("host".to_string(), Node::from("10.0.1.20"));
    svc_worker.insert("port".to_string(), Node::from(9000));
    svc_worker.insert("concurrency".to_string(), Node::from(16));
    services_array.push(svc_worker);

    // Service 3: Cache Node
    let mut svc_cache = Node::new_table();
    svc_cache.insert("name".to_string(), Node::from("cache-cluster"));
    svc_cache.insert("host".to_string(), Node::from("10.0.1.30"));
    svc_cache.insert("port".to_string(), Node::from(6379));
    svc_cache.insert("max_memory_mb".to_string(), Node::from(2048));
    services_array.push(svc_cache);

    root.insert("services".to_string(), services_array);

    // 5. Serialize the constructed DOM
    println!("Constructed DOM Document:\n");
    let output = to_string_pretty(&root)?;
    println!("{}", output);

    Ok(())
}
