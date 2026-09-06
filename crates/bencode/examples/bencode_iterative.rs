//! Iterative (stack-based) parser example
//!
//! **Best Practice:** Use `parse_iterative` and related functions for deeply nested bencode data or in embedded/stack-limited environments.
//! This avoids stack overflows by using an explicit heap stack, making parsing robust and predictable for any nesting depth.
//!
//! Demonstrates the iterative parser that avoids recursion, making it suitable
//! for embedded systems with limited stack space or handling deeply nested structures.
//!
//! The iterative parser uses an explicit heap-allocated stack instead of the call stack,
//! preventing stack overflow when parsing deeply nested bencode structures.

use bencode_lib::{parse_iterative, stringify_to_bytes, BufferSource};

fn main() {
    println!("=== Iterative Parser Demo ===\n");

    // Example 1: Simple parsing comparison
    println!("1. Simple structure:");
    let simple = b"d3:agei25e4:name4:Johne";

    let mut source = BufferSource::new(simple);
    match parse_iterative(&mut source) {
        Ok(node) => {
            println!("   Parsed successfully with iterative parser");
            if let Ok(output) = stringify_to_bytes(&node) {
                println!("   Output: {}", String::from_utf8_lossy(&output));
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 2: Deeply nested lists
    println!("\n2. Deeply nested structure (100 levels):");
    let mut deeply_nested = String::from("i42e");
    for _ in 0..100 {
        deeply_nested = format!("l{}e", deeply_nested);
    }
    println!("   Created nested list with {} bytes", deeply_nested.len());

    // Try with iterative parser (should work)
    match bencode_lib::parse_bytes_iterative(deeply_nested.as_bytes()) {
        Ok(_) => println!("   ✓ Iterative parser succeeded"),
        Err(e) => println!("   ✗ Iterative parser failed: {}", e),
    }

    // Try with recursive parser (works but uses more stack)
    match bencode_lib::parse_bytes(deeply_nested.as_bytes()) {
        Ok(_) => println!("   ✓ Recursive parser succeeded (but used more stack)"),
        Err(e) => println!("   ✗ Recursive parser failed: {}", e),
    }

    // Example 3: Very deeply nested (1000 levels) - only iterative should handle well
    println!("\n3. Very deeply nested structure (1000 levels):");
    let mut very_deep = String::from("i1e");
    for _ in 0..1000 {
        very_deep = format!("l{}e", very_deep);
    }
    println!("   Created nested list with {} bytes", very_deep.len());

    match bencode_lib::parse_bytes_iterative(very_deep.as_bytes()) {
        Ok(_) => println!("   ✓ Iterative parser succeeded (no stack issues)"),
        Err(e) => println!("   ✗ Iterative parser failed: {}", e),
    }

    // Example 4: Complex nested dictionary
    println!("\n4. Complex nested dictionary:");
    let complex = b"d4:infod6:lengthi123e4:name9:test.file12:piece lengthi16384eee";

    let mut source = BufferSource::new(complex);
    match parse_iterative(&mut source) {
        Ok(node) => {
            println!("   Parsed complex structure successfully");
            if let Ok(output) = stringify_to_bytes(&node) {
                println!("   Size: {} bytes", output.len());
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 5: Deeply nested mixed structure
    println!("\n5. Mixed nested structure:");
    let mixed = b"d4:dictd1:ai1e1:bi2ee4:listlli1ei2eeli3ei4eeee";

    let mut source = BufferSource::new(mixed);
    match parse_iterative(&mut source) {
        Ok(node) => {
            println!("   Parsed mixed nested structure");
            if let Ok(output) = stringify_to_bytes(&node) {
                println!("   Output: {}", String::from_utf8_lossy(&output));
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n=== Advantages of Iterative Parser ===");
    println!("✓ No recursion - uses explicit stack on heap");
    println!("✓ Safe for deeply nested structures (>1000 levels)");
    println!("✓ Predictable memory usage");
    println!("✓ Ideal for embedded systems with limited stack space");
    println!("✓ Same functionality as recursive parser");
}
