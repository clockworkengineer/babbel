//! # TOML Fibonacci Sequence Generator & Storage
//!
//! Maintains a persistent Fibonacci sequence in a TOML file.
//! Demonstrates file reading (`FileSource`), DOM parsing, array manipulation,
//! checked arithmetic, and file writing (`FileDestination`).

use std::path::Path;
use babbel_toml::{from_str, to_string_pretty, DatetimeKind, FileDestination, FileSource, Node, TomlDatetime};

/// Reads a Fibonacci document from a TOML file or initializes a default sequence.
fn read_or_init_sequence(path: &Path) -> Result<Node, Box<dyn std::error::Error>> {
    if !path.exists() {
        println!("Sequence file not found. Initializing new Fibonacci document...");
        let mut root = Node::new_table();

        let mut meta = Node::new_table();
        meta.insert("name".to_string(), Node::from("Fibonacci Sequence"));
        meta.insert("algorithm".to_string(), Node::from("Babbel Checked Arithmetic"));
        meta.insert("created_at".to_string(), Node::Datetime(TomlDatetime::new(
            DatetimeKind::OffsetDateTime,
            "2026-09-09T15:00:00Z",
        )));
        root.insert("metadata".to_string(), meta);

        let mut numbers = Node::new_array();
        numbers.push(Node::from(1i64));
        numbers.push(Node::from(1i64));
        root.insert("numbers".to_string(), numbers);

        return Ok(root);
    }

    let mut source = FileSource::new(path)?;
    let mut content = String::new();
    while source.more() {
        if let Some(ch) = source.current() {
            content.push(ch);
        }
        source.next();
    }
    let node = from_str(&content)?;
    Ok(node)
}

/// Appends the next Fibonacci number by summing the last two numbers using checked addition.
fn append_next_fibonacci(doc: &mut Node) -> Result<i64, String> {
    let (prev, last) = match doc.get("numbers") {
        Some(Node::Array(arr)) if arr.len() >= 2 => {
            let p = arr[arr.len() - 2].as_i64().ok_or("Invalid number type")?;
            let l = arr[arr.len() - 1].as_i64().ok_or("Invalid number type")?;
            (p, l)
        }
        _ => return Err("Expected at least 2 numbers in array".to_string()),
    };

    let next = prev
        .checked_add(last)
        .ok_or_else(|| "Integer overflow generating next Fibonacci number".to_string())?;

    let new_len = {
        let numbers = doc
            .get_mut("numbers")
            .ok_or_else(|| "Missing 'numbers' array".to_string())?;
        numbers.push(Node::from(next));
        numbers.len() as i64
    };

    // Update metadata if present
    if let Some(meta) = doc.get_mut("metadata") {
        meta.insert("count".to_string(), Node::from(new_len));
    }

    Ok(next)
}

/// Saves the Fibonacci document to a TOML file.
fn save_sequence(path: &Path, doc: &Node) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = to_string_pretty(doc)?;
    let mut dest = FileDestination::new(path)?;
    dest.add_bytes(&serialized);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TOML Fibonacci Generator Example ===\n");

    let temp_file = Path::new("target/fibonacci_demo.toml");

    // Clean up previous run if any
    if temp_file.exists() {
        let _ = std::fs::remove_file(temp_file);
    }

    let mut doc = read_or_init_sequence(temp_file)?;

    // Generate next 10 numbers
    println!("Generating next Fibonacci numbers:");
    for step in 1..=10 {
        let next_val = append_next_fibonacci(&mut doc)?;
        println!("  Step {:>2}: generated {}", step, next_val);
    }

    // Save to disk
    save_sequence(temp_file, &doc)?;
    println!("\nSaved sequence to {}", temp_file.display());

    // Read back and inspect
    let reloaded = read_or_init_sequence(temp_file)?;
    if let Some(numbers) = reloaded.get("numbers").and_then(|n| n.as_array()) {
        println!("Total sequence length: {}", numbers.len());
        let vals: Vec<String> = numbers.iter().filter_map(|n| n.as_i64().map(|i| i.to_string())).collect();
        println!("Sequence: [{}]", vals.join(", "));
    }

    // Clean up temp file
    let _ = std::fs::remove_file(temp_file);
    println!("\nCleaned up temporary sequence file.");

    Ok(())
}
