/// Error message for unexpected character encountered during parsing
pub const ERR_UNEXPECTED_CHAR: &str = "Unexpected character: ";

/// Error message when input is empty
pub const ERR_EMPTY_INPUT: &str = "Empty input";

/// Error message when an object key is not a string
pub const ERR_OBJECT_KEY: &str = "Object key must be a string";

/// Error message when colon is missing after an object key
pub const ERR_EXPECT_COLON: &str = "Expected ':' after object key";

/// Error message when an object is not properly terminated
pub const ERR_EXPECT_OBJECT_END: &str = "Expected ',' or '}' in object";

/// Error message when an array is not properly terminated
pub const ERR_EXPECT_ARRAY_END: &str = "Expected ',' or ']' in array";

/// Error message for an invalid escape sequence in string
pub const ERR_INVALID_ESCAPE: &str = "Invalid escape sequence";

/// Error message for unterminated string
pub const ERR_UNTERMINATED_STRING: &str = "Unterminated string";

/// Error message when a number has multiple decimal points
pub const ERR_MULTIPLE_DECIMAL: &str = "Multiple decimal points in number";

/// Error message for invalid float number format
pub const ERR_INVALID_FLOAT: &str = "Invalid float number";

/// Error message for invalid integer number format
pub const ERR_INVALID_INTEGER: &str = "Invalid integer number";

/// Error message when true literal is expected but not found
pub const ERR_EXPECT_TRUE: &str = "Expected 'true'";

/// Error message when a false literal is expected but not found
pub const ERR_EXPECT_FALSE: &str = "Expected 'false'";

/// Error message when null literal is expected but not found
pub const ERR_EXPECT_NULL: &str = "Expected 'null'";