//! # XPath 1.0 Built-in Functions Example
//!
//! Demonstrates evaluating XPath numeric and string functions (`count()`, `sum()`, `concat()`, `round()`)
//! against a DOM `Document` using `XPathEngine`.

use xml_lib_rust::{parse, XPathEngine};

fn main() {
    println!("--- XPath Functions Example ---");
    let xml = r#"<bookstore>
  <book category="cooking"><title>Everyday Italian</title><price>30.00</price></book>
  <book category="web"><title>Learning XML</title><price>39.95</price></book>
</bookstore>"#;

    // Parse XML string into DOM arena
    let doc = parse(xml).expect("Parse clean");
    let xpath = XPathEngine::new(&doc);

    // Evaluate count(//book)
    let count = xpath.evaluate("count(//book)", None).unwrap();
    println!("count(//book) = {:?}", count);

    // Evaluate sum(//price)
    let sum = xpath.evaluate("sum(//price)", None).unwrap();
    println!("sum(//price) = {:?}", sum);

    // Evaluate string concatenation function concat(...)
    let concat = xpath
        .evaluate("concat('Total Books: ', count(//book))", None)
        .unwrap();
    println!("concat() = {:?}", concat);

    // Evaluate math rounding function round(...)
    let round = xpath.evaluate("round(39.95)", None).unwrap();
    println!("round(39.95) = {:?}", round);
}
