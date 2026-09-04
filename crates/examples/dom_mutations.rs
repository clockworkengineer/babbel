//! # W3C DOM Core Mutations Example
//!
//! Demonstrates live tree manipulation and memory management:
//! - Creating and inserting elements (`insert_before`)
//! - Modifying and removing attributes (`set_attribute`, `remove_attribute`)
//! - Replacing and removing child elements (`replace_child`, `remove_child`)
//! - Deep node cloning (`clone_node`)
//! - Garbage compaction (`compact`) to reclaim memory from dead arena slots

use xml_lib_rust::{parse, stringify};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- W3C DOM Core Mutations Example ---");

    let xml = "<todo_list><task id='1'>Write tests</task><task id='2'>Review PR</task></todo_list>";
    let mut doc = parse(xml)?;
    let root_id = doc.root_element_id().expect("Root element");

    println!("Initial XML:\n{}\n", stringify(&doc));

    // 1. Create a new task element and insert before task 2
    let new_task = doc.create_element("task");
    doc.set_attribute(new_task, "id", "1.5")?;
    doc.set_text_content(new_task, "Run benchmarks")?;

    let task2 = doc.get_children(root_id)[1];
    doc.insert_before(root_id, new_task, task2)?;
    println!("After insert_before (task 1.5 inserted):\n{}\n", stringify(&doc));

    // 2. Replace task 1 with an updated task
    let urgent_task = doc.create_element("urgent_task");
    doc.set_attribute(urgent_task, "priority", "critical")?;
    doc.set_text_content(urgent_task, "Fix regression")?;

    let task1 = doc.get_children(root_id)[0];
    doc.replace_child(root_id, urgent_task, task1)?;
    println!("After replace_child:\n{}\n", stringify(&doc));

    // 3. Remove task 2
    doc.remove_child(root_id, task2)?;
    println!("After remove_child (task 2 removed):\n{}\n", stringify(&doc));

    // 4. Arena Garbage Compaction
    println!("Arena node count before compaction: {}", doc.len());
    doc.compact()?;
    println!("Arena node count after compaction: {}", doc.len());

    println!("\nFinal XML Output:\n{}", stringify(&doc));
    println!("DOM mutation example complete.");
    Ok(())
}
