//! # Parser Subsystem
//!
//! Exposes the [`XmlParser`] recursive descent tokenizer and DOM tree constructor.

pub mod pull_parser;
pub mod xml_parser;

pub use pull_parser::{XmlPullAttribute, XmlPullEvent, XmlPullParser};
pub use xml_parser::XmlParser;
