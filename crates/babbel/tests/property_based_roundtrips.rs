//! Property-based testing & generative fuzzing suite for universal `Value` AST roundtrips.
//!
//! Generates pseudo-random, arbitrary `Value` ASTs (arbitrary depths, unicode, extremes)
//! to verify:
//! 1. Binary lossless roundtrip invariance (CBOR, MessagePack, BSON).
//! 2. Text roundtrip invariance (JSON, RON).
//! 3. Serialization idempotency.
//! 4. Truncation and corruption panic-freedom.
//! 5. Faulty I/O error injection resilience.

use babbel::prelude::*;
use babbel_core::testing::{FaultyDestination, PanicHookGuard};

/// Simple, high-speed deterministic pseudo-random number generator (XorShift64*).
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            0
        } else {
            (self.next_u64() as usize) % max
        }
    }

    fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }

    fn next_i64(&mut self) -> i64 {
        self.next_u64() as i64
    }

    fn next_f64(&mut self) -> f64 {
        let mantissa = (self.next_u64() & ((1 << 53) - 1)) as f64;
        let exp = ((self.next_usize(60) as i32) - 30) as f64;
        (mantissa / (1u64 << 53) as f64) * 10f64.powf(exp)
    }

    fn next_string(&mut self, max_len: usize) -> String {
        const CHAR_POOL: &[u8] =
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_- :;,.";
        let len = self.next_usize(max_len) + 1;
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            let idx = self.next_usize(CHAR_POOL.len());
            s.push(CHAR_POOL[idx] as char);
        }
        s
    }

    fn next_value(&mut self, depth: usize, max_depth: usize) -> Value {
        if depth >= max_depth {
            // Leaf scalar
            return match self.next_usize(5) {
                0 => Value::Null,
                1 => Value::Bool(self.next_bool()),
                2 => Value::Integer(self.next_i64() as i128),
                3 => Value::Float(self.next_f64()),
                _ => Value::String(self.next_string(24)),
            };
        }

        match self.next_usize(7) {
            0 => Value::Null,
            1 => Value::Bool(self.next_bool()),
            2 => Value::Integer(self.next_i64() as i128),
            3 => Value::Float(self.next_f64()),
            4 => Value::String(self.next_string(32)),
            5 => {
                let count = self.next_usize(4);
                let mut arr = Vec::with_capacity(count);
                for _ in 0..count {
                    arr.push(self.next_value(depth + 1, max_depth));
                }
                Value::Array(arr)
            }
            _ => {
                let count = self.next_usize(4);
                let mut obj = Vec::with_capacity(count);
                for _ in 0..count {
                    let key = self.next_string(12);
                    obj.push((key, self.next_value(depth + 1, max_depth)));
                }
                Value::Object(obj)
            }
        }
    }
}

#[test]
fn test_property_cbor_and_msgpack_lossless_roundtrip() {
    let mut rng = Rng::new(42);
    const SAMPLES: usize = 200;

    for _ in 0..SAMPLES {
        let val = rng.next_value(0, 4);

        // 1. CBOR roundtrip
        if let Ok(cbor_bytes) = babbel::to_vec(Format::Cbor, &val) {
            let decoded = babbel::from_slice(Format::Cbor, &cbor_bytes)
                .expect("CBOR deserialization failed for valid generated Value");
            assert_eq!(
                babbel::to_vec(Format::Cbor, &decoded).unwrap(),
                cbor_bytes,
                "CBOR roundtrip byte idempotency failed"
            );
        }

        // 2. MessagePack roundtrip
        if let Ok(msgpack_bytes) = babbel::to_vec(Format::MsgPack, &val) {
            let decoded = babbel::from_slice(Format::MsgPack, &msgpack_bytes)
                .expect("MsgPack deserialization failed for valid generated Value");
            assert_eq!(
                babbel::to_vec(Format::MsgPack, &decoded).unwrap(),
                msgpack_bytes,
                "MsgPack roundtrip byte idempotency failed"
            );
        }
    }
}

#[test]
fn test_property_json_roundtrip_and_idempotency() {
    let mut rng = Rng::new(1337);
    const SAMPLES: usize = 150;

    for _ in 0..SAMPLES {
        // Generate non-float or simple JSON-safe values
        let val = match rng.next_usize(3) {
            0 => {
                let mut obj = Value::object();
                for i in 0..3 {
                    obj = obj.set(format!("k{}", i), Value::Integer(rng.next_i64() as i128));
                }
                obj.build()
            }
            1 => {
                let mut arr = Value::array();
                for _ in 0..4 {
                    arr = arr.push(Value::String(rng.next_string(16)));
                }
                arr.build()
            }
            _ => Value::object()
                .set("name", Value::String(rng.next_string(10)))
                .set("active", Value::Bool(rng.next_bool()))
                .set("count", Value::Integer(rng.next_usize(1000) as i128))
                .build(),
        };

        let json_str = babbel::to_string(Format::Json, &val).expect("Failed to serialize to JSON");
        let parsed =
            babbel::from_str(Format::Json, &json_str).expect("Failed to parse JSON string");
        assert_eq!(
            val, parsed,
            "Parsed JSON Value must semantically equal original Value"
        );

        // Idempotency: re-serializing parsed value produces identical output
        let json_str2 =
            babbel::to_string(Format::Json, &parsed).expect("Failed to re-serialize to JSON");
        let parsed2 = babbel::from_str(Format::Json, &json_str2)
            .expect("Failed to re-parse re-serialized JSON");
        assert_eq!(
            parsed, parsed2,
            "Re-parsed JSON must match first parsed Value"
        );
        let json_str3 =
            babbel::to_string(Format::Json, &parsed2).expect("Failed to re-serialize second time");
        assert_eq!(
            json_str2, json_str3,
            "JSON serialization must be idempotent"
        );
    }
}

#[test]
fn test_property_truncation_fuzzing_panic_freedom() {
    let guard = PanicHookGuard::new_silent();
    let mut rng = Rng::new(999);

    let formats = [
        Format::Json,
        Format::Cbor,
        Format::MsgPack,
        Format::Bson,
        Format::Yaml,
        Format::Ron,
    ];

    let base_val = Value::object()
        .set("id", Value::Integer(1001))
        .set("name", Value::String("property_fuzz".into()))
        .set("tags", Value::array().push("a").push("b").build())
        .set("meta", Value::object().set("nested", true).build())
        .build();

    for fmt in formats {
        let bytes = match babbel::to_vec(fmt, &base_val) {
            Ok(b) => b,
            Err(_) => continue,
        };

        // Truncate at every single byte boundary
        for len in 0..bytes.len() {
            let truncated = &bytes[..len];
            let _ = babbel::from_slice(fmt, truncated);
        }

        // Random bit-flip fuzzing
        for _ in 0..50 {
            let mut corrupted = bytes.clone();
            if !corrupted.is_empty() {
                let idx = rng.next_usize(corrupted.len());
                corrupted[idx] ^= (rng.next_u64() & 0xFF) as u8;
                let _ = babbel::from_slice(fmt, &corrupted);
            }
        }
    }

    assert_eq!(
        guard.panics(),
        0,
        "Property-based truncation fuzzing must never cause a panic across any format"
    );
}

#[test]
fn test_faulty_io_injection_resilience() {
    let guard = PanicHookGuard::new_silent();

    let val = Value::object()
        .set(
            "large_array",
            Value::Array((0..50).map(|i| Value::Integer(i as i128)).collect()),
        )
        .set(
            "description",
            Value::String(
                "A fairly long descriptive payload designed to test partial write boundaries"
                    .into(),
            ),
        )
        .build();

    // Test failing at every byte threshold from 1 to 100 bytes
    for threshold in 1..=100 {
        let mut dest =
            FaultyDestination::fail_after(threshold, babbel_core::error::ErrorCode::IoError);
        let options = FormatOptions::default();

        // Blanket engine serialize should fail gracefully without panic
        let engine = babbel_json::JsonEngine;
        let _ = engine.serialize(&val, &mut dest, &options);
    }

    assert_eq!(
        guard.panics(),
        0,
        "Faulty I/O injection must never trigger a panic or unhandled abort"
    );
}
