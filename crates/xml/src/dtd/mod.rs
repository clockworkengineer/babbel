//! # DTD Subsystem
//!
//! Subsystem for DTD subset parsing, content model checking, and required attribute validation.

pub mod validator;

pub use validator::{
    ContentModel, DtdAttributeRule, DtdElementRule, DtdValidator, ExternalSubsetResolver,
};
