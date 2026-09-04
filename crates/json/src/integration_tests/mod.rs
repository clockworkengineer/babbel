//! Integration tests module
//!
//! Contains integration tests that verify the library works correctly with actual files
//! and across module boundaries.

#[cfg(feature = "file-io")]
mod file_integration_tests;

#[cfg(feature = "file-io")]
mod parser_integration_tests;

#[cfg(feature = "file-io")]
mod round_trip_integration_tests;

#[cfg(feature = "file-io")]
mod validate_integration_tests;
