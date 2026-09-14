//! # KDL Parse and Serialize Example
//!
//! Demonstrates parsing a realistic KDL configuration (Zellij / Helix editor style),
//! traversing the resulting Babbel `Value` AST, serializing to compact and pretty KDL,
//! and verifying roundtrip consistency.

use babbel_core::Value;
use babbel_kdl::{from_str, to_string, to_string_pretty};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel KDL Parse & Serialize Example ===\n");

    let kdl_source = r##"
    // Zellij terminal multiplexer configuration
    layout {
        /* Top tab bar */
        pane size=1 borderless=true {
            plugin location="zellij:tab-bar"
        }

        // Main editing and execution split
        pane split_direction="vertical" {
            pane name="editor" size="70%"
            pane name="terminal" size="30%"
        }

        /* Bottom status bar */
        pane size=2 {
            plugin location="zellij:status-bar"
        }
    }

    // Keybindings configuration
    keybinds {
        normal {
            bind "Ctrl h" { MoveFocus "Left"; }
            bind "Ctrl l" { MoveFocus "Right"; }
        }
    }

    theme "catppuccin-mocha"
    default_shell "zsh"
    simplified_ui false
    max_panes 100
    "##;

    println!("--- 1. Input KDL Document ---");
    println!("{}\n", kdl_source.trim());

    // 1. Parse into Value AST
    println!("--- 2. Parsing into Babbel Value AST ---");
    let parsed: Value = from_str(kdl_source)?;
    println!("Parsed successfully into Value AST!");

    if let Some(theme) = parsed.get("theme").and_then(|v| v.as_str()) {
        println!("Theme:         {}", theme);
    }
    if let Some(shell) = parsed.get("default_shell").and_then(|v| v.as_str()) {
        println!("Default Shell: {}", shell);
    }
    if let Some(ui) = parsed.get("simplified_ui").and_then(|v| v.as_bool()) {
        println!("Simplified UI: {}", ui);
    }
    if let Some(max) = parsed.get("max_panes").and_then(|v| v.as_i64()) {
        println!("Max Panes:     {}", max);
    }
    if let Some(layout) = parsed.get("layout") {
        println!("Layout defined: Object with {} top-level entries", layout.as_object().map_or(0, |o| o.len()));
    }

    // 2. Serialize to compact KDL
    println!("\n--- 3. Serializing to Compact KDL ---");
    let compact = to_string(&parsed)?;
    println!("{}\n", compact.trim());

    // 3. Serialize to Pretty-printed KDL
    println!("--- 4. Serializing to Pretty-printed KDL ---");
    let pretty = to_string_pretty(&parsed, 2)?;
    println!("{}", pretty);

    // 4. Verify roundtrip fidelity
    let roundtrip = from_str(&compact)?;
    assert_eq!(roundtrip, parsed);
    println!("\nRoundtrip fidelity verified: Parsed AST exactly matches original.");

    Ok(())
}
