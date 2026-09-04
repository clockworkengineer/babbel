use xml_lib_rust::{parse, XsdValidator};

#[test]
fn test_xsd_sequence_compositor_and_attributes() {
    let schema_xml = r#"<?xml version="1.0"?>
    <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
      <xs:element name="person">
        <xs:complexType>
          <xs:sequence>
            <xs:element name="first" type="xs:string"/>
            <xs:element name="last" type="xs:string"/>
          </xs:sequence>
          <xs:attribute name="id" type="xs:integer" use="required"/>
        </xs:complexType>
      </xs:element>
    </xs:schema>
    "#;

    let mut validator = XsdValidator::new();
    validator.parse_schema(schema_xml).expect("Parse schema");

    let valid_xml = r#"<person id="101"><first>John</first><last>Doe</last></person>"#;
    let doc_valid = parse(valid_xml).unwrap();
    assert!(validator.validate(&doc_valid).is_ok());

    // Missing required attribute "id"
    let missing_attr_xml = r#"<person><first>John</first><last>Doe</last></person>"#;
    let doc_missing_attr = parse(missing_attr_xml).unwrap();
    let res = validator.validate(&doc_missing_attr);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Required attribute 'id' missing"));

    // Missing sequence child <last>
    let missing_child_xml = r#"<person id="101"><first>John</first></person>"#;
    let doc_missing_child = parse(missing_child_xml).unwrap();
    let res2 = validator.validate(&doc_missing_child);
    assert!(res2.is_err());
    assert!(res2.unwrap_err().to_string().contains("missing required sequence child <last>"));
}

#[test]
fn test_xsd_choice_compositor() {
    let schema_xml = r#"<?xml version="1.0"?>
    <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
      <xs:element name="contact">
        <xs:complexType>
          <xs:choice>
            <xs:element name="email" type="xs:string"/>
            <xs:element name="phone" type="xs:string"/>
          </xs:choice>
        </xs:complexType>
      </xs:element>
    </xs:schema>
    "#;

    let mut validator = XsdValidator::new();
    validator.parse_schema(schema_xml).unwrap();

    let doc_email = parse("<contact><email>info@example.com</email></contact>").unwrap();
    assert!(validator.validate(&doc_email).is_ok());

    let doc_phone = parse("<contact><phone>+123456789</phone></contact>").unwrap();
    assert!(validator.validate(&doc_phone).is_ok());

    let doc_invalid = parse("<contact><fax>+123456789</fax></contact>").unwrap();
    let res = validator.validate(&doc_invalid);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("not allowed in choice group"));
}

#[test]
fn test_xsd_global_complex_type_reference() {
    let schema_xml = r#"<?xml version="1.0"?>
    <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
      <xs:complexType name="AddressType">
        <xs:sequence>
          <xs:element name="city" type="xs:string"/>
          <xs:element name="zip" type="xs:integer"/>
        </xs:sequence>
      </xs:complexType>

      <xs:element name="address" type="AddressType"/>
    </xs:schema>
    "#;

    let mut validator = XsdValidator::new();
    validator.parse_schema(schema_xml).unwrap();

    let doc_valid = parse("<address><city>Boston</city><zip>02101</zip></address>").unwrap();
    assert!(validator.validate(&doc_valid).is_ok());

    let doc_invalid = parse("<address><city>Boston</city><zip>not-a-number</zip></address>").unwrap();
    let res = validator.validate(&doc_invalid);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("not a valid integer"));
}

#[test]
fn test_xsd_min_max_occurs() {
    let schema_xml = r#"<?xml version="1.0"?>
    <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
      <xs:element name="basket">
        <xs:complexType>
          <xs:sequence>
            <xs:element name="apple" type="xs:string" minOccurs="1" maxOccurs="2"/>
          </xs:sequence>
        </xs:complexType>
      </xs:element>
    </xs:schema>
    "#;

    let mut validator = XsdValidator::new();
    validator.parse_schema(schema_xml).unwrap();

    // 1 apple - ok
    let doc_one = parse("<basket><apple>Fuji</apple></basket>").unwrap();
    assert!(validator.validate(&doc_one).is_ok());

    // 2 apples - ok
    let doc_two = parse("<basket><apple>Fuji</apple><apple>Gala</apple></basket>").unwrap();
    assert!(validator.validate(&doc_two).is_ok());

    // 0 apples - err (minOccurs = 1)
    let doc_zero = parse("<basket></basket>").unwrap();
    assert!(validator.validate(&doc_zero).is_err());

    // 3 apples - err (maxOccurs = 2)
    let doc_three = parse("<basket><apple>A</apple><apple>B</apple><apple>C</apple></basket>").unwrap();
    let res = validator.validate(&doc_three);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("exceeds maxOccurs (2)"));
}
