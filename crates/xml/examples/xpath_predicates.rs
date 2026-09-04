//! # XPath 1.0 Filter Predicates Example
//!
//! Demonstrates evaluating XPath attribute predicates (`[@category='web']`)
//! and positional index predicates (`[1]`) using `XPathEngine`.

use xml_lib_rust::{parse, XPathEngine};

fn main() {
    println!("--- XPath Predicates Example ---");
    let xml = r#"<bookstore>
  <book category="cooking"><title>Everyday Italian</title></book>
  <book category="children"><title>Harry Potter</title></book>
  <book category="web"><title>Learning XML</title></book>
</bookstore>"#;

    // Parse XML string into DOM arena
    let doc = parse(xml).expect("Parse clean");
    let xpath = XPathEngine::new(&doc);

    // Attribute Predicate: Filter books with category attribute equal to "web"
    let web_books = xpath
        .evaluate_nodes("//book[@category='web']", None)
        .unwrap();
    println!("Web books count: {}", web_books.len());

    // Position Predicate: Select first book element in sequence
    let first_book = xpath.evaluate_nodes("//book[1]", None).unwrap();
    println!("First book selected ID: {:?}", first_book.first());
}
