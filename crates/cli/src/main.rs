//! # Babbel CLI
//!
//! Universal command-line utility for multi-format serialization, document querying,
//! AST diffing, RFC 6902 patching, schema validation, and formatting.

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;

use babbel::default_registry;
use babbel::core::io::Buffer;
use babbel::core::{
    apply_merge_patch, diff, diff_merge_patch, jsonpath_query, CompiledSchema,
    FormatOptions, Patch, Value, FormatEmitter,
};

fn print_help() {
    println!(
        r#"Babbel CLI {} - Universal Polyglot Serialization & Document Toolkit

USAGE:
    babbel <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    convert     Convert documents between any supported formats
    query       Query documents using RFC 9535 JSONPath expressions
    diff        Compute structural diff (RFC 6902 Patch or RFC 7396 Merge Patch)
    patch       Apply an RFC 6902 or RFC 7396 patch to a document
    validate    Validate a document against a JSON Schema (Draft 7 / 2020-12)
    fmt         Format and pretty-print a document
    inspect     Inspect format, structure, size, and depth of a document
    help        Print this help message
    version     Print version information

OPTIONS:
    -h, --help       Print help
    -V, --version    Print version
"#,
        env!("CARGO_PKG_VERSION")
    );
}

fn print_convert_help() {
    println!(
        r#"Convert documents between formats

USAGE:
    babbel convert <INPUT> -t <TO_FORMAT> [OPTIONS]

ARGS:
    <INPUT>              Path to input file (use '-' for stdin)

OPTIONS:
    -f, --from <FORMAT>  Source format (auto-detected from file extension if omitted)
    -t, --to <FORMAT>    Target format (e.g. json, yaml, toml, xml, cbor, msgpack, bson, ron, kdl, parquet, hcl, avro)
    -o, --output <PATH>  Output file path (defaults to stdout)
    --pretty             Pretty-print formatted output (default: true for text formats)
    --indent <N>         Indentation spaces (default: 2)
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        process::exit(0);
    }

    match args[1].as_str() {
        "-h" | "--help" | "help" => {
            print_help();
        }
        "-V" | "--version" | "version" => {
            println!("babbel {}", env!("CARGO_PKG_VERSION"));
        }
        "convert" => cmd_convert(&args[2..]),
        "query" => cmd_query(&args[2..]),
        "diff" => cmd_diff(&args[2..]),
        "patch" => cmd_patch(&args[2..]),
        "validate" => cmd_validate(&args[2..]),
        "fmt" => cmd_fmt(&args[2..]),
        "inspect" => cmd_inspect(&args[2..]),
        other => {
            eprintln!("error: unrecognized subcommand '{}'. Run 'babbel --help' for usage.", other);
            process::exit(1);
        }
    }
}

fn read_input(path: &str) -> (Vec<u8>, Option<String>) {
    if path == "-" {
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer).expect("failed to read from stdin");
        (buffer, None)
    } else {
        let bytes = fs::read(path).unwrap_or_else(|e| {
            eprintln!("error: failed to read file '{}': {}", path, e);
            process::exit(1);
        });
        let ext = Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        (bytes, ext)
    }
}

fn detect_or_get_engine(format: Option<&str>, ext: Option<&str>) -> std::sync::Arc<dyn babbel::FormatEngine> {
    let registry = default_registry();
    if let Some(fmt) = format {
        if let Some(engine) = registry.get(fmt) {
            return engine;
        }
        eprintln!("error: unsupported format '{}'. Available: {:?}", fmt, registry.available_formats());
        process::exit(1);
    }
    if let Some(e) = ext {
        if let Some(engine) = registry.get_by_extension(e) {
            return engine;
        }
    }
    // Fallback: try json
    registry.get("json").expect("json engine must exist")
}

fn cmd_convert(args: &[String]) {
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print_convert_help();
        return;
    }

    let mut input_path = None;
    let mut from_format = None;
    let mut to_format = None;
    let mut output_path = None;
    let mut pretty = true;
    let mut indent = 2;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--from" => {
                i += 1;
                from_format = args.get(i).cloned();
            }
            "-t" | "--to" => {
                i += 1;
                to_format = args.get(i).cloned();
            }
            "-o" | "--output" => {
                i += 1;
                output_path = args.get(i).cloned();
            }
            "--pretty" => pretty = true,
            "--compact" => pretty = false,
            "--indent" => {
                i += 1;
                if let Some(val) = args.get(i).and_then(|s| s.parse().ok()) {
                    indent = val;
                }
            }
            other if !other.starts_with('-') && input_path.is_none() => {
                input_path = Some(other.to_string());
            }
            other => {
                eprintln!("error: unrecognized argument '{}'", other);
                process::exit(1);
            }
        }
        i += 1;
    }

    let input_path = input_path.unwrap_or_else(|| {
        eprintln!("error: input file is required");
        process::exit(1);
    });

    let to_fmt = to_format.unwrap_or_else(|| {
        eprintln!("error: target format (-t / --to) is required");
        process::exit(1);
    });

    let (input_bytes, ext) = read_input(&input_path);
    let src_engine = detect_or_get_engine(from_format.as_deref(), ext.as_deref());
    let dst_engine = detect_or_get_engine(Some(&to_fmt), None);

    let val = src_engine.parse_bytes(&input_bytes).unwrap_or_else(|e| {
        eprintln!("error: failed to parse input: {}", e);
        process::exit(1);
    });

    let mut dest = Buffer::new();
    let options = FormatOptions { pretty, indent };
    dst_engine.serialize(&val, &mut dest, &options).unwrap_or_else(|e| {
        eprintln!("error: failed to serialize output: {}", e);
        process::exit(1);
    });

    let out_bytes = dest.into_vec();
    if let Some(out_p) = output_path {
        fs::write(&out_p, &out_bytes).unwrap_or_else(|e| {
            eprintln!("error: failed to write to '{}': {}", out_p, e);
            process::exit(1);
        });
    } else {
        io::stdout().write_all(&out_bytes).unwrap();
        if !out_bytes.ends_with(b"\n") && !dst_engine.is_binary() {
            println!();
        }
    }
}

fn cmd_query(args: &[String]) {
    if args.len() < 2 {
        println!("USAGE: babbel query <INPUT> -q <JSONPATH>");
        return;
    }

    let input_path = &args[0];
    let mut query_expr = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "-q" || args[i] == "--query" {
            query_expr = args.get(i + 1).cloned();
            i += 1;
        }
        i += 1;
    }

    let query_str = query_expr.unwrap_or_else(|| {
        eprintln!("error: query expression (-q) is required");
        process::exit(1);
    });

    let (input_bytes, ext) = read_input(input_path);
    let engine = detect_or_get_engine(None, ext.as_deref());
    let val = engine.parse_bytes(&input_bytes).unwrap_or_else(|e| {
        eprintln!("error: parse error: {}", e);
        process::exit(1);
    });

    let results = jsonpath_query(&val, &query_str).unwrap_or_else(|e| {
        eprintln!("error: query evaluation error: {}", e);
        process::exit(1);
    });

    let res_array = Value::Array(results.into_iter().cloned().collect());
    let mut dest = Buffer::new();
    let json_engine = default_registry().get("json").unwrap();
    json_engine.emit_pretty(&res_array, &mut dest, 2).unwrap();
    println!("{}", dest.to_string());
}

fn cmd_diff(args: &[String]) {
    if args.len() < 2 {
        println!("USAGE: babbel diff <FILE1> <FILE2> [--format patch|merge]");
        return;
    }

    let file1 = &args[0];
    let file2 = &args[1];
    let use_merge = args.iter().any(|a| a == "--merge");

    let (bytes1, ext1) = read_input(file1);
    let (bytes2, ext2) = read_input(file2);

    let engine1 = detect_or_get_engine(None, ext1.as_deref());
    let engine2 = detect_or_get_engine(None, ext2.as_deref());

    let val1 = engine1.parse_bytes(&bytes1).unwrap();
    let val2 = engine2.parse_bytes(&bytes2).unwrap();

    let json_engine = default_registry().get("json").unwrap();
    let mut dest = Buffer::new();

    if use_merge {
        let merge_patch = diff_merge_patch(&val1, &val2);
        json_engine.emit_pretty(&merge_patch, &mut dest, 2).unwrap();
    } else {
        let patch = diff(&val1, &val2);
        let patch_val = patch.to_value();
        json_engine.emit_pretty(&patch_val, &mut dest, 2).unwrap();
    }

    println!("{}", dest.to_string());
}

fn cmd_patch(args: &[String]) {
    if args.len() < 3 {
        println!("USAGE: babbel patch <INPUT> -p <PATCH_FILE> [-o <OUTPUT>]");
        return;
    }

    let input_path = &args[0];
    let mut patch_path = None;
    let mut output_path = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--patch" => {
                patch_path = args.get(i + 1).cloned();
                i += 1;
            }
            "-o" | "--output" => {
                output_path = args.get(i + 1).cloned();
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    let patch_file = patch_path.expect("patch file (-p) required");
    let (input_bytes, ext) = read_input(input_path);
    let (patch_bytes, patch_ext) = read_input(&patch_file);

    let engine = detect_or_get_engine(None, ext.as_deref());
    let patch_engine = detect_or_get_engine(None, patch_ext.as_deref());

    let mut target_val = engine.parse_bytes(&input_bytes).unwrap();
    let patch_val = patch_engine.parse_bytes(&patch_bytes).unwrap();

    if let Ok(patch) = Patch::from_value(&patch_val) {
        patch.apply_inplace(&mut target_val).expect("failed to apply patch");
    } else {
        apply_merge_patch(&mut target_val, &patch_val);
    }

    let mut dest = Buffer::new();
    engine.emit_pretty(&target_val, &mut dest, 2).unwrap();
    let out_bytes = dest.into_vec();

    if let Some(out_p) = output_path {
        fs::write(out_p, out_bytes).unwrap();
    } else {
        io::stdout().write_all(&out_bytes).unwrap();
        println!();
    }
}

fn cmd_validate(args: &[String]) {
    if args.len() < 3 {
        println!("USAGE: babbel validate <INPUT> --schema <SCHEMA_FILE>");
        return;
    }

    let input_path = &args[0];
    let schema_file = args.iter().position(|a| a == "--schema" || a == "-s")
        .and_then(|idx| args.get(idx + 1))
        .expect("schema file (--schema) required");

    let (input_bytes, ext) = read_input(input_path);
    let (schema_bytes, s_ext) = read_input(schema_file);

    let engine = detect_or_get_engine(None, ext.as_deref());
    let schema_engine = detect_or_get_engine(None, s_ext.as_deref());

    let val = engine.parse_bytes(&input_bytes).unwrap();
    let schema_val = schema_engine.parse_bytes(&schema_bytes).unwrap();

    let compiled = CompiledSchema::compile(&schema_val).expect("failed to compile schema");
    match compiled.validate(&val) {
        Ok(()) => {
            println!("VALID: '{}' conforms to schema '{}'", input_path, schema_file);
        }
        Err(errors) => {
            eprintln!("INVALID: '{}' violates schema ({} errors):", input_path, errors.len());
            for err in errors {
                eprintln!("  - [{}] at '{}': {}", err.keyword, err.pointer, err.message);
            }
            process::exit(1);
        }
    }
}

fn cmd_fmt(args: &[String]) {
    if args.is_empty() {
        println!("USAGE: babbel fmt <INPUT> [--indent <N>] [--in-place]");
        return;
    }

    let input_path = &args[0];
    let in_place = args.iter().any(|a| a == "--in-place" || a == "-i");
    let indent = args.iter().position(|a| a == "--indent")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    let (input_bytes, ext) = read_input(input_path);
    let engine = detect_or_get_engine(None, ext.as_deref());
    let val = engine.parse_bytes(&input_bytes).unwrap();

    let mut dest = Buffer::new();
    engine.emit_pretty(&val, &mut dest, indent).unwrap();
    let out_bytes = dest.into_vec();

    if in_place && input_path != "-" {
        fs::write(input_path, out_bytes).unwrap();
        println!("Formatted '{}'", input_path);
    } else {
        io::stdout().write_all(&out_bytes).unwrap();
        println!();
    }
}

fn cmd_inspect(args: &[String]) {
    if args.is_empty() {
        println!("USAGE: babbel inspect <INPUT>");
        return;
    }

    let input_path = &args[0];
    let (input_bytes, ext) = read_input(input_path);
    let engine = detect_or_get_engine(None, ext.as_deref());
    let val = engine.parse_bytes(&input_bytes).unwrap();

    println!("File:             {}", input_path);
    println!("Detected Format:  {} ({})", engine.format_id(), engine.mime_type());
    println!("Byte Size:        {} bytes", input_bytes.len());
    println!("Root Type:        {}", match &val {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Integer(_) => "integer",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Bytes(_) => "bytes",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    });

    if let Value::Object(entries) = &val {
        println!("Key Count:        {}", entries.len());
    } else if let Value::Array(items) = &val {
        println!("Element Count:    {}", items.len());
    }

    fn calculate_depth(v: &Value) -> usize {
        match v {
            Value::Array(items) => 1 + items.iter().map(calculate_depth).max().unwrap_or(0),
            Value::Object(entries) => 1 + entries.iter().map(|(_, val)| calculate_depth(val)).max().unwrap_or(0),
            _ => 1,
        }
    }

    println!("Max AST Depth:    {}", calculate_depth(&val));
}
