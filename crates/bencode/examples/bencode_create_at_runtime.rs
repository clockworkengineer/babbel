//! This example demonstrates how to create a complex Bencode data structure at runtime.
//! It shows various nested structures commonly found in torrent files.

use bencode_lib::nodes::node::Node;
use bencode_lib::make_node;
use std::collections::HashMap;

/// Creates a complex Bencode tree structure that resembles a typical torrent file,
/// including announce information, file lists, and nested metadata.
fn main() {
    // Root dictionary
    let mut root = HashMap::new();

    // Simple string and integer nodes
    root.insert("announce".to_string(), make_node("udp://tracker.example.com:80"));
    root.insert("created by".to_string(), make_node("AI Assistant Example Generator"));
    root.insert("creation date".to_string(), make_node(1_725_000_000));

    // Announce list (list of lists)
    let announce_list = make_node(vec![
        make_node(vec![
            make_node("udp://tracker.example.com:80"),
            make_node("http://tracker.example.com/announce"),
        ]),
        make_node(vec![
            make_node("udp://backup-tracker.example.org:1337"),
        ]),
    ]);
    root.insert("announce-list".to_string(), announce_list);

    // Info dictionary (nested)  
    root.insert("info".to_string(), build_info_dict());

    // Optional: top-level "url-list" as a list or single string
    root.insert(
        "url-list".to_string(),
        make_node(vec![
            make_node("https://mirror1.example.org/file"),
            make_node("https://mirror2.example.org/file"),
        ]),
    );

    // Optional: top-level comment
    root.insert("comment".to_string(), make_node("Multi-file torrent with complex structure"));

    // Wrap into the root Node
    let tree = Node::Dictionary(root);

    // For demonstration, pretty-print the structure
    println!("{:#?}", tree);
}

/// Builds the 'info' dictionary section of the torrent structure.
/// Contains essential torrent metadata like name, piece length, and file information.
fn build_info_dict() -> Node {
    let mut info = HashMap::new();

    // Common fields
    info.insert("name".to_string(), make_node("example-project"));
    info.insert("piece length".to_string(), make_node(256 * 1024)); // 256 KiB

    // Pieces: normally a raw byte string; here a placeholder hex-like string
    info.insert(
        "pieces".to_string(),
        make_node("abcdef012345...abcdef012345"),
    );

    // Private flag
    info.insert("private".to_string(), make_node(1));

    // Multi-file mode: "files" is a list of dictionaries
    info.insert("files".to_string(), build_files_list());

    // Optional: 'meta' nested dictionary to show deeper nesting
    info.insert("meta".to_string(), build_meta_dict());

    Node::Dictionary(info)
}

/// Creates a list of file entries for a multi-file torrent.
/// Each file entry contains length, path, and optional attributes.
fn build_files_list() -> Node {
    let file1 = {
        let mut f = HashMap::new();
        f.insert("length".to_string(), make_node(1_048_576)); // 1 MiB
        f.insert(
            "path".to_string(),
            make_node(vec![
                make_node("src"),
                make_node("main.rs"),
            ]),
        );
        f.insert(
            "md5sum".to_string(),
            make_node("d41d8cd98f00b204e9800998ecf8427e"),
        );
        Node::Dictionary(f)
    };

    let file2 = {
        let mut f = HashMap::new();
        f.insert("length".to_string(), make_node(2_621_440)); // 2.5 MiB
        f.insert(
            "path".to_string(),
            make_node(vec![
                make_node("assets"),
                make_node("images"),
                make_node("logo.png"),
            ]),
        );
        // Demonstrate optional per-file attributes
        f.insert(
            "attr".to_string(),
            Node::Dictionary({
                let mut attr = HashMap::new();
                attr.insert("read_only".to_string(), make_node(0));
                attr.insert("lang".to_string(), make_node("en-US"));
                attr
            }),
        );
        Node::Dictionary(f)
    };

    let file3 = {
        let mut f = HashMap::new();
        f.insert("length".to_string(), make_node(512_000)); // ~500 KiB
        f.insert(
            "path".to_string(),
            make_node(vec![
                make_node("docs"),
                make_node("guide.md"),
            ]),
        );
        Node::Dictionary(f)
    };

    make_node(vec![file1, file2, file3])
}

/// Constructs a deeply nested metadata dictionary containing
/// author information, tags, and build configuration details.  
fn build_meta_dict() -> Node {
    // Deeply nested structure to demonstrate complex trees:
    // meta -> authors (list of dicts), tags (list), build (dict -> list)
    let authors = make_node(vec![
        Node::Dictionary({
            let mut a = HashMap::new();
            a.insert("name".to_string(), make_node("Alice"));
            a.insert("email".to_string(), make_node("alice@example.com"));
            a
        }),
        Node::Dictionary({
            let mut a = HashMap::new();
            a.insert("name".to_string(), make_node("Bob"));
            a.insert("email".to_string(), make_node("bob@example.com"));
            a
        }),
    ]);

    let tags = make_node(vec![
        make_node("rust"),
        make_node("bencode"),
        make_node("example"),
    ]);

    let build = Node::Dictionary({
        let mut b = HashMap::new();
        b.insert(
            "targets".to_string(),
            make_node(vec![
                make_node("x86_64-unknown-linux-gnu"),
                make_node("aarch64-apple-darwin"),
            ]),
        );
        b.insert(
            "features".to_string(),
            make_node(vec![
                make_node("serde"),
                make_node("cli"),
            ]),
        );
        b
    });

    Node::Dictionary({
        let mut meta = HashMap::new();
        meta.insert("version".to_string(), make_node("1.2.3"));
        meta.insert("authors".to_string(), authors);
        meta.insert("tags".to_string(), tags);
        meta.insert("build".to_string(), build);
        meta
    })
}