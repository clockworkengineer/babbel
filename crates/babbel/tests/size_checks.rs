//! Memory footprint and struct size invariant assertions.
//!
//! Locks in memory compaction optimizations to ensure struct sizes
//! do not regress as new features are added.

use babbel::{bencode as bencode_lib, json as json_lib, xml as xml_lib, yaml as yaml_lib};

#[test]
fn test_core_value_memory_size() {
    use core::mem::size_of;
    assert_eq!(
        size_of::<babbel_core::Value>(),
        32,
        "babbel_core::Value must fit within 32 bytes (half a cache line)"
    );
}

#[test]
fn test_xml_node_kind_compacted_size() {
    use core::mem::size_of;
    assert!(
        size_of::<xml_lib::NodeKind>() <= 48,
        "xml_lib::NodeKind must be <= 48 bytes (shrunk from 72 bytes via boxing)"
    );
    assert!(
        size_of::<xml_lib::NodeData>() <= 88,
        "xml_lib::NodeData must be <= 88 bytes (shrunk from 112 bytes)"
    );
}

#[test]
fn test_yaml_node_memory_size() {
    use core::mem::size_of;
    assert!(
        size_of::<yaml_lib::Node>() <= 40,
        "yaml_lib::Node must be <= 40 bytes"
    );
}

#[test]
fn test_json_node_memory_size() {
    use core::mem::size_of;
    assert!(
        size_of::<json_lib::Node>() <= 56,
        "json_lib::Node must be <= 56 bytes"
    );
}

#[test]
fn test_bencode_node_memory_size() {
    use core::mem::size_of;
    assert!(
        size_of::<bencode_lib::Node>() <= 56,
        "bencode_lib::Node must be <= 56 bytes"
    );
}
