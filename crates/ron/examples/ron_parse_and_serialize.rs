//! # RON Parse and Serialize Example
//!
//! Demonstrates constructing Babbel `Value` AST, serializing to RON,
//! parsing game/Rust-style configuration with comments and raw strings,
//! and verifying roundtrip fidelity.

use babbel_core::Value;
use babbel_ron::{from_str, to_string, to_string_pretty};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel RON Parse & Serialize Example ===\n");

    let ron_source = r##"
    // Bevy Engine game graphics & audio settings
    GameSettings(
        /* Display configuration */
        resolution: (width: 2560, height: 1440),
        fullscreen: true,
        vsync: false,
        max_fps: Some(165),

        // Graphics quality flags
        shadow_map_size: 0x800, // 2048 in hex
        anti_aliasing: "MSAA4x",
        render_scale: 1.0,

        /* Audio levels */
        volumes: (
            master: 0.85,
            music: 0.70,
            sfx: 1.0,
        ),

        // Asset search paths with raw strings
        search_paths: [
            r#"assets/shaders"#,
            r#"assets/textures"#,
        ],
    )
    "##;

    println!("--- 1. Input RON Document ---");
    println!("{}\n", ron_source.trim());

    // 1. Parse into Value AST
    println!("--- 2. Parsing into Babbel Value AST ---");
    let parsed: Value = from_str(ron_source)?;
    println!("Parsed successfully into Value AST!");

    if let Some(fs) = parsed.get("fullscreen").and_then(|v| v.as_bool()) {
        println!("Fullscreen:      {}", fs);
    }
    if let Some(fps) = parsed.get("max_fps").and_then(|v| v.as_i64()) {
        println!("Max FPS:         {}", fps);
    }
    if let Some(shadow) = parsed.get("shadow_map_size").and_then(|v| v.as_i64()) {
        println!("Shadow Map Size: {} (from 0x800)", shadow);
    }
    if let Some(res) = parsed.get("resolution") {
        let w = res.get("width").and_then(|v| v.as_i64()).unwrap_or(0);
        let h = res.get("height").and_then(|v| v.as_i64()).unwrap_or(0);
        println!("Resolution:      {}x{}", w, h);
    }
    if let Some(paths) = parsed.get("search_paths").and_then(|v| v.as_array()) {
        println!("Search Paths ({}):", paths.len());
        for p in paths {
            if let Some(s) = p.as_str() {
                println!("  - {}", s);
            }
        }
    }

    // 2. Serialize to compact RON
    println!("\n--- 3. Serializing to Compact RON ---");
    let compact = to_string(&parsed)?;
    println!("{}\n", compact);

    // 3. Serialize to Pretty-printed RON
    println!("--- 4. Serializing to Pretty-printed RON ---");
    let pretty = to_string_pretty(&parsed, 2)?;
    println!("{}", pretty);

    // 4. Verify roundtrip equality
    let roundtrip = from_str(&compact)?;
    assert_eq!(roundtrip, parsed);
    println!("\nRoundtrip fidelity verified: Parsed AST exactly matches original.");

    Ok(())
}
