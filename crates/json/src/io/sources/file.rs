//! File Source for JSON
//!
//! Re-exports the unified `FileSource` from `babbel_core::io`.

pub use babbel_core::io::FileSource as File;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::traits::ISource;
    use std::fs;
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn create_test_file(content: &str) -> String {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let path = format!("test_json_file_src_{pid}_{id}.txt");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    fn cleanup_file(path: &str) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn create_source_file_works() {
        let path = create_test_file("i32e");
        let source = File::new(&path);
        assert!(source.is_ok());
        cleanup_file(&path);
    }

    #[test]
    fn create_source_non_existent_file_fails() {
        let result = File::new("non_existent_file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn create_source_empty_file_works() {
        let path = create_test_file("");
        let mut source = File::new(&path).unwrap();
        assert_eq!(source.current(), None);
        assert!(!source.more());
        cleanup_file(&path);
    }

    #[test]
    fn read_character_from_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        assert_eq!(source.current(), Some('i'));
        cleanup_file(&path);
    }

    #[test]
    fn move_to_next_character_in_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        source.next();
        assert_eq!(source.current(), Some('3'));
        cleanup_file(&path);
    }

    #[test]
    fn move_to_last_character_in_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        while source.more() {
            source.next();
        }
        assert_eq!(source.current(), None);
        cleanup_file(&path);
    }

    #[test]
    fn reset_in_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        while source.more() {
            source.next();
        }
        source.reset();
        assert_eq!(source.current(), Some('i'));
        cleanup_file(&path);
    }

    #[test]
    fn read_complete_file_content_matches() {
        let test_content = "i32e";
        let path = create_test_file(test_content);
        let mut source = File::new(&path).unwrap();
        let mut content = String::new();
        while source.more() {
            content.push(source.current().unwrap());
            source.next();
        }
        assert_eq!(content, test_content);
        cleanup_file(&path);
    }

    #[test]
    fn more_returns_true_when_file_has_content() {
        let path = create_test_file("abc");
        let mut source = File::new(&path).unwrap();
        assert!(source.more());
        cleanup_file(&path);
    }

    #[test]
    fn more_returns_false_after_exhausting_file() {
        let path = create_test_file("x");
        let mut source = File::new(&path).unwrap();
        source.next();
        assert!(!source.more());
        cleanup_file(&path);
    }

    #[test]
    fn next_past_end_does_not_panic() {
        let path = create_test_file("a");
        let mut source = File::new(&path).unwrap();
        source.next();
        source.next();
        assert_eq!(source.current(), None);
        assert!(!source.more());
        cleanup_file(&path);
    }

    #[test]
    fn unicode_multibyte_reading_works() {
        let path = create_test_file("🦀 Rust");
        let mut source = File::new(&path).unwrap();
        assert_eq!(source.current(), Some('🦀'));
        source.next();
        assert_eq!(source.current(), Some(' '));
        source.next();
        assert_eq!(source.current(), Some('R'));
        cleanup_file(&path);
    }
}
