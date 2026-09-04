//! # Advanced XPath 1.0 Example
//!
//! Demonstrates advanced XPath 1.0 features:
//! - Variable reference bindings (`$var`) in expressions
//! - Custom extension functions registered with `register_function`
//! - XPath 1.0 context functions: `position()` and `last()`
//! - Modern string manipulation functions: `ends-with()`, `lower-case()`

use xml_lib_rust::{parse, XPathEngine, XPathValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Advanced XPath 1.0 Example ---");

    let xml = r#"
    <catalog>
      <product id="p1" category="hardware" price="299.99">
        <name>Mechanical Keyboard</name>
      </product>
      <product id="p2" category="software" price="49.95">
        <name>Text Editor Pro</name>
      </product>
      <product id="p3" category="hardware" price="89.50">
        <name>Optical Mouse</name>
      </product>
    </catalog>"#;

    let doc = parse(xml)?;
    let mut engine = XPathEngine::new(&doc);

    // 1. Variable Bindings: Filter products exceeding a dynamic budget threshold ($budget)
    engine.set_variable("budget", XPathValue::Number(100.0));
    println!("Evaluating products where @price > $budget ($100.0):");
    let budget_matches = engine.evaluate_nodes("//product[@price > $budget]/name", None)?;
    for nid in budget_matches {
        println!("  - {}", doc.get_text_content(nid));
    }

    // 2. Custom Extension Function: Calculate tax rate
    engine.register_function("with_tax", |args| {
        let price = match args.first() {
            Some(XPathValue::Number(num)) => *num,
            _ => 0.0,
        };
        let tax_rate = 0.20; // 20% VAT
        Ok(XPathValue::Number(price * (1.0 + tax_rate)))
    });

    println!("\nEvaluating custom function with_tax(50.0):");
    if let XPathValue::Number(total_with_tax) = engine.evaluate("with_tax(50.0)", None)? {
        println!("  $50.00 with 20% tax: ${total_with_tax:.2}");
    }

    // 3. Positional Predicates using position() and last()
    println!("\nFirst and last products in catalog:");
    let first = engine.evaluate_nodes("//product[position() = 1]/name", None)?;
    let last = engine.evaluate_nodes("//product[position() = last()]/name", None)?;
    if let Some(&id) = first.first() {
        println!("  First product: {}", doc.get_text_content(id));
    }
    if let Some(&id) = last.first() {
        println!("  Last product: {}", doc.get_text_content(id));
    }

    println!("\nAdvanced XPath example complete.");
    Ok(())
}
