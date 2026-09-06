//! Example demonstrating const generics for compile-time memory bounds.
//!
//! **Best Practice:** Use `FixedSizeBuffer<N>` for stack-allocated, compile-time checked buffers in embedded or size-sensitive applications.
//! This ensures predictable memory usage, avoids heap allocation, and enables compile-time safety checks.
//!
//! This example shows how to use const generics to ensure memory usage
//! is known at compile time, perfect for embedded systems.

use bencode_lib::{parse_borrowed, validate_bencode, FixedSizeBuffer, MemoryBounds};

fn main() {
    println!("=== Const Generics for Embedded Systems ===\n");

    // Example 1: Compile-Time Buffer Sizing
    println!("1. Compile-Time Fixed-Size Buffers\n");

    // Define buffer size at compile time
    const BUFFER_SIZE: usize = 256;
    const STACK_BYTES: usize = MemoryBounds::stack_buffer_size(BUFFER_SIZE);

    println!("   Buffer capacity: {} bytes", BUFFER_SIZE);
    println!(
        "   Stack memory used: {} bytes (known at compile time)",
        STACK_BYTES
    );

    // Create a stack-allocated buffer with compile-time size
    let mut buffer = FixedSizeBuffer::<BUFFER_SIZE>::new();

    // Add some bencode data
    let data = b"d4:name4:test4:sizei1024ee";
    if buffer.extend_from_slice(data) {
        println!("   ✓ Data fits in buffer ({} bytes used)\n", buffer.len());
    } else {
        println!("   ✗ Data too large for buffer\n");
    }

    // Example 2: Memory Estimation
    println!("2. Compile-Time Memory Estimation\n");

    // Estimate memory for a structure at compile time
    const NUM_NODES: usize = 20;
    const NUM_CONTAINERS: usize = 3;
    const AVG_SIZE: usize = 5;
    const ESTIMATED_HEAP: usize =
        MemoryBounds::borrowed_parse_estimate(NUM_NODES, NUM_CONTAINERS, AVG_SIZE);

    println!("   Structure complexity:");
    println!("   - {} total nodes", NUM_NODES);
    println!("   - {} containers (lists/dicts)", NUM_CONTAINERS);
    println!("   - {} avg items per container", AVG_SIZE);
    println!(
        "   Estimated heap usage: {} bytes (compile-time calculation)\n",
        ESTIMATED_HEAP
    );

    // Example 3: Stack Safety Calculation
    println!("3. Maximum Safe Nesting Depth\n");

    // Calculate max depth for different stack sizes
    const SMALL_STACK: usize = 4096; // 4KB - typical embedded
    const MEDIUM_STACK: usize = 16384; // 16KB
    const FRAME_SIZE: usize = 128; // Typical stack frame

    const MAX_DEPTH_SMALL: usize = MemoryBounds::max_safe_depth(SMALL_STACK, FRAME_SIZE);
    const MAX_DEPTH_MEDIUM: usize = MemoryBounds::max_safe_depth(MEDIUM_STACK, FRAME_SIZE);

    println!(
        "   Stack size: {} bytes → Max depth: {}",
        SMALL_STACK, MAX_DEPTH_SMALL
    );
    println!(
        "   Stack size: {} bytes → Max depth: {}",
        MEDIUM_STACK, MAX_DEPTH_MEDIUM
    );
    println!("   (Calculated at compile time with 50% safety margin)\n");

    // Example 4: Type-Safe Buffer Sizes
    println!("4. Type-Safe Fixed-Size Buffers\n");

    // Different buffer sizes for different purposes
    type SmallBuffer = FixedSizeBuffer<64>; // For small messages
    type MediumBuffer = FixedSizeBuffer<256>; // For typical data
    type LargeBuffer = FixedSizeBuffer<1024>; // For large structures

    println!(
        "   SmallBuffer:  {} bytes (stack)",
        core::mem::size_of::<SmallBuffer>()
    );
    println!(
        "   MediumBuffer: {} bytes (stack)",
        core::mem::size_of::<MediumBuffer>()
    );
    println!(
        "   LargeBuffer:  {} bytes (stack)",
        core::mem::size_of::<LargeBuffer>()
    );
    println!("   All sizes known at compile time!\n");

    // Example 5: Practical Usage
    println!("5. Practical Example: Stack-Only Parsing\n");

    // Use a fixed-size buffer with const generic
    let mut parse_buffer = FixedSizeBuffer::<128>::new();
    let bencode_data = b"li1ei2ei3e5:helloe";

    if parse_buffer.extend_from_slice(bencode_data) {
        println!("   ✓ Data loaded into 128-byte stack buffer");
        println!(
            "   Buffer usage: {}/{} bytes",
            parse_buffer.len(),
            parse_buffer.capacity()
        );

        // Validate without parsing
        match validate_bencode(parse_buffer.as_slice()) {
            Ok(_) => println!("   ✓ Data validated (minimal memory)"),
            Err(e) => println!("   ✗ Invalid: {}", e),
        }

        // Parse with zero-copy
        match parse_borrowed(parse_buffer.as_slice()) {
            Ok(node) => {
                println!("   ✓ Parsed with zero-copy");
                println!("   Result: {}", node);

                if let Some(list) = node.as_list() {
                    println!("   List contains {} items", list.len());
                }
            }
            Err(e) => println!("   ✗ Parse error: {}", e),
        }
    }

    println!("\n   Total memory used:");
    println!(
        "   - Stack: {} bytes (FixedSizeBuffer)",
        core::mem::size_of::<FixedSizeBuffer<128>>()
    );
    println!("   - Heap: ~100-200 bytes (BorrowedNode structure only)");
    println!("   - String data: 0 bytes (borrowed from stack buffer!)");

    // Example 6: Compile-Time Assertions
    println!("\n6. Compile-Time Safety Checks\n");

    // These assertions are checked at compile time
    const MIN_BUFFER: usize = 64;
    const ACTUAL_BUFFER: usize = 128;

    // This compiles because ACTUAL_BUFFER >= MIN_BUFFER
    bencode_lib::assert_buffer_size!(ACTUAL_BUFFER, MIN_BUFFER);

    println!("   ✓ Buffer size assertion passed at compile time");
    println!(
        "   Required: {} bytes, Provided: {} bytes\n",
        MIN_BUFFER, ACTUAL_BUFFER
    );

    // Example 7: Memory Budget Planning
    println!("7. Memory Budget Planning\n");

    // Plan memory usage for a specific embedded system
    const TARGET_RAM: usize = 8192; // 8KB total RAM
    const RESERVED_STACK: usize = 2048; // 2KB for stack
    const AVAILABLE_HEAP: usize = TARGET_RAM - RESERVED_STACK; // 6KB for heap

    println!("   Target System:");
    println!("   - Total RAM: {} bytes", TARGET_RAM);
    println!("   - Stack: {} bytes", RESERVED_STACK);
    println!("   - Available heap: {} bytes", AVAILABLE_HEAP);

    // Calculate what can fit
    const NUM_PARSEABLE_NODES: usize = AVAILABLE_HEAP / 100; // ~100 bytes per node

    println!(
        "\n   With {} bytes heap, can parse ~{} nodes",
        AVAILABLE_HEAP, NUM_PARSEABLE_NODES
    );
    println!("   (All calculated at compile time!)\n");

    // Summary
    println!("=== Benefits of Const Generics for Embedded ===\n");
    println!("✓ Buffer sizes known at compile time");
    println!("✓ Memory usage predictable and bounded");
    println!("✓ Type safety for different buffer sizes");
    println!("✓ Compile-time assertions prevent runtime errors");
    println!("✓ Zero runtime overhead for size checks");
    println!("✓ Perfect for safety-critical systems");
    println!("\nCombine FixedSizeBuffer<N> + parse_borrowed() for maximum efficiency!");
}
