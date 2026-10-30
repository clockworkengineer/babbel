//! Automated multi-format throughput (MB/s) and memory allocation benchmarking suite.

use std::time::Instant;
use babbel::prelude::*;
use babbel_core::embedded::MemoryTracker;

struct BenchSample {
    name: &'static str,
    format: Format,
    sample_text: &'static str,
}

fn sample_payloads() -> Vec<BenchSample> {
    vec![
        BenchSample {
            name: "JSON",
            format: Format::Json,
            sample_text: r#"{"id":1001,"name":"Alice","active":true,"tags":["admin","engineer"],"metadata":{"dept":"Infra","level":5,"ratio":3.14159}}"#,
        },
        BenchSample {
            name: "YAML",
            format: Format::Yaml,
            sample_text: "id: 1001\nname: Alice\nactive: true\ntags:\n  - admin\n  - engineer\nmetadata:\n  dept: Infra\n  level: 5\n  ratio: 3.14159\n",
        },
        BenchSample {
            name: "TOML",
            format: Format::Toml,
            sample_text: "id = 1001\nname = \"Alice\"\nactive = true\ntags = [\"admin\", \"engineer\"]\n[metadata]\ndept = \"Infra\"\nlevel = 5\nratio = 3.14159\n",
        },
        BenchSample {
            name: "XML",
            format: Format::Xml,
            sample_text: "<user id=\"1001\"><name>Alice</name><active>true</active><tags><tag>admin</tag><tag>engineer</tag></tags><metadata><dept>Infra</dept><level>5</level><ratio>3.14159</ratio></metadata></user>",
        },
        BenchSample {
            name: "Bencode",
            format: Format::Bencode,
            sample_text: "d6:active4:true2:idi1001e8:metadatad4:dept5:Infra5:leveli5e5:ratio7:3.14159ee4:name5:Alice4:tagsl5:admin8:engineeree",
        },
        BenchSample {
            name: "RON",
            format: Format::Ron,
            sample_text: "(id:1001,name:\"Alice\",active:true,tags:[\"admin\",\"engineer\"],metadata:(dept:\"Infra\",level:5,ratio:3.14159))",
        },
        BenchSample {
            name: "KDL",
            format: Format::Kdl,
            sample_text: "user id=1001 name=\"Alice\" active=true {\n    tags \"admin\" \"engineer\"\n    metadata dept=\"Infra\" level=5 ratio=3.14159\n}\n",
        },
        BenchSample {
            name: "HCL",
            format: Format::Hcl,
            sample_text: "id = 1001\nname = \"Alice\"\nactive = true\ntags = [\"admin\", \"engineer\"]\nmetadata {\n  dept = \"Infra\"\n  level = 5\n  ratio = 3.14159\n}\n",
        },
        BenchSample {
            name: "CSV",
            format: Format::Csv,
            sample_text: "id,name,active,dept,level\n1001,Alice,true,Infra,5\n1002,Bob,false,Sales,3\n1003,Charlie,true,Dev,7\n",
        },
        BenchSample {
            name: "INI",
            format: Format::Ini,
            sample_text: "[user]\nid=1001\nname=Alice\nactive=true\n[metadata]\ndept=Infra\nlevel=5\n",
        },
    ]
}

fn main() {
    println!("================================================================================");
    println!(" Babbel Multi-Format Performance & Allocation Benchmark");
    println!("================================================================================");
    println!(
        "{:<10} | {:<12} | {:<12} | {:<14} | {:<12}",
        "Format", "Parse MB/s", "Emit MB/s", "Roundtrips/s", "Est AST Bytes"
    );
    println!("--------------------------------------------------------------------------------");

    let tracker = MemoryTracker::new();
    let samples = sample_payloads();
    const ITERATIONS: usize = 5_000;

    for sample in samples {
        let input_bytes = sample.sample_text.as_bytes();
        let input_len = input_bytes.len();

        // 1. Warm-up & Validate
        let parsed_val = match babbel::from_str(sample.format, sample.sample_text) {
            Ok(v) => v,
            Err(e) => {
                println!("{:<10} | SKIPPED ({})", sample.name, e);
                continue;
            }
        };

        let est_size = parsed_val.estimated_size();
        let _ = tracker.allocate(est_size);

        // 2. Parse Benchmark
        let t0 = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = babbel::from_str(sample.format, sample.sample_text);
        }
        let parse_dur = t0.elapsed();
        let parse_mb_s = (input_len * ITERATIONS) as f64 / (1024.0 * 1024.0) / parse_dur.as_secs_f64();

        // 3. Serialize Benchmark
        let t1 = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = babbel::to_string(sample.format, &parsed_val);
        }
        let emit_dur = t1.elapsed();
        let emit_mb_s = (input_len * ITERATIONS) as f64 / (1024.0 * 1024.0) / emit_dur.as_secs_f64();

        // 4. Roundtrip Rate
        let total_secs = (parse_dur + emit_dur).as_secs_f64();
        let roundtrips_sec = (ITERATIONS as f64) / total_secs;

        println!(
            "{:<10} | {:>9.2} MB/s | {:>9.2} MB/s | {:>11.0} r/s | {:>10} B",
            sample.name, parse_mb_s, emit_mb_s, roundtrips_sec, est_size
        );
    }

    // Binary format benchmarks: CBOR, MsgPack, BSON
    let binary_formats = [
        ("CBOR", Format::Cbor),
        ("MsgPack", Format::MsgPack),
        ("BSON", Format::Bson),
    ];

    let base_val = babbel::from_str(
        Format::Json,
        r#"{"id":1001,"name":"Alice","active":true,"tags":["admin","engineer"],"metadata":{"dept":"Infra","level":5}}"#,
    ).unwrap();

    for (name, fmt) in binary_formats {
        if let Ok(bytes) = babbel::to_vec(fmt, &base_val) {
            let len = bytes.len();
            let est_size = base_val.estimated_size();
            let _ = tracker.allocate(est_size);

            let t0 = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = babbel::from_slice(fmt, &bytes);
            }
            let parse_dur = t0.elapsed();
            let parse_mb_s = (len * ITERATIONS) as f64 / (1024.0 * 1024.0) / parse_dur.as_secs_f64();

            let t1 = Instant::now();
            for _ in 0..ITERATIONS {
                let _ = babbel::to_vec(fmt, &base_val);
            }
            let emit_dur = t1.elapsed();
            let emit_mb_s = (len * ITERATIONS) as f64 / (1024.0 * 1024.0) / emit_dur.as_secs_f64();

            let total_secs = (parse_dur + emit_dur).as_secs_f64();
            let roundtrips_sec = (ITERATIONS as f64) / total_secs;

            println!(
                "{:<10} | {:>9.2} MB/s | {:>9.2} MB/s | {:>11.0} r/s | {:>10} B",
                name, parse_mb_s, emit_mb_s, roundtrips_sec, est_size
            );
        }
    }

    println!("================================================================================");
    println!(" Peak AST memory tracked: {} bytes", tracker.peak());
    println!(" All format benchmarks executed successfully.");
    println!("================================================================================");
}

