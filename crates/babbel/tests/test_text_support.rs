//! End-to-end tests for Babbel Text File Support (CSV, TSV, INI, JSON Lines, and Frontmatter).

use babbel::core::io::SliceSource;
use babbel::core::model::Value;
use babbel::core::{
    dedent, indent, parse_csv, parse_ini, sniff_delimiter, split_frontmatter, CsvOptions,
    FrontmatterFormat, IniOptions,
};

#[test]
fn test_frontmatter_and_text_utils() {
    let doc = "---\ntitle: Babbel Architecture\nversion: 2.0\n---\n# Babbel Core\n\nHigh speed polyglot processing.";
    let parsed = split_frontmatter(doc);
    assert_eq!(parsed.format, Some(FrontmatterFormat::Yaml));
    assert!(parsed.frontmatter.unwrap().contains("version: 2.0"));
    assert_eq!(parsed.content, "# Babbel Core\n\nHigh speed polyglot processing.");

    let toml_doc = "+++\napp = \"babbel\"\n+++\nBody";
    let parsed_toml = split_frontmatter(toml_doc);
    assert_eq!(parsed_toml.format, Some(FrontmatterFormat::Toml));

    let indented = indent("line1\nline2", "    ");
    assert_eq!(indented, "    line1\n    line2");

    let dedented = dedent("    line1\n    line2\n");
    assert_eq!(dedented, "line1\nline2\n");
}

#[test]
fn test_line_reader_streaming() {
    let text = "alpha\r\nbeta\ngamma\rdelta";
    let mut source = SliceSource::new(text);
    let mut lines = Vec::new();
    while let Some(line) = source.read_line() {
        lines.push(line);
    }
    assert_eq!(lines, vec!["alpha", "beta", "gamma", "delta"]);
}

#[test]
fn test_csv_tsv_parsing_and_emission() {
    let csv_data = "name,department,salary,active\nAlice,Engineering,125000,true\nBob,Design,95000.50,false\n";
    let parsed = parse_csv(csv_data, &CsvOptions::default()).expect("CSV parse");
    let rows = parsed.as_array().expect("rows array");
    assert_eq!(rows.len(), 2);

    let first = &rows[0];
    if let Value::Object(fields) = first {
        assert_eq!(fields[0].1, Value::String("Alice".into()));
        assert_eq!(fields[2].1, Value::Integer(125000));
        assert_eq!(fields[3].1, Value::Bool(true));
    } else {
        panic!("Expected Object row");
    }

    // TSV
    let tsv_data = "id\tval\n1\talpha\n2\tbeta\n";
    let tsv_parsed = parse_csv(tsv_data, &CsvOptions::tsv()).expect("TSV parse");
    assert_eq!(tsv_parsed.as_array().unwrap().len(), 2);

    // Sniffer
    assert_eq!(sniff_delimiter(csv_data), ',');
    assert_eq!(sniff_delimiter(tsv_data), '\t');
}

#[test]
fn test_ini_and_env_parsing() {
    let ini_doc = r#"
    [server]
    host = 127.0.0.1
    port = 8080
    workers = 4

    [logging]
    level = debug
    structured = true
    "#;

    let parsed = parse_ini(ini_doc, &IniOptions::default()).expect("INI parse");
    let obj = parsed.as_object().expect("root object");
    assert_eq!(obj.len(), 2);

    let env_doc = "PORT=5000\nDATABASE_URL=sqlite://data.db\nVERBOSE=true\n";
    let parsed_env = parse_ini(env_doc, &IniOptions::env()).expect("ENV parse");
    let env_obj = parsed_env.as_object().expect("env object");
    assert_eq!(env_obj[0], ("PORT".to_string(), Value::Integer(5000)));
    assert_eq!(env_obj[2], ("VERBOSE".to_string(), Value::Bool(true)));
}

#[cfg(feature = "convert")]
#[test]
fn test_conversions_csv_tsv_ini_jsonlines() {
    // 1. CSV -> JSON
    let csv = "id,name,score\n1,Alice,100\n2,Bob,95\n";
    let json = babbel::convert::csv_to_json(csv).expect("CSV -> JSON");
    assert!(json.contains("\"name\":\"Alice\"") || json.contains("\"name\": \"Alice\""));

    // 2. JSON -> CSV
    let csv_out = babbel::convert::json_to_csv(&json).expect("JSON -> CSV");
    assert!(csv_out.contains("Alice"));
    assert!(csv_out.contains("Bob"));

    // 3. TSV -> JSON & JSON -> TSV
    let tsv = "col1\tcol2\nx\t10\ny\t20\n";
    let json_tsv = babbel::convert::tsv_to_json(tsv).expect("TSV -> JSON");
    let tsv_out = babbel::convert::json_to_tsv(&json_tsv).expect("JSON -> TSV");
    assert!(tsv_out.contains("x\t10") || tsv_out.contains("x\t 10") || tsv_out.contains("col1\tcol2"));

    // 4. INI -> JSON & JSON -> INI
    let ini_src = "[app]\nname = Babbel\nversion = 1\n";
    let json_from_ini = babbel::convert::ini_to_json(ini_src).expect("INI -> JSON");
    let ini_out = babbel::convert::json_to_ini(&json_from_ini).expect("JSON -> INI");
    assert!(ini_out.contains("[app]"));
    assert!(ini_out.contains("name = Babbel") || ini_out.contains("name = \"Babbel\""));

    // 5. JSON Lines -> JSON & JSON -> JSON Lines
    let jsonl = "{\"id\":1}\n{\"id\":2}\n";
    let json_from_lines = babbel::convert::jsonlines_to_json(jsonl).expect("JSONL -> JSON");
    let jsonl_out = babbel::convert::json_to_jsonlines(&json_from_lines).expect("JSON -> JSONL");
    assert_eq!(jsonl_out, "{\"id\":1}\n{\"id\":2}\n");

    // 6. CSV -> JSON Lines
    let jsonl_from_csv = babbel::convert::csv_to_jsonlines(csv).expect("CSV -> JSONL");
    assert!(jsonl_from_csv.contains("\"name\":\"Alice\"") || jsonl_from_csv.contains("\"name\": \"Alice\""));
}
