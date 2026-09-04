//! # Entity Subsystem
//!
//! Handles XML predefined entities, numeric references, custom entity tables, and XXE security policies.

pub mod mapper;
pub mod resolver;

pub use mapper::EntityMapper;
pub use resolver::{DefaultEntityResolver, EntityResolver};
