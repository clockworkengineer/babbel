//! # XSD Subsystem
//!
//! Handles XSD schema document parsing and DOM structure/type validation.

pub mod validator;

pub use validator::{
    Compositor, XsdAttributeRule, XsdComplexType, XsdElementRule, XsdRestriction, XsdValidator,
};
