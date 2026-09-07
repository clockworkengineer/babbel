//! Tests verifying integration of `babbel_core` streaming traits (`ISource`, `IDestination`, `IPositionAware`)
//! with `babbel_xml`.

use babbel_core::io::destinations::Buffer;
use babbel_core::io::sources::{BufferSource, StringSource};
use babbel_core::io::traits::{
    ICharStream, IClearable, IDestination, IPositionAware, IRewindable, ISource,
};
use babbel_xml::{
    canonicalize_to, parse_source, stringify_to, stringify_to_with_options, Document,
    SerializeOptions, XmlDestination, XmlSource,
};

#[test]
fn test_xml_source_implements_isource() {
    let mut xml_source = XmlSource::from_string("<root><child a=\"1\"/></root>");

    // 1. Test ISource
    {
        let source: &mut dyn ISource = &mut xml_source;
        assert_eq!(source.current(), Some('<'));
        assert!(source.more());
        source.next();
        assert_eq!(source.current(), Some('r'));
    }

    // 2. Test IPositionAware
    {
        let pos_aware: &dyn IPositionAware = &xml_source;
        assert_eq!(pos_aware.position(), 1);
    }

    // 3. Drain via ISource
    {
        let source: &mut dyn ISource = &mut xml_source;
        let mut collected = String::new();
        collected.push('<');
        collected.push('r');
        source.next();
        while source.more() {
            if let Some(ch) = source.current() {
                collected.push(ch);
            }
            source.next();
        }
        assert_eq!(collected, "<root><child a=\"1\"/></root>");
        assert!(!source.more());
        assert_eq!(source.current(), None);

        // Test reset
        source.reset();
        assert_eq!(source.current(), Some('<'));
    }

    // 4. Test ICharStream
    {
        let char_stream: &mut dyn ICharStream = &mut xml_source;
        assert_eq!(char_stream.current(), Some('<'));
        char_stream.next();
        assert_eq!(char_stream.current(), Some('r'));
    }

    // 5. Test IRewindable
    {
        let rewindable: &mut dyn IRewindable = &mut xml_source;
        rewindable.reset();
    }

    // 6. Final position check
    let pos_aware: &dyn IPositionAware = &xml_source;
    assert_eq!(pos_aware.position(), 0);
}

#[test]
fn test_xml_destination_implements_idestination() {
    let mut dest = XmlDestination::new();
    let idest: &mut dyn IDestination = &mut dest;

    idest.add_bytes("<greeting>");
    idest.add_byte(b'H');
    idest.add_bytes("i</greeting>");

    assert_eq!(idest.last(), Some(b'>'));
    assert_eq!(dest.as_str(), "<greeting>Hi</greeting>");

    // Test DestinationExt
    dest.write_str("<!-- comment -->");
    dest.write_char('\n');
    assert!(dest.as_str().contains("<!-- comment -->\n"));

    // Test IClearable
    let clearable: &mut dyn IClearable = &mut dest;
    clearable.clear();
    assert_eq!(dest.as_str(), "");
    assert_eq!(dest.last(), None);
}

#[test]
fn test_parse_from_arbitrary_isource() {
    let xml = "<config><timeout value=\"30\"/><retries>3</retries></config>";

    // 1. From XmlSource (as ISource)
    let mut xml_src = XmlSource::from_string(xml);
    let doc1 = parse_source(&mut xml_src).expect("should parse from XmlSource");
    assert_eq!(doc1.get_root_element_name(), Some("config"));

    // 2. From StringSource (babbel_core)
    let mut str_src = StringSource::new(xml);
    let doc2 = parse_source(&mut str_src).expect("should parse from StringSource");
    assert_eq!(doc2.get_root_element_name(), Some("config"));

    // 3. From BufferSource (babbel_core)
    let mut buf_src = BufferSource::new(xml.as_bytes());
    let doc3 = parse_source(&mut buf_src).expect("should parse from BufferSource");
    assert_eq!(doc3.get_root_element_name(), Some("config"));
}

#[test]
fn test_document_parse_source_and_stringify_to() {
    let xml = "<library><book id=\"1\"><title>Rust</title></book></library>";
    let mut src = StringSource::new(xml);

    let doc = Document::parse_source(&mut src).expect("parse_source failed");
    assert_eq!(doc.get_root_element_name(), Some("library"));

    let mut dest = Buffer::new();
    doc.stringify_to(&mut dest);
    let output = dest.to_string();
    assert!(output.contains("<book id=\"1\">"));
    assert!(output.contains("<title>Rust</title>"));
}

#[test]
fn test_stringify_to_destinations() {
    let doc = babbel_xml::parse("<data key=\"val\"><item>1</item><item>2</item></data>").unwrap();

    // 1. Write to XmlDestination
    let mut xml_dest = XmlDestination::new();
    stringify_to(&doc, &mut xml_dest);
    assert!(xml_dest.as_str().contains("<data key=\"val\">"));

    // 2. Write to babbel_core::Buffer
    let mut buf = Buffer::new();
    let mut opts = SerializeOptions::default();
    opts.pretty_print = false;
    stringify_to_with_options(&doc, &mut buf, &opts);
    let s = buf.to_string();
    assert!(s.contains("<item>1</item>"));
    assert!(s.contains("<item>2</item>"));
}

#[test]
fn test_canonicalize_to_destination() {
    let doc = babbel_xml::parse("<root b=\"2\" a=\"1\"><empty/></root>").unwrap();
    let mut dest = Buffer::new();

    canonicalize_to(&doc, &mut dest);
    assert_eq!(dest.to_string(), "<root a=\"1\" b=\"2\"><empty></empty></root>");
}

#[test]
fn test_streaming_round_trip() {
    let original = "<service name=\"auth\"><endpoint url=\"/login\" secure=\"true\"/></service>";

    // Ingest through BufferSource
    let mut in_buf = BufferSource::new(original.as_bytes());
    let doc = parse_source(&mut in_buf).expect("failed initial parse");

    // Emit through Buffer (IDestination)
    let mut out_buf = Buffer::new();
    let mut opts = SerializeOptions::default();
    opts.pretty_print = false;
    opts.omit_xml_declaration = true;
    stringify_to_with_options(&doc, &mut out_buf, &opts);

    // Re-ingest output buffer via BufferSource
    let mut re_in_buf = BufferSource::new(out_buf.as_bytes());
    let re_doc = parse_source(&mut re_in_buf).expect("failed re-parse");

    assert_eq!(re_doc.get_root_element_name(), Some("service"));
}
