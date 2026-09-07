//! Comprehensive integration tests for Babbel Embedded Systems Support.
//!
//! Tests zero-allocation streaming pull parsing, stack-allocated destinations,
//! bounded memory tracking, and embedded limits.

use babbel::embedded::*;

#[test]
fn test_embedded_slice_destination_json() {
    let mut buffer = [0u8; 64];
    let mut dest = SliceDestination::new(&mut buffer);

    dest.add_bytes(r#"{"temp":24.5,"status":"ok"}"#);
    assert_eq!(dest.as_str().unwrap(), r#"{"temp":24.5,"status":"ok"}"#);
    assert_eq!(dest.len(), 27);
    assert!(!dest.is_truncated());
}

#[test]
fn test_embedded_array_vec_destination() {
    let mut dest = ArrayVecDestination::<128>::new();
    dest.add_bytes("sensor_id=0x42,reading=1013.25");

    assert_eq!(dest.as_str().unwrap(), "sensor_id=0x42,reading=1013.25");
    assert_eq!(dest.len(), 30);
    assert_eq!(dest.remaining(), 98);
}

#[test]
fn test_embedded_json_pull_parser() {
    let json = r#"{"sensor":"bmp280","press_hpa":1013.2,"valid":true}"#;
    let mut parser = JsonPullParser::new(json);

    assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::StartObject)));
    assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::Key("sensor"))));
    assert_eq!(
        parser.next_event(),
        Ok(Some(JsonPullEvent::Value(JsonScalar::String("bmp280"))))
    );

    assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::Key("press_hpa"))));
    let num_ev = parser.next_event().unwrap().unwrap();
    match num_ev {
        JsonPullEvent::Value(s) => {
            assert_eq!(s.to_f64(), Some(1013.2));
        }
        _ => panic!("expected number"),
    }

    assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::Key("valid"))));
    assert_eq!(
        parser.next_event(),
        Ok(Some(JsonPullEvent::Value(JsonScalar::Bool(true))))
    );
    assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::EndObject)));
    assert_eq!(parser.next_event(), Ok(None));
}

#[test]
fn test_embedded_csv_pull_parser() {
    let csv = "rpm,temp_c\n3000,85.2\n3200,86.1\n";
    let mut parser = CsvPullParser::new(csv, &babbel::CsvOptions::default());

    let header = parser.next_record().unwrap();
    let cols: Vec<&str> = header.fields().collect();
    assert_eq!(cols, vec!["rpm", "temp_c"]);

    let row1 = parser.next_record().unwrap();
    let fields1: Vec<&str> = row1.fields().collect();
    assert_eq!(fields1, vec!["3000", "85.2"]);

    let row2 = parser.next_record().unwrap();
    let fields2: Vec<&str> = row2.fields().collect();
    assert_eq!(fields2, vec!["3200", "86.1"]);

    assert!(parser.next_record().is_none());
}

#[test]
fn test_embedded_ini_pull_parser() {
    let ini = "[network]\nip=192.168.1.100\nport=80\n";
    let mut parser = IniPullParser::new(ini);

    assert_eq!(parser.next_event(), Some(IniEvent::Section("network")));
    assert_eq!(parser.next_event(), Some(IniEvent::Entry { key: "ip", val: "192.168.1.100" }));
    assert_eq!(parser.next_event(), Some(IniEvent::Entry { key: "port", val: "80" }));
    assert_eq!(parser.next_event(), None);
}

#[test]
fn test_embedded_stack_buffer() {
    let mut buf = StackBuffer::<32>::new();
    assert!(buf.push(b'A'));
    assert!(buf.extend_from_slice(b"BCDEF"));
    assert_eq!(buf.as_slice(), b"ABCDEF");
    assert_eq!(buf.len(), 6);
    assert_eq!(buf.remaining(), 26);
}

#[test]
fn test_embedded_memory_tracker() {
    let tracker = MemoryTracker::with_limit(256);
    assert!(tracker.allocate(128).is_ok());
    assert_eq!(tracker.current(), 128);
    assert_eq!(tracker.peak(), 128);
    assert!(tracker.allocate(200).is_err()); // 128 + 200 = 328 > 256
    tracker.deallocate(64);
    assert_eq!(tracker.current(), 64);
    assert_eq!(tracker.peak(), 128);
}
