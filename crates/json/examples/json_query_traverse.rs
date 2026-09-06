//! Demonstrates querying and traversing JSON structures
//!
//! Shows safe navigation patterns, tree traversal, searching,
//! and extracting data from complex nested JSON documents.

use json_lib::{parse, BufferSource, Node};

fn main() {
    println!("=== JSON Query and Traversal Demo ===\n");

    // Sample JSON data - a product catalog
    let json_data = r#"{
  "catalog": {
    "name": "Tech Store",
    "categories": [
      {
        "id": 1,
        "name": "Laptops",
        "products": [
          {
            "id": 101,
            "name": "UltraBook Pro",
            "price": 1299.99,
            "in_stock": true,
            "specs": {
              "cpu": "Intel i7",
              "ram": 16,
              "storage": 512
            },
            "tags": ["laptop", "portable", "business"]
          },
          {
            "id": 102,
            "name": "Gaming Laptop X",
            "price": 1899.99,
            "in_stock": false,
            "specs": {
              "cpu": "AMD Ryzen 9",
              "ram": 32,
              "storage": 1024
            },
            "tags": ["laptop", "gaming", "performance"]
          }
        ]
      },
      {
        "id": 2,
        "name": "Accessories",
        "products": [
          {
            "id": 201,
            "name": "Wireless Mouse",
            "price": 29.99,
            "in_stock": true,
            "tags": ["accessory", "peripheral"]
          }
        ]
      }
    ]
  }
}"#;

    let mut source = BufferSource::new(json_data.as_bytes());
    let root = parse(&mut source).expect("Failed to parse JSON");

    // Example 1: Safe navigation with error handling
    println!("1. Safe navigation:");
    
    match root.get("catalog")
        .and_then(|c| c.get("name"))
        .and_then(|n| n.as_str()) {
        Some(name) => println!("  Catalog name: {}", name),
        None => println!("  Catalog name not found"),
    }

    // Example 2: Traversing arrays
    println!("\n2. Traversing categories:");
    
    if let Some(categories) = root.get("catalog")
        .and_then(|c| c.get("categories"))
        .and_then(|c| c.as_array()) {
        
        println!("  Found {} categories", categories.len());
        
        for (idx, category) in categories.iter().enumerate() {
            if let Some(name) = category.get("name").and_then(|n| n.as_str()) {
                println!("  - Category {}: {}", idx + 1, name);
            }
        }
    }

    // Example 3: Deep traversal - finding all products
    println!("\n3. Finding all products:");
    
    fn collect_products<'a>(node: &'a Node, products: &mut Vec<&'a Node>) {
        if let Some(arr) = node.as_array() {
            for item in arr {
                if item.get("price").is_some() {
                    // Looks like a product
                    products.push(item);
                }
                collect_products(item, products);
            }
        } else if let Some(obj) = node.as_object() {
            for (_key, value) in obj {
                collect_products(value, products);
            }
        }
    }
    
    let mut all_products = Vec::new();
    collect_products(&root, &mut all_products);
    
    println!("  Total products found: {}", all_products.len());
    for product in &all_products {
        if let (Some(name), Some(price)) = (
            product.get("name").and_then(|n| n.as_str()),
            product.get("price").and_then(|p| p.as_number()),
        ) {
            println!("    - {} (${:?})", name, price);
        }
    }

    // Example 4: Filtering - find products in stock
    println!("\n4. Products in stock:");
    
    for product in &all_products {
        let in_stock = product.get("in_stock")
            .and_then(|s| s.as_bool())
            .unwrap_or(false);
        
        if in_stock {
            if let Some(name) = product.get("name").and_then(|n| n.as_str()) {
                println!("  - {}", name);
            }
        }
    }

    // Example 5: Searching by criteria
    println!("\n5. Products with price > $1000:");
    
    for product in &all_products {
        if let Some(price_num) = product.get("price").and_then(|p| p.as_number()) {
            // Convert to float for comparison
            let price = match price_num {
                json_lib::Numeric::Float(f) => *f,
                json_lib::Numeric::Integer(i) => *i as f64,
                _ => 0.0,
            };
            
            if price > 1000.0 {
                if let Some(name) = product.get("name").and_then(|n| n.as_str()) {
                    println!("  - {} (${:.2})", name, price);
                }
            }
        }
    }

    // Example 6: Searching within arrays - find products with specific tag
    println!("\n6. Products tagged 'gaming':");
    
    for product in &all_products {
        if let Some(tags) = product.get("tags").and_then(|t| t.as_array()) {
            let has_gaming_tag = tags.iter().any(|tag| {
                tag.as_str() == Some("gaming")
            });
            
            if has_gaming_tag {
                if let Some(name) = product.get("name").and_then(|n| n.as_str()) {
                    println!("  - {}", name);
                }
            }
        }
    }

    // Example 7: Extracting nested data
    println!("\n7. Laptop specifications:");
    
    if let Some(categories) = root.get("catalog")
        .and_then(|c| c.get("categories"))
        .and_then(|c| c.as_array()) {
        
        for category in categories {
            if category.get("name").and_then(|n| n.as_str()) == Some("Laptops") {
                if let Some(products) = category.get("products").and_then(|p| p.as_array()) {
                    for product in products {
                        let name = product.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                        
                        if let Some(specs) = product.get("specs") {
                            let cpu = specs.get("cpu").and_then(|c| c.as_str()).unwrap_or("N/A");
                            let ram = specs.get("ram").and_then(|r| r.as_number());
                            
                            print!("  - {}: CPU={}", name, cpu);
                            if let Some(ram_val) = ram {
                                print!(", RAM={:?}GB", ram_val);
                            }
                            println!();
                        }
                    }
                }
            }
        }
    }

    // Example 8: Counting and aggregation
    println!("\n8. Statistics:");
    
    let total_products = all_products.len();
    let in_stock_count = all_products.iter()
        .filter(|p| p.get("in_stock").and_then(|s| s.as_bool()).unwrap_or(false))
        .count();
    
    // Calculate average price
    let prices: Vec<f64> = all_products.iter()
        .filter_map(|p| p.get("price").and_then(|pr| pr.as_number()))
        .filter_map(|num| match num {
            json_lib::Numeric::Float(f) => Some(*f),
            json_lib::Numeric::Integer(i) => Some(*i as f64),
            _ => None,
        })
        .collect();
    
    let avg_price = if !prices.is_empty() {
        prices.iter().sum::<f64>() / prices.len() as f64
    } else {
        0.0
    };
    
    println!("  Total products: {}", total_products);
    println!("  In stock: {}", in_stock_count);
    println!("  Out of stock: {}", total_products - in_stock_count);
    println!("  Average price: ${:.2}", avg_price);

    // Example 9: Path existence checking
    println!("\n9. Checking for optional fields:");
    
    for (idx, product) in all_products.iter().enumerate() {
        let has_specs = product.get("specs").is_some();
        let has_tags = product.get("tags").is_some();
        let has_description = product.get("description").is_some();
        
        if let Some(name) = product.get("name").and_then(|n| n.as_str()) {
            println!("  Product {}: {}", idx + 1, name);
            println!("    Has specs: {}", has_specs);
            println!("    Has tags: {}", has_tags);
            println!("    Has description: {}", has_description);
        }
    }

    // Example 10: Custom traversal with callback
    println!("\n10. Finding all string values:");
    
    fn find_strings(node: &Node, path: &str, results: &mut Vec<(String, String)>) {
        match node {
            Node::Str(s) => {
                results.push((path.to_string(), s.clone()));
            }
            Node::Array(arr) => {
                for (i, item) in arr.iter().enumerate() {
                    find_strings(item, &format!("{}[{}]", path, i), results);
                }
            }
            Node::Object(obj) => {
                for (key, value) in obj {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    find_strings(value, &new_path, results);
                }
            }
            _ => {}
        }
    }
    
    let mut string_values = Vec::new();
    find_strings(&root, "", &mut string_values);
    
    println!("  Found {} string values", string_values.len());
    for (path, value) in string_values.iter().take(5) {
        println!("    {}: {}", path, value);
    }
    if string_values.len() > 5 {
        println!("    ... and {} more", string_values.len() - 5);
    }

    println!("\n=== Demo Complete ===");
}
