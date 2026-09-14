//! # JSON5 and JSONC Features Example
//!
//! Demonstrates parsing relaxed human-centric JSON5 / JSONC configurations:
//! - Single-line (`//`) and multi-line (`/* */`) comments
//! - Unquoted ECMAScript 5.1 identifier keys
//! - Single-quoted string literals and multi-line strings
//! - Trailing commas in arrays and objects
//! - Hexadecimal integers (`0x...`), leading/trailing decimal points (`.5`, `5.`), explicit signs (`+42`)

use babbel_json::parser::json5::parse_json5;
use babbel_json::Json5Engine;
use babbel_core::{FormatEngine, FormatOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel JSON5 & JSONC Features Example ===\n");

    let json5_text = r#"
    // Visual Studio Code / TypeScript style configuration
    {
        /* Core build options */
        target: 'ES2024',
        moduleResolution: 'bundler',
        strict: true,
        declaration: false,

        // Resource limits & timeouts
        maxRetries: 5,
        memoryLimitMb: 0x400, // 1024 MB in hex
        threshold: .75,        // leading decimal point
        scaleFactor: 2.,       // trailing decimal point
        priorityOffset: +10,   // explicit plus sign

        /* Output destinations */
        outDir: './dist',
        plugins: [
            'esbuild-plugin-rust',
            'esbuild-plugin-wasm', // trailing comma in array!
        ], // trailing comma in object!
    }
    "#;

    println!("--- 1. Input JSON5 document ---");
    println!("{}\n", json5_text.trim());

    // 1. Parse with parse_json5 into Babbel Node
    println!("--- 2. Parsing into Babbel Node ---");
    let node = parse_json5(json5_text)?;
    println!("Parsed successfully into Node tree!");

    if let Some(target) = node["target"].as_str() {
        println!("Target:          {}", target);
    }
    if let Some(mem) = node["memoryLimitMb"].as_i64() {
        println!("Memory Limit:    {} MB (from 0x400)", mem);
    }
    if let Some(thresh) = node["threshold"].as_f64() {
        println!("Threshold:       {} (from .75)", thresh);
    }
    if let Some(scale) = node["scaleFactor"].as_f64() {
        println!("Scale Factor:    {} (from 2.)", scale);
    }
    if let Some(prio) = node["priorityOffset"].as_i64() {
        println!("Priority Offset: +{}", prio);
    }
    if let Some(plugins) = node["plugins"].as_array() {
        println!("Plugins ({}):", plugins.len());
        for (i, p) in plugins.iter().enumerate() {
            if let Some(name) = p.as_str() {
                println!("  [{}] {}", i, name);
            }
        }
    }

    // 2. Use Json5Engine via FormatEngine trait
    println!("\n--- 3. Using Json5Engine via FormatEngine trait ---");
    let engine = Json5Engine::default();
    println!("Engine format ID:   {}", engine.format_id());
    println!("File extensions:    {:?}", engine.file_extensions());
    println!("MIME type:          {}", engine.mime_type());

    let value = engine.parse_str(json5_text)?;
    let serialized_standard_json = engine.serialize_to_string(&value, &FormatOptions::default())?;
    println!("\nConverted to standard strict JSON:");
    println!("{}", serialized_standard_json);

    println!("\nJSON5 parsing and FormatEngine execution completed successfully!");
    Ok(())
}
