//! TOML v1.0.0 Conformance test runner.

use babbel_toml::{from_str, to_string};

#[test]
fn test_conformance_valid_specs() {
    let valid_cases = [
        // 1. Bare keys and dotted keys
        (
            "bare_and_dotted",
            r#"
key = "value"
bare_key = "value"
bare-key = "value"
1234 = "value"
a.b.c = "nested"
"quoted.key" = "quoted"
'literal.key' = "literal"
"#,
        ),
        // 2. String escapes
        (
            "strings_escaped",
            r#"
str1 = "I'm a string. \"You can quote me\"."
str2 = "Name\tJos\u00E9\nLoc\tSF."
str3 = """
Roses are red
Violets are blue"""
str4 = """\
    The quick brown \
    fox jumps over \
    the lazy dog.\
    """
lit1 = 'C:\Users\nodejs\templates'
lit2 = '''
[client]
name = 'test'
'''
"#,
        ),
        // 3. Numbers and floats
        (
            "numbers",
            r#"
int1 = +99
int2 = 42
int3 = 0
int4 = -17
int5 = 1_000
hex = 0x01234567
oct = 0o755
bin = 0b11010110

flt1 = +1.0
flt2 = 3.1415
flt3 = -0.01
flt4 = 5e+22
flt5 = 1e06
flt6 = -2E-2
flt7 = 6.626e-34
flt8 = 224_617.445_991_228
inf_val = inf
nan_val = nan
"#,
        ),
        // 4. Datetimes
        (
            "datetimes",
            r#"
odt1 = 1979-05-27T07:32:00Z
odt2 = 1979-05-27T00:32:00-07:00
odt3 = 1979-05-27T07:32:00.999999Z
ldt1 = 1979-05-27T07:32:00
ldt2 = 1979-05-27T07:32:00.999999
ld1 = 1979-05-27
lt1 = 07:32:00
lt2 = 00:32:00.999999
"#,
        ),
        // 5. Arrays and heterogeneous arrays
        (
            "arrays",
            r#"
integers = [ 1, 2, 3 ]
colors = [ "red", "yellow", "green" ]
nested_arrays_of_ints = [ [ 1, 2 ], [3, 4, 5] ]
nested_mixed_array = [ [ 1, 2 ], ["a", "b", "c"] ]
string_array = [ "all", 'strings', """are the same""", '''type''' ]
contributors = [
  "Foo Bar <foo@example.com>",
  { name = "Baz Qux", email = "bazqux@example.com", url = "https://example.com/bazqux" }
]
"#,
        ),
        // 6. Tables and hierarchy
        (
            "tables",
            r#"
[table-1]
key1 = "some string"
key2 = 123

[table-2]
key1 = "another string"
key2 = 456

[dog."tater.man"]
type.name = "pug"

[a.b.c]
d = "val"
"#,
        ),
        // 7. Array of tables
        (
            "array_of_tables",
            r#"
[[products]]
name = "Hammer"
sku = 738594937

[[products]]

[[products]]
name = "Nail"
sku = 284758393
color = "gray"

[[fruit]]
  name = "apple"
  [fruit.physical]
    color = "red"
    shape = "round"

  [[fruit.variety]]
    name = "red delicious"

  [[fruit.variety]]
    name = "granny smith"

[[fruit]]
  name = "banana"
  [[fruit.variety]]
    name = "plantain"
"#,
        ),
        // 8. TOML 1.1.0 Features
        (
            "toml_1_1_escapes",
            r#"
ansi = "\e[32mOK\e[0m"
hex_esc = "\x41\x42\x43"
"#,
        ),
        (
            "toml_1_1_inline_tables",
            r#"
contact = {
    personal = {
        name = "Donald Duck",
        email = "donald@duckburg.com",
    },
    work = {
        name = "Coin cleaner",
        email = "donald@ScroogeCorp.com",
    },
}
"#,
        ),
        (
            "toml_1_1_datetimes",
            r#"
odt = 1979-05-27 07:32Z
ldt = 1979-05-27 07:32
lt = 07:32
"#,
        ),
    ];

    for (name, toml_str) in valid_cases {
        let node = from_str(toml_str)
            .unwrap_or_else(|e| panic!("Valid test case '{}' failed parsing: {}", name, e));
        assert!(node.is_table(), "Root for '{}' must be table", name);

        // Test round-trip
        let serialized = to_string(&node)
            .unwrap_or_else(|e| panic!("Serialization failed for '{}': {}", name, e));
        let roundtrip_node = from_str(&serialized)
            .unwrap_or_else(|e| panic!("Round-trip parsing failed for '{}': {}\nSerialized:\n{}", name, e, serialized));
        assert!(roundtrip_node.is_table());
    }
}

#[test]
fn test_conformance_invalid_rejections() {
    let invalid_cases = [
        ("unclosed_string", "key = \"unclosed"),
        ("unclosed_multiline", "key = \"\"\"unclosed"),
        ("unclosed_table", "[table"),
        ("unclosed_array_of_tables", "[[table]"),
        ("unclosed_array", "arr = [1, 2, "),
        ("unclosed_inline_table", "tbl = { a = 1, "),
        ("missing_value", "key = "),
        ("missing_key", " = 42"),
    ];

    for (name, toml_str) in invalid_cases {
        let result = from_str(toml_str);
        assert!(
            result.is_err(),
            "Invalid test case '{}' was unexpectedly accepted:\n{}",
            name,
            toml_str
        );
    }
}
