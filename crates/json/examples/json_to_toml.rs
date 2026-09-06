use json_lib::{parse, to_toml, FileDestination, FileSource};
use std::fs;
use std::path::Path;

#[path = "common/utility.rs"]
mod json_utility_lib;
use json_utility_lib::get_json_file_list;

/// Processes a single JSON file by converting it to TOML format.
fn process_json_file(file_path: &str) -> Result<(), String> {
    let mut source = FileSource::new(file_path).map_err(|e| e.to_string())?;
    let node = parse(&mut source).map_err(|e| e.to_string())?;
    let mut destination = FileDestination::new(
        Path::new(file_path)
            .with_extension("toml")
            .to_string_lossy()
            .as_ref(),
    )
    .map_err(|e| e.to_string())?;

    to_toml(&node, &mut destination)?;
    Ok(())
}

fn main() {
    let json_files = get_json_file_list("files");
    for file_path in json_files {
        match process_json_file(&file_path) {
            Ok(()) => println!("Successfully converted {}", file_path),
            Err(e) => {
                eprintln!("Failed to convert {}: {}", file_path, e);
                let toml_file = file_path.replace(".json", ".toml");
                let _ = fs::remove_file(toml_file);
            }
        }
    }
}
