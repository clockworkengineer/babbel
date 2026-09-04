//! # Input / Output Module
//!
//! Subsystem providing stream reader [`XmlSource`], stringifier destination [`XmlDestination`], BOM encoding detection, and character utilities.

pub mod char_utils;
pub mod destination;
pub mod encoding;
pub mod source;

pub use char_utils::{is_valid_xml_char, is_xml_name_char, is_xml_name_start, is_xml_whitespace};
pub use destination::XmlDestination;
pub use encoding::Format;
pub use source::XmlSource;
