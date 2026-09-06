//! Example demonstrating memory management utilities for embedded systems.
//!
//! This example shows how to use MemoryTracker, Arena, and StackBuffer
//! to manage memory in resource-constrained environments.

use bencode_lib::{parse_borrowed, Arena, MemoryTracker, StackBuffer};

fn main() {
    println!("=== Memory Management for Embedded Systems ===\n");

    // Example 1: Memory Tracking
    println!("1. Memory Usage Tracking");
    println!("   Track allocations to monitor memory consumption\n");

    let tracker = MemoryTracker::with_limit(1024);

    println!("   Initial state:");
    println!("   - Current: {} bytes", tracker.current());
    println!("   - Peak: {} bytes", tracker.peak());
    println!("   - Limit: {} bytes", tracker.limit());

    // Simulate allocations
    tracker.allocate(256).unwrap();
    println!("\n   After allocating 256 bytes:");
    println!("   - Current: {} bytes", tracker.current());
    println!("   - Peak: {} bytes", tracker.peak());

    tracker.allocate(512).unwrap();
    println!("\n   After allocating 512 more bytes:");
    println!("   - Current: {} bytes", tracker.current());
    println!("   - Peak: {} bytes", tracker.peak());

    tracker.deallocate(256);
    println!("\n   After deallocating 256 bytes:");
    println!(
        "   - Current: {} bytes (freed some memory)",
        tracker.current()
    );
    println!("   - Peak: {} bytes (peak remains high)", tracker.peak());

    // Try to exceed limit
    match tracker.allocate(600) {
        Ok(_) => println!("\n   ✓ Allocated 600 more bytes"),
        Err(e) => println!("\n   ✗ Cannot allocate 600 bytes: {}", e),
    }

    println!("\n   This helps prevent out-of-memory errors in embedded systems!\n");

    // Example 2: Arena Allocator
    println!("2. Arena (Bump) Allocator");
    println!("   Fast allocation from a fixed buffer\n");

    let arena = Arena::with_capacity(4096);

    println!("   Arena capacity: {} bytes", arena.capacity());
    println!("   Arena used: {} bytes", arena.used());
    println!("   Arena remaining: {} bytes", arena.remaining());

    // Allocate some buffers
    if let Some(buffer1) = arena.alloc_bytes(128) {
        buffer1[0] = b'A';
        println!("\n   ✓ Allocated 128-byte buffer");
        println!("   - Used: {} bytes", arena.used());
        println!("   - Remaining: {} bytes", arena.remaining());
    }

    if let Some(buffer2) = arena.alloc_bytes(256) {
        buffer2[0] = b'B';
        println!("\n   ✓ Allocated 256-byte buffer");
        println!("   - Used: {} bytes", arena.used());
        println!("   - Remaining: {} bytes", arena.remaining());
    }

    // Reset the arena to reuse memory
    unsafe {
        arena.reset();
    }
    println!("\n   Arena reset - all memory available again");
    println!("   - Used: {} bytes", arena.used());
    println!("   - Remaining: {} bytes", arena.remaining());

    println!("\n   Perfect for temporary allocations that can be batch-freed!\n");

    // Example 3: Stack Buffer
    println!("3. Stack-Based Fixed-Size Buffer");
    println!("   Zero heap allocation - everything on the stack\n");

    let mut stack_buf = StackBuffer::<256>::new();

    println!("   Stack buffer capacity: {} bytes", stack_buf.capacity());
    println!("   Stack buffer length: {} bytes", stack_buf.len());

    // Add some data
    stack_buf.push(b'H');
    stack_buf.extend_from_slice(b"ello, embedded world!");

    println!("\n   Added data to buffer:");
    println!("   - Length: {} bytes", stack_buf.len());
    println!(
        "   - Data: {:?}",
        core::str::from_utf8(stack_buf.as_slice()).unwrap()
    );

    // Try to overflow
    let large_data = &[b'X'; 300];
    if !stack_buf.extend_from_slice(large_data) {
        println!(
            "\n   ✗ Cannot add 300 bytes - buffer only has {} bytes free",
            stack_buf.capacity() - stack_buf.len()
        );
    }

    println!("\n   Stack buffers prevent unbounded memory growth!\n");

    // Example 4: Parsing with Memory Budget
    println!("4. Zero-Copy Parsing with Memory Awareness");
    println!("   Parse bencode data with known memory limits\n");

    // Simple bencode data
    let data1 = b"d4:name4:test4:sizei1024ee";

    println!("   Input size: {} bytes", data1.len());
    println!("   Parsing with zero-copy (no string allocation)...");

    match parse_borrowed(data1) {
        Ok(node) => {
            println!("   ✓ Parsed successfully");
            println!(
                "   - Memory used: ~{} bytes (only Vec/HashMap overhead)",
                core::mem::size_of_val(&node)
            );
            println!("   - String data: borrowed from input (0 bytes allocated)");
            println!("   - Result: {}", node);
        }
        Err(e) => println!("   ✗ Parse error: {}", e),
    }

    // Example 5: Stack-Only Parsing
    println!("\n5. Stack-Only Parsing Example");
    println!("   Parse small bencode without any heap allocation\n");

    // Small bencode that fits in stack buffer
    let small_data = b"i42e";

    // Copy to stack buffer
    if let Some(stack_data) = StackBuffer::<64>::from_slice(small_data) {
        println!("   Data copied to 64-byte stack buffer");
        println!(
            "   Buffer usage: {}/{} bytes",
            stack_data.len(),
            stack_data.capacity()
        );

        // Parse from stack buffer
        match parse_borrowed(stack_data.as_slice()) {
            Ok(node) => {
                println!("   ✓ Parsed from stack buffer");
                println!("   - Result: {}", node);
                if let Some(val) = node.as_integer() {
                    println!("   - Value: {}", val);
                }
            }
            Err(e) => println!("   ✗ Parse error: {}", e),
        }
    }

    // Example 6: Memory Budget Comparison
    println!("\n6. Memory Budget Comparison\n");

    let test_data = b"li1ei2ei3e5:hello5:worlde";

    println!("   Input: {:?}", core::str::from_utf8(test_data).unwrap());
    println!("   Size: {} bytes", test_data.len());

    // Standard parsing (allocates strings)
    println!("\n   Standard parse() method:");
    println!("   - Allocates String for each byte string");
    println!(
        "   - Estimated memory: {} bytes (Vec + String overhead)",
        test_data.len() + 200
    ); // Rough estimate

    // Zero-copy parsing
    println!("\n   Zero-copy parse_borrowed() method:");
    println!("   - Borrows from input buffer");
    println!("   - Estimated memory: ~100 bytes (only Vec/HashMap structure)");
    println!(
        "   - Memory savings: ~{}%",
        ((test_data.len() + 200 - 100) * 100) / (test_data.len() + 200)
    );

    // Practical Recommendations
    println!("\n=== Recommendations for Embedded Systems ===\n");
    println!("1. Use parse_borrowed() instead of parse()");
    println!("   → Eliminates string allocation overhead");
    println!("");
    println!("2. Use MemoryTracker to enforce budgets");
    println!("   → Prevent out-of-memory errors");
    println!("");
    println!("3. Use Arena for temporary allocations");
    println!("   → Reduce fragmentation, batch free");
    println!("");
    println!("4. Use StackBuffer for small, bounded data");
    println!("   → Zero heap usage for predictable sizes");
    println!("");
    println!("5. Use validate_bencode() before parsing");
    println!("   → Check validity without committing memory");
    println!("");
    println!("6. Combine techniques for maximum efficiency");
    println!("   → Stack buffer + zero-copy + validation = minimal footprint");
}
