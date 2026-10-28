//! # Babbel CLI
//!
//! Universal command-line utility for multi-format serialization, document querying,
//! AST diffing, RFC 6902 patching, schema validation, and formatting.

use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};

use babbel::core::io::Buffer;
use babbel::core::{
    CompiledSchema, FormatOptions, Patch, Value, apply_merge_patch, diff, diff_merge_patch,
    infer_schema, jsonpath_query, transform,
};
use babbel::{FormatEmitter, default_registry};

#[derive(Parser, Debug)]
#[command(
    name = "babbel",
    about = "Universal Polyglot Serialization & Document Toolkit",
    version,
    long_about = "Universal command-line utility for multi-format serialization, RFC 9535 JSONPath querying, AST diffing, RFC 6902/7396 patching, schema validation, and document formatting across 16 formats."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert documents between any supported formats
    Convert(ConvertArgs),

    /// Query documents using RFC 9535 JSONPath expressions
    Query(QueryArgs),

    /// Compute structural diff (RFC 6902 Patch or RFC 7396 Merge Patch)
    Diff(DiffArgs),

    /// Apply an RFC 6902 or RFC 7396 patch to a document
    Patch(PatchArgs),

    /// Validate a document against a JSON Schema (Draft 7 / 2020-12)
    Validate(ValidateArgs),

    /// Format and pretty-print a document
    Fmt(FmtArgs),

    /// Inspect format, structure, size, and depth of a document
    Inspect(InspectArgs),

    /// Set key or path values in documents (e.g. 'server.port=8080')
    Set(SetArgs),

    /// Sequentially merge multiple documents using RFC 7396 Merge Patch
    Merge(MergeArgs),

    /// Generate JSON Schema Draft 2020-12 from document structure
    Schema(SchemaArgs),

    /// Generate shell autocompletions (bash, zsh, fish, powershell, elvish)
    Completions(CompletionsArgs),
}

#[derive(Args, Debug)]
struct ConvertArgs {
    /// Path to input file (use '-' for stdin)
    input: String,

    /// Source format (auto-detected from file extension if omitted)
    #[arg(short = 'f', long = "from")]
    from: Option<String>,

    /// Target format (e.g. json, yaml, toml, xml, cbor, msgpack, bson, ron, kdl, parquet, hcl, avro)
    #[arg(short = 't', long = "to")]
    to: String,

    /// Output file path (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Pretty-print formatted output (default: true for text formats)
    #[arg(long = "pretty", default_value_t = true, conflicts_with = "compact")]
    pretty: bool,

    /// Emit compact output without indentation
    #[arg(long = "compact")]
    compact: bool,

    /// Indentation spaces (default: 2)
    #[arg(long = "indent", default_value_t = 2)]
    indent: usize,
}

#[derive(Args, Debug)]
struct QueryArgs {
    /// Path to input file (use '-' for stdin)
    input: String,

    /// RFC 9535 JSONPath query expression (e.g. '$.store.book[*].title')
    #[arg(short = 'q', long = "query", default_value = "$")]
    query: String,

    /// Input document format (inferred from extension if omitted)
    #[arg(short = 'f', long = "from")]
    from: Option<String>,

    /// Transform pipeline expression (e.g. 'map(.name) | sort | unique')
    #[arg(long = "transform")]
    transform: Option<String>,

    /// Target output format (default: json)
    #[arg(short = 't', long = "to", default_value = "json")]
    to: String,

    /// Return only the first matching element instead of an array
    #[arg(long = "first")]
    first: bool,

    /// Output file path (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Indentation spaces (default: 2)
    #[arg(long = "indent", default_value_t = 2)]
    indent: usize,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum DiffFormat {
    Patch,
    Merge,
}

#[derive(Args, Debug)]
struct DiffArgs {
    /// Original document file path (use '-' for stdin)
    file1: String,

    /// Target document file path
    file2: String,

    /// Diff representation format: patch (RFC 6902) or merge (RFC 7396)
    #[arg(long = "format", value_enum, default_value_t = DiffFormat::Patch)]
    format: DiffFormat,

    /// Shortcut for '--format merge' (RFC 7396 JSON Merge Patch)
    #[arg(long = "merge")]
    merge: bool,

    /// Output file path (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    output: Option<String>,
}

#[derive(Args, Debug)]
struct PatchArgs {
    /// Path to target document (use '-' for stdin)
    input: String,

    /// Path to RFC 6902 or RFC 7396 patch file
    #[arg(short = 'p', long = "patch")]
    patch: String,

    /// Force RFC 7396 Merge Patch semantics
    #[arg(long = "merge")]
    merge: bool,

    /// Output file path (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    output: Option<String>,
}

#[derive(Args, Debug)]
struct ValidateArgs {
    /// Path to document to validate (use '-' for stdin)
    input: String,

    /// Path to JSON Schema file
    #[arg(short = 's', long = "schema")]
    schema: String,
}

#[derive(Args, Debug)]
struct FmtArgs {
    /// Path to input file (use '-' for stdin)
    input: String,

    /// Indentation spaces (default: 2)
    #[arg(long = "indent", default_value_t = 2)]
    indent: usize,

    /// Overwrite input file in-place
    #[arg(short = 'i', long = "in-place")]
    in_place: bool,

    /// Output file path (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    output: Option<String>,
}

#[derive(Args, Debug)]
struct InspectArgs {
    /// Path to input file (use '-' for stdin)
    input: String,
}

#[derive(Args, Debug)]
struct CompletionsArgs {
    /// Target shell to generate completions for
    shell: Shell,
}

#[derive(Args, Debug)]
struct SetArgs {
    /// Path to input file (use '-' for stdin)
    input: String,

    /// Key-value assignment pairs (e.g. 'foo.bar=123', 'active=true')
    #[arg(required = true, num_args = 1..)]
    assignments: Vec<String>,

    /// Source format (auto-detected from file extension if omitted)
    #[arg(short = 'f', long = "from")]
    from: Option<String>,

    /// Target format (auto-detected from input format if omitted)
    #[arg(short = 't', long = "to")]
    to: Option<String>,

    /// Overwrite input file in-place
    #[arg(short = 'i', long = "in-place")]
    in_place: bool,

    /// Output file path (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Indentation spaces (default: 2)
    #[arg(long = "indent", default_value_t = 2)]
    indent: usize,
}

#[derive(Args, Debug)]
struct MergeArgs {
    /// Paths to input files to merge sequentially (left-to-right)
    #[arg(required = true, num_args = 1..)]
    inputs: Vec<String>,

    /// Output target format (defaults to format of first input file)
    #[arg(short = 't', long = "to")]
    to: Option<String>,

    /// Output file path (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Emit compact output without indentation
    #[arg(long = "compact")]
    compact: bool,

    /// Indentation spaces (default: 2)
    #[arg(long = "indent", default_value_t = 2)]
    indent: usize,
}

#[derive(Args, Debug)]
struct SchemaArgs {
    /// Path to input file (use '-' for stdin)
    input: String,

    /// Input document format (inferred from extension if omitted)
    #[arg(short = 'f', long = "from")]
    from: Option<String>,

    /// Target schema output format (default: json)
    #[arg(short = 't', long = "to", default_value = "json")]
    to: String,

    /// Output file path (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Indentation spaces (default: 2)
    #[arg(long = "indent", default_value_t = 2)]
    indent: usize,
}

#[derive(Debug)]
enum CliError {
    Io(String),
    Format(String),
    Validation(String),
    Usage(String),
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        match err {
            CliError::Usage(msg) => {
                eprintln!("error: {}", msg);
                process::exit(1);
            }
            CliError::Io(msg) | CliError::Format(msg) => {
                eprintln!("error: {}", msg);
                process::exit(2);
            }
            CliError::Validation(msg) => {
                eprintln!("{}", msg);
                process::exit(3);
            }
        }
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Commands::Convert(args) => cmd_convert(args),
        Commands::Query(args) => cmd_query(args),
        Commands::Diff(args) => cmd_diff(args),
        Commands::Patch(args) => cmd_patch(args),
        Commands::Validate(args) => cmd_validate(args),
        Commands::Fmt(args) => cmd_fmt(args),
        Commands::Inspect(args) => cmd_inspect(args),
        Commands::Completions(args) => cmd_completions(args),
        Commands::Set(args) => cmd_set(args),
        Commands::Merge(args) => cmd_merge(args),
        Commands::Schema(args) => cmd_schema(args),
    }
}

fn read_input(path: &str) -> Result<(Vec<u8>, Option<String>), CliError> {
    let mut bytes = if path == "-" {
        let mut buffer = Vec::new();
        io::stdin()
            .read_to_end(&mut buffer)
            .map_err(|e| CliError::Io(format!("failed to read from stdin: {}", e)))?;
        buffer
    } else {
        fs::read(path)
            .map_err(|e| CliError::Io(format!("failed to read file '{}': {}", path, e)))?
    };

    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        bytes.drain(0..3);
    }

    let ext = if path == "-" {
        None
    } else {
        Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
    };

    Ok((bytes, ext))
}

fn write_output(bytes: &[u8], path: Option<&str>, is_binary: bool) -> Result<(), CliError> {
    if let Some(out_p) = path {
        fs::write(out_p, bytes)
            .map_err(|e| CliError::Io(format!("failed to write to '{}': {}", out_p, e)))?;
    } else {
        io::stdout()
            .write_all(bytes)
            .map_err(|e| CliError::Io(format!("failed to write to stdout: {}", e)))?;
        if !bytes.ends_with(b"\n") && !is_binary {
            println!();
        }
    }
    Ok(())
}

fn detect_engine(
    format: Option<&str>,
    ext: Option<&str>,
) -> Result<std::sync::Arc<dyn babbel::FormatEngine>, CliError> {
    let registry = default_registry();
    if let Some(fmt) = format {
        if let Some(engine) = registry.get(fmt) {
            return Ok(engine);
        }
        return Err(CliError::Usage(format!(
            "unsupported format '{}'. Available formats: {:?}",
            fmt,
            registry.available_formats()
        )));
    }
    if let Some(e) = ext {
        if let Some(engine) = registry.get_by_extension(e) {
            return Ok(engine);
        }
    }
    // Fallback: try json
    registry
        .get("json")
        .ok_or_else(|| CliError::Format("json format engine not found".to_string()))
}

fn cmd_convert(args: ConvertArgs) -> Result<(), CliError> {
    let (input_bytes, ext) = read_input(&args.input)?;
    let src_engine = detect_engine(args.from.as_deref(), ext.as_deref())?;
    let dst_engine = detect_engine(Some(&args.to), None)?;

    let val = src_engine
        .parse_bytes(&input_bytes)
        .map_err(|e| CliError::Format(format!("failed to parse input: {}", e)))?;

    let mut dest = Buffer::new();
    let pretty = if args.compact { false } else { args.pretty };
    let options = FormatOptions {
        pretty,
        indent: args.indent,
    };
    dst_engine
        .serialize(&val, &mut dest, &options)
        .map_err(|e| CliError::Format(format!("failed to serialize output: {}", e)))?;

    let out_bytes = dest.into_vec();
    write_output(&out_bytes, args.output.as_deref(), dst_engine.is_binary())
}

fn cmd_query(args: QueryArgs) -> Result<(), CliError> {
    let (input_bytes, ext) = read_input(&args.input)?;
    let engine = detect_engine(args.from.as_deref(), ext.as_deref())?;
    let val = engine
        .parse_bytes(&input_bytes)
        .map_err(|e| CliError::Format(format!("parse error: {}", e)))?;

    let mut output_val = if args.query.trim() == "$" || args.query.trim().is_empty() {
        val
    } else {
        let results = jsonpath_query(&val, &args.query)
            .map_err(|e| CliError::Format(format!("query evaluation error: {}", e)))?;
        if args.first {
            results.into_iter().next().cloned().unwrap_or(Value::Null)
        } else {
            Value::Array(results.into_iter().cloned().collect())
        }
    };

    if let Some(ref expr) = args.transform {
        output_val = transform(&output_val, expr)
            .map_err(|e| CliError::Format(format!("transform error: {}", e)))?;
    }

    let dst_engine = detect_engine(Some(&args.to), None)?;
    let mut dest = Buffer::new();
    let options = FormatOptions {
        pretty: true,
        indent: args.indent,
    };
    dst_engine
        .serialize(&output_val, &mut dest, &options)
        .map_err(|e| CliError::Format(format!("failed to format query output: {}", e)))?;

    let out_bytes = dest.into_vec();
    write_output(&out_bytes, args.output.as_deref(), dst_engine.is_binary())
}

fn parse_literal_value(s: &str) -> Value {
    let trimmed = s.trim();
    if trimmed == "null" {
        Value::Null
    } else if trimmed == "true" {
        Value::Bool(true)
    } else if trimmed == "false" {
        Value::Bool(false)
    } else if let Ok(i) = trimmed.parse::<i128>() {
        Value::Integer(i)
    } else if let Ok(f) = trimmed.parse::<f64>() {
        Value::Float(f)
    } else if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        || (trimmed.starts_with('"') && trimmed.ends_with('"'))
    {
        if let Some(json_engine) = default_registry().get("json") {
            if let Ok(v) = json_engine.parse_str(trimmed) {
                return v;
            }
        }
        Value::String(trimmed.to_string())
    } else {
        Value::String(trimmed.to_string())
    }
}

fn set_value_at_path(val: &mut Value, path: &str, new_val: Value) -> Result<(), CliError> {
    let clean = path.trim_start_matches('$').trim_start_matches('.');
    let parts: Vec<&str> = clean.split('.').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        *val = new_val;
        return Ok(());
    }

    let mut current = val;
    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;
        if let Some(bracket_idx) = part.find('[') {
            if !part.ends_with(']') {
                return Err(CliError::Usage(format!("invalid path segment '{}'", part)));
            }
            let field_name = &part[..bracket_idx];
            let idx_str = &part[bracket_idx + 1..part.len() - 1];
            let idx: usize = idx_str.parse().map_err(|_| {
                CliError::Usage(format!(
                    "invalid array index '{}' in path '{}'",
                    idx_str, part
                ))
            })?;

            if !field_name.is_empty() {
                if !matches!(current, Value::Object(_)) {
                    *current = Value::Object(Vec::new());
                }
                if let Value::Object(entries) = current {
                    let pos = entries.iter().position(|(k, _)| k == field_name);
                    let idx_entry = match pos {
                        Some(p) => p,
                        None => {
                            entries.push((field_name.to_string(), Value::Array(Vec::new())));
                            entries.len() - 1
                        }
                    };
                    current = &mut entries[idx_entry].1;
                }
            }

            if !matches!(current, Value::Array(_)) {
                *current = Value::Array(Vec::new());
            }

            if let Value::Array(items) = current {
                while items.len() <= idx {
                    items.push(Value::Null);
                }
                if is_last {
                    items[idx] = new_val;
                    return Ok(());
                } else {
                    current = &mut items[idx];
                }
            }
        } else if is_last {
            if !matches!(current, Value::Object(_)) {
                *current = Value::Object(Vec::new());
            }
            if let Value::Object(entries) = current {
                if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == *part) {
                    *v = new_val;
                } else {
                    entries.push((part.to_string(), new_val));
                }
            }
            return Ok(());
        } else {
            if !matches!(current, Value::Object(_)) {
                *current = Value::Object(Vec::new());
            }
            if let Value::Object(entries) = current {
                let pos = entries.iter().position(|(k, _)| k == *part);
                let entry_idx = match pos {
                    Some(p) => p,
                    None => {
                        entries.push((part.to_string(), Value::Object(Vec::new())));
                        entries.len() - 1
                    }
                };
                current = &mut entries[entry_idx].1;
            }
        }
    }
    Ok(())
}

fn cmd_set(args: SetArgs) -> Result<(), CliError> {
    let (input_bytes, ext) = read_input(&args.input)?;
    let src_engine = detect_engine(args.from.as_deref(), ext.as_deref())?;
    let mut val = src_engine
        .parse_bytes(&input_bytes)
        .map_err(|e| CliError::Format(format!("failed to parse input: {}", e)))?;

    for assignment in &args.assignments {
        let (path, value_str) = assignment.split_once('=').ok_or_else(|| {
            CliError::Usage(format!(
                "invalid assignment '{}': expected format <path>=<value>",
                assignment
            ))
        })?;
        let parsed_val = parse_literal_value(value_str);
        set_value_at_path(&mut val, path.trim(), parsed_val)?;
    }

    let dst_engine = if let Some(target_fmt) = &args.to {
        detect_engine(Some(target_fmt), None)?
    } else {
        src_engine
    };

    let mut dest = Buffer::new();
    let options = FormatOptions {
        pretty: true,
        indent: args.indent,
    };
    dst_engine
        .serialize(&val, &mut dest, &options)
        .map_err(|e| CliError::Format(format!("failed to format output: {}", e)))?;
    let out_bytes = dest.into_vec();

    if args.in_place && args.input != "-" {
        fs::write(&args.input, &out_bytes).map_err(|e| {
            CliError::Io(format!(
                "failed to write in-place to '{}': {}",
                args.input, e
            ))
        })?;
        println!("Updated '{}'", args.input);
        Ok(())
    } else {
        write_output(&out_bytes, args.output.as_deref(), dst_engine.is_binary())
    }
}

fn cmd_merge(args: MergeArgs) -> Result<(), CliError> {
    if args.inputs.is_empty() {
        return Err(CliError::Usage(
            "at least one input file required for merge".into(),
        ));
    }

    let (first_bytes, first_ext) = read_input(&args.inputs[0])?;
    let first_engine = detect_engine(None, first_ext.as_deref())?;
    let mut base_val = first_engine
        .parse_bytes(&first_bytes)
        .map_err(|e| CliError::Format(format!("failed to parse '{}': {}", args.inputs[0], e)))?;

    for input_path in &args.inputs[1..] {
        let (bytes, ext) = read_input(input_path)?;
        let engine = detect_engine(None, ext.as_deref())?;
        let next_val = engine
            .parse_bytes(&bytes)
            .map_err(|e| CliError::Format(format!("failed to parse '{}': {}", input_path, e)))?;
        apply_merge_patch(&mut base_val, &next_val);
    }

    let dst_engine = if let Some(target_fmt) = &args.to {
        detect_engine(Some(target_fmt), None)?
    } else {
        first_engine
    };

    let mut dest = Buffer::new();
    let options = FormatOptions {
        pretty: !args.compact,
        indent: args.indent,
    };
    dst_engine
        .serialize(&base_val, &mut dest, &options)
        .map_err(|e| CliError::Format(format!("failed to serialize merged document: {}", e)))?;
    let out_bytes = dest.into_vec();

    write_output(&out_bytes, args.output.as_deref(), dst_engine.is_binary())
}

fn cmd_schema(args: SchemaArgs) -> Result<(), CliError> {
    let (input_bytes, ext) = read_input(&args.input)?;
    let src_engine = detect_engine(args.from.as_deref(), ext.as_deref())?;
    let val = src_engine
        .parse_bytes(&input_bytes)
        .map_err(|e| CliError::Format(format!("failed to parse input: {}", e)))?;

    let schema_val = infer_schema(&val);
    let dst_engine = detect_engine(Some(&args.to), None)?;

    let mut dest = Buffer::new();
    let options = FormatOptions {
        pretty: true,
        indent: args.indent,
    };
    dst_engine
        .serialize(&schema_val, &mut dest, &options)
        .map_err(|e| CliError::Format(format!("failed to format schema: {}", e)))?;
    let out_bytes = dest.into_vec();

    write_output(&out_bytes, args.output.as_deref(), dst_engine.is_binary())
}

fn cmd_diff(args: DiffArgs) -> Result<(), CliError> {
    let (bytes1, ext1) = read_input(&args.file1)?;
    let (bytes2, ext2) = read_input(&args.file2)?;

    let engine1 = detect_engine(None, ext1.as_deref())?;
    let engine2 = detect_engine(None, ext2.as_deref())?;

    let val1 = engine1
        .parse_bytes(&bytes1)
        .map_err(|e| CliError::Format(format!("failed to parse '{}': {}", args.file1, e)))?;
    let val2 = engine2
        .parse_bytes(&bytes2)
        .map_err(|e| CliError::Format(format!("failed to parse '{}': {}", args.file2, e)))?;

    let json_engine = default_registry()
        .get("json")
        .ok_or_else(|| CliError::Format("json engine missing".to_string()))?;
    let mut dest = Buffer::new();

    let is_merge = args.merge || args.format == DiffFormat::Merge;
    if is_merge {
        let merge_patch = diff_merge_patch(&val1, &val2);
        json_engine
            .emit_pretty(&merge_patch, &mut dest, 2)
            .map_err(|e| CliError::Format(format!("serialization error: {}", e)))?;
    } else {
        let patch = diff(&val1, &val2);
        let patch_val = patch.to_value();
        json_engine
            .emit_pretty(&patch_val, &mut dest, 2)
            .map_err(|e| CliError::Format(format!("serialization error: {}", e)))?;
    }

    let out_bytes = dest.into_vec();
    write_output(&out_bytes, args.output.as_deref(), false)
}

fn cmd_patch(args: PatchArgs) -> Result<(), CliError> {
    let (input_bytes, ext) = read_input(&args.input)?;
    let (patch_bytes, patch_ext) = read_input(&args.patch)?;

    let engine = detect_engine(None, ext.as_deref())?;
    let patch_engine = detect_engine(None, patch_ext.as_deref())?;

    let mut target_val = engine
        .parse_bytes(&input_bytes)
        .map_err(|e| CliError::Format(format!("failed to parse '{}': {}", args.input, e)))?;
    let patch_val = patch_engine
        .parse_bytes(&patch_bytes)
        .map_err(|e| CliError::Format(format!("failed to parse '{}': {}", args.patch, e)))?;

    if args.merge {
        apply_merge_patch(&mut target_val, &patch_val);
    } else if let Ok(patch) = Patch::from_value(&patch_val) {
        patch
            .apply_inplace(&mut target_val)
            .map_err(|e| CliError::Format(format!("failed to apply patch: {}", e)))?;
    } else {
        apply_merge_patch(&mut target_val, &patch_val);
    }

    let mut dest = Buffer::new();
    engine
        .emit_pretty(&target_val, &mut dest, 2)
        .map_err(|e| CliError::Format(format!("serialization error: {}", e)))?;

    let out_bytes = dest.into_vec();
    write_output(&out_bytes, args.output.as_deref(), engine.is_binary())
}

fn cmd_validate(args: ValidateArgs) -> Result<(), CliError> {
    let (input_bytes, ext) = read_input(&args.input)?;
    let (schema_bytes, s_ext) = read_input(&args.schema)?;

    let engine = detect_engine(None, ext.as_deref())?;
    let schema_engine = detect_engine(None, s_ext.as_deref())?;

    let val = engine
        .parse_bytes(&input_bytes)
        .map_err(|e| CliError::Format(format!("failed to parse '{}': {}", args.input, e)))?;
    let schema_val = schema_engine
        .parse_bytes(&schema_bytes)
        .map_err(|e| CliError::Format(format!("failed to parse '{}': {}", args.schema, e)))?;

    let compiled = CompiledSchema::compile(&schema_val)
        .map_err(|e| CliError::Format(format!("failed to compile schema: {}", e)))?;

    match compiled.validate(&val) {
        Ok(()) => {
            println!(
                "VALID: '{}' conforms to schema '{}'",
                args.input, args.schema
            );
            Ok(())
        }
        Err(errors) => {
            let mut msg = format!(
                "INVALID: '{}' violates schema ({} errors):",
                args.input,
                errors.len()
            );
            for err in errors {
                msg.push_str(&format!(
                    "\n  - [{}] at '{}': {}",
                    err.keyword, err.pointer, err.message
                ));
            }
            Err(CliError::Validation(msg))
        }
    }
}

fn cmd_fmt(args: FmtArgs) -> Result<(), CliError> {
    let (input_bytes, ext) = read_input(&args.input)?;
    let engine = detect_engine(None, ext.as_deref())?;
    let val = engine
        .parse_bytes(&input_bytes)
        .map_err(|e| CliError::Format(format!("failed to parse '{}': {}", args.input, e)))?;

    let mut dest = Buffer::new();
    engine
        .emit_pretty(&val, &mut dest, args.indent)
        .map_err(|e| CliError::Format(format!("failed to format output: {}", e)))?;
    let out_bytes = dest.into_vec();

    if args.in_place && args.input != "-" {
        fs::write(&args.input, &out_bytes).map_err(|e| {
            CliError::Io(format!(
                "failed to write in-place to '{}': {}",
                args.input, e
            ))
        })?;
        println!("Formatted '{}'", args.input);
        Ok(())
    } else {
        write_output(&out_bytes, args.output.as_deref(), engine.is_binary())
    }
}

fn cmd_inspect(args: InspectArgs) -> Result<(), CliError> {
    let (input_bytes, ext) = read_input(&args.input)?;
    let engine = detect_engine(None, ext.as_deref())?;
    let val = engine
        .parse_bytes(&input_bytes)
        .map_err(|e| CliError::Format(format!("failed to parse '{}': {}", args.input, e)))?;

    println!("File:             {}", args.input);
    println!(
        "Detected Format:  {} ({})",
        engine.format_id(),
        engine.mime_type()
    );
    println!("Byte Size:        {} bytes", input_bytes.len());
    println!(
        "Root Type:        {}",
        match &val {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Bytes(_) => "bytes",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    );

    if let Value::Object(entries) = &val {
        println!("Key Count:        {}", entries.len());
    } else if let Value::Array(items) = &val {
        println!("Element Count:    {}", items.len());
    }

    fn calculate_depth(v: &Value) -> usize {
        match v {
            Value::Array(items) => 1 + items.iter().map(calculate_depth).max().unwrap_or(0),
            Value::Object(entries) => {
                1 + entries
                    .iter()
                    .map(|(_, val)| calculate_depth(val))
                    .max()
                    .unwrap_or(0)
            }
            _ => 1,
        }
    }

    println!("Max AST Depth:    {}", calculate_depth(&val));
    Ok(())
}

fn cmd_completions(args: CompletionsArgs) -> Result<(), CliError> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, bin_name, &mut io::stdout());
    Ok(())
}
