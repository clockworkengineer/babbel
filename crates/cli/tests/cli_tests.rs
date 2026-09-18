//! Integration tests for babbel-cli executable.

use std::fs;
use std::process::Command;

fn get_babbel_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("babbel.exe");
    if !path.exists() {
        path.set_extension("");
    }
    path
}

#[test]
fn test_cli_version_and_help() {
    let bin = get_babbel_bin();
    let out = Command::new(&bin).arg("--version").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("babbel 0.2.2"));

    let out_help = Command::new(&bin).arg("--help").output().unwrap();
    assert!(out_help.status.success());
    let help_str = String::from_utf8(out_help.stdout).unwrap();
    assert!(help_str.contains("Commands:") || help_str.contains("SUBCOMMANDS:"));
}

#[test]
fn test_cli_convert_subcommand() {
    let bin = get_babbel_bin();
    let temp_dir = std::env::temp_dir().join("babbel_cli_convert_test");
    let _ = fs::create_dir_all(&temp_dir);

    let json_file = temp_dir.join("input.json");
    let yaml_file = temp_dir.join("output.yaml");
    fs::write(&json_file, r#"{"name": "babbel", "version": "0.2.2"}"#).unwrap();

    let status = Command::new(&bin)
        .arg("convert")
        .arg(&json_file)
        .arg("-t")
        .arg("yaml")
        .arg("-o")
        .arg(&yaml_file)
        .status()
        .unwrap();

    assert!(status.success());
    let yaml_content = fs::read_to_string(&yaml_file).unwrap();
    assert!(yaml_content.contains("name: babbel"));
    assert!(yaml_content.contains("version: 0.2.2"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_cli_query_subcommand() {
    let bin = get_babbel_bin();
    let temp_dir = std::env::temp_dir().join("babbel_cli_query_test");
    let _ = fs::create_dir_all(&temp_dir);

    let json_file = temp_dir.join("store.json");
    fs::write(&json_file, r#"{"store": {"book": [{"title": "Rust In Action", "price": 40}, {"title": "Design Patterns", "price": 50}]}}"#).unwrap();

    let out = Command::new(&bin)
        .arg("query")
        .arg(&json_file)
        .arg("-q")
        .arg("$.store.book[?(@.price < 45)].title")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Rust In Action"));
    assert!(!stdout.contains("Design Patterns"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_cli_diff_and_patch_subcommands() {
    let bin = get_babbel_bin();
    let temp_dir = std::env::temp_dir().join("babbel_cli_diff_patch_test");
    let _ = fs::create_dir_all(&temp_dir);

    let file_a = temp_dir.join("a.json");
    let file_b = temp_dir.join("b.json");
    let patch_file = temp_dir.join("patch.json");
    let patched_file = temp_dir.join("patched.json");

    fs::write(&file_a, r#"{"title": "Initial", "active": false}"#).unwrap();
    fs::write(&file_b, r#"{"title": "Updated", "active": true}"#).unwrap();

    // Compute diff
    let diff_out = Command::new(&bin)
        .arg("diff")
        .arg(&file_a)
        .arg(&file_b)
        .output()
        .unwrap();

    assert!(diff_out.status.success());
    fs::write(&patch_file, &diff_out.stdout).unwrap();

    // Apply patch
    let patch_status = Command::new(&bin)
        .arg("patch")
        .arg(&file_a)
        .arg("-p")
        .arg(&patch_file)
        .arg("-o")
        .arg(&patched_file)
        .status()
        .unwrap();

    assert!(patch_status.success());
    let patched_content = fs::read_to_string(&patched_file).unwrap();
    assert!(patched_content.contains("Updated"));
    assert!(patched_content.contains("true"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_cli_validate_subcommand() {
    let bin = get_babbel_bin();
    let temp_dir = std::env::temp_dir().join("babbel_cli_val_test");
    let _ = fs::create_dir_all(&temp_dir);

    let doc_file = temp_dir.join("user.json");
    let schema_file = temp_dir.join("schema.json");

    fs::write(&doc_file, r#"{"username": "admin", "age": 30}"#).unwrap();
    fs::write(
        &schema_file,
        r#"{"type": "object", "required": ["username", "age"]}"#,
    )
    .unwrap();

    let out = Command::new(&bin)
        .arg("validate")
        .arg(&doc_file)
        .arg("--schema")
        .arg(&schema_file)
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("VALID:"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_cli_completions_subcommand() {
    let bin = get_babbel_bin();
    let out = Command::new(&bin)
        .arg("completions")
        .arg("bash")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("babbel"));
}

#[test]
fn test_cli_query_first_flag() {
    let bin = get_babbel_bin();
    let temp_dir = std::env::temp_dir().join("babbel_cli_first_test");
    let _ = fs::create_dir_all(&temp_dir);

    let json_file = temp_dir.join("data.json");
    fs::write(&json_file, r#"{"items": [10, 20, 30]}"#).unwrap();

    let out = Command::new(&bin)
        .arg("query")
        .arg(&json_file)
        .arg("-q")
        .arg("$.items[*]")
        .arg("--first")
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.trim(), "10");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_cli_error_exit_code_without_panic() {
    let bin = get_babbel_bin();
    let out = Command::new(&bin)
        .arg("convert")
        .arg("non_existent_file_xyz_123.json")
        .arg("-t")
        .arg("yaml")
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("error: failed to read file"));
    assert!(!stderr.contains("panicked at"));
}
