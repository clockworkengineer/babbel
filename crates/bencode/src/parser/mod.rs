/// Module implementing the default recursive bencode parser.
/// Provides standard bencode parsing functionality using recursive descent approach.
pub mod default;

/// Zero-copy borrowed parser for embedded systems
pub mod borrowed;

/// Iterative (stack-based) parser for deeply nested structures
/// Avoids recursion to prevent stack overflow on embedded systems
pub mod iterative;
