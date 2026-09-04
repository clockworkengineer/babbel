//! # XML Namespaces 1.0 Example
//!
//! Demonstrates parsing, inspecting, and resolving W3C Namespaces in XML documents:
//! - Extracting element prefixes and local names from QNames
//! - Resolving namespace URIs based on hierarchical scope
//! - Finding elements using namespace-aware query selectors (`get_elements_by_tag_name_ns`)

use xml_lib_rust::parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- XML Namespaces 1.0 Example ---");

    let xml = r#"<?xml version="1.0"?>
    <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                   xmlns:m="http://example.org/math">
      <soap:Header/>
      <soap:Body>
        <m:Add>
          <m:x>10</m:x>
          <m:y>20</m:y>
        </m:Add>
      </soap:Body>
    </soap:Envelope>"#;

    let doc = parse(xml)?;
    let root_id = doc.root_element_id().expect("Root element");

    // 1. Inspect Root Element Namespace
    let prefix = doc.get_prefix(root_id).unwrap_or("(none)");
    let local_name = doc.get_local_name(root_id);
    let ns_uri = doc.get_namespace_uri(root_id).unwrap_or_else(|| "(none)".to_string());

    println!("Root QName: {prefix}:{local_name}");
    println!("Root Namespace URI: {ns_uri}\n");

    // 2. Namespace Scope Lookups
    println!("Looking up prefix 'm' from root: {:?}", doc.lookup_namespace_uri(root_id, "m"));
    println!("Looking up prefix for 'http://schemas.xmlsoap.org/soap/envelope/': {:?}", doc.lookup_prefix(root_id, "http://schemas.xmlsoap.org/soap/envelope/"));

    // 3. Namespace-Aware Query Selectors
    let math_ns = "http://example.org/math";
    let add_elements = doc.get_elements_by_tag_name_ns(math_ns, "Add");
    println!("\nFound {} element(s) with local name 'Add' in namespace '{math_ns}':", add_elements.len());

    for elem_id in add_elements {
        for child_id in doc.get_children(elem_id) {
            if doc.get_node(child_id).map_or(false, |n| matches!(n.kind, xml_lib_rust::NodeKind::Element { .. })) {
                let child_local = doc.get_local_name(child_id);
                let text = doc.get_text_content(child_id);
                println!("  Parameter <m:{child_local}> = {text}");
            }
        }
    }

    println!("\nNamespace example complete.");
    Ok(())
}
