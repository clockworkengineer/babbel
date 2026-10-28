//! RFC 9535 JSONPath universal query engine.
//!
//! Provides query expression parsing, compilation, and evaluation over [`Value`] AST.

pub mod ast;
pub mod eval;
pub mod parser;
pub mod transform;

use crate::error::BabbelError;
use crate::model::Value;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
pub use ast::{ComparisonOp, FilterExpr, FilterOperand, LocatedNode, PathQuery, QueryPath, Segment};
pub use parser::JsonPathParser;
pub use transform::transform;

/// A compiled RFC 9535 JSONPath expression.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonPath {
    raw: String,
    path: QueryPath,
}

impl JsonPath {
    /// Parse and compile a JSONPath expression string into a reusable `JsonPath` instance.
    pub fn parse(expr: &str) -> Result<Self, BabbelError> {
        let mut parser = JsonPathParser::new(expr);
        let path = parser.parse()?;
        Ok(Self {
            raw: expr.to_string(),
            path,
        })
    }

    /// Query borrowed references from a `Value` root node.
    #[inline]
    pub fn query<'a>(&self, root: &'a Value) -> Vec<&'a Value> {
        eval::evaluate(&self.path, root)
    }

    /// Query matched nodes alongside their RFC 9535 normalized paths and RFC 6901 JSON Pointers.
    #[inline]
    pub fn query_located<'a>(&self, root: &'a Value) -> Vec<LocatedNode<'a>> {
        eval::evaluate_located(&self.path, root)
    }

    /// Query mutable references from a mutable `Value` root node.
    #[inline]
    pub fn query_mut<'a>(&self, root: &'a mut Value) -> Vec<&'a mut Value> {
        eval::evaluate_mut(&self.path, root)
    }

    /// Returns the raw JSONPath expression string.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Returns reference to internal `QueryPath` AST.
    #[inline]
    pub fn ast(&self) -> &QueryPath {
        &self.path
    }
}

/// Convenience function to compile and evaluate a JSONPath expression on a `Value` in one step.
pub fn query<'a>(root: &'a Value, expr: &str) -> Result<Vec<&'a Value>, BabbelError> {
    let path = JsonPath::parse(expr)?;
    Ok(path.query(root))
}

/// Convenience function to compile and evaluate a JSONPath expression returning located nodes.
pub fn query_located<'a>(
    root: &'a Value,
    expr: &str,
) -> Result<Vec<LocatedNode<'a>>, BabbelError> {
    let path = JsonPath::parse(expr)?;
    Ok(path.query_located(root))
}

/// Convenience function to compile and evaluate a mutable JSONPath expression on a `Value` in one step.
pub fn query_mut<'a>(root: &'a mut Value, expr: &str) -> Result<Vec<&'a mut Value>, BabbelError> {
    let path = JsonPath::parse(expr)?;
    Ok(path.query_mut(root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    fn sample_bookstore() -> Value {
        Value::Object(vec![(
            "store".to_string(),
            Value::Object(vec![
                (
                    "books".to_string(),
                    Value::Array(vec![
                        Value::Object(vec![
                            (
                                "category".to_string(),
                                Value::String("reference".to_string()),
                            ),
                            (
                                "author".to_string(),
                                Value::String("Nigel Rees".to_string()),
                            ),
                            (
                                "title".to_string(),
                                Value::String("Sayings of the Century".to_string()),
                            ),
                            ("price".to_string(), Value::Float(8.95)),
                            ("active".to_string(), Value::Bool(true)),
                        ]),
                        Value::Object(vec![
                            ("category".to_string(), Value::String("fiction".to_string())),
                            (
                                "author".to_string(),
                                Value::String("Evelyn Waugh".to_string()),
                            ),
                            (
                                "title".to_string(),
                                Value::String("Sword of Honour".to_string()),
                            ),
                            ("price".to_string(), Value::Float(12.99)),
                            ("active".to_string(), Value::Bool(false)),
                        ]),
                        Value::Object(vec![
                            ("category".to_string(), Value::String("fiction".to_string())),
                            (
                                "author".to_string(),
                                Value::String("Herman Melville".to_string()),
                            ),
                            ("title".to_string(), Value::String("Moby Dick".to_string())),
                            (
                                "isbn".to_string(),
                                Value::String("0-553-21311-3".to_string()),
                            ),
                            ("price".to_string(), Value::Float(8.99)),
                            ("active".to_string(), Value::Bool(true)),
                        ]),
                        Value::Object(vec![
                            ("category".to_string(), Value::String("fiction".to_string())),
                            (
                                "author".to_string(),
                                Value::String("J. R. R. Tolkien".to_string()),
                            ),
                            (
                                "title".to_string(),
                                Value::String("The Lord of the Rings".to_string()),
                            ),
                            (
                                "isbn".to_string(),
                                Value::String("0-395-19395-8".to_string()),
                            ),
                            ("price".to_string(), Value::Float(22.99)),
                            ("active".to_string(), Value::Bool(true)),
                        ]),
                    ]),
                ),
                (
                    "bicycle".to_string(),
                    Value::Object(vec![
                        ("color".to_string(), Value::String("red".to_string())),
                        ("price".to_string(), Value::Float(19.95)),
                    ]),
                ),
            ]),
        )])
    }

    #[test]
    fn test_dot_and_bracket_navigation() {
        let root = sample_bookstore();
        let bicycle_color = query(&root, "$.store.bicycle.color").unwrap();
        assert_eq!(bicycle_color.len(), 1);
        assert_eq!(bicycle_color[0].as_str(), Some("red"));

        let bicycle_color_bracket = query(&root, "$['store']['bicycle']['color']").unwrap();
        assert_eq!(bicycle_color_bracket.len(), 1);
        assert_eq!(bicycle_color_bracket[0].as_str(), Some("red"));
    }

    #[test]
    fn test_wildcard_and_array_indexing() {
        let root = sample_bookstore();
        let all_authors = query(&root, "$.store.books[*].author").unwrap();
        assert_eq!(all_authors.len(), 4);
        assert_eq!(all_authors[0].as_str(), Some("Nigel Rees"));
        assert_eq!(all_authors[3].as_str(), Some("J. R. R. Tolkien"));

        let first_author = query(&root, "$.store.books[0].author").unwrap();
        assert_eq!(first_author.len(), 1);
        assert_eq!(first_author[0].as_str(), Some("Nigel Rees"));

        let last_author = query(&root, "$.store.books[-1].author").unwrap();
        assert_eq!(last_author.len(), 1);
        assert_eq!(last_author[0].as_str(), Some("J. R. R. Tolkien"));
    }

    #[test]
    fn test_slice_operations() {
        let numbers = Value::Array(vec![
            Value::Integer(10),
            Value::Integer(20),
            Value::Integer(30),
            Value::Integer(40),
            Value::Integer(50),
        ]);

        let slice_0_2 = query(&numbers, "$[0:2]").unwrap();
        assert_eq!(slice_0_2.len(), 2);
        assert_eq!(slice_0_2[0].as_i64(), Some(10));
        assert_eq!(slice_0_2[1].as_i64(), Some(20));

        let slice_step = query(&numbers, "$[::2]").unwrap();
        assert_eq!(slice_step.len(), 3);
        assert_eq!(slice_step[0].as_i64(), Some(10));
        assert_eq!(slice_step[1].as_i64(), Some(30));
        assert_eq!(slice_step[2].as_i64(), Some(50));
    }

    #[test]
    fn test_recursive_descent() {
        let root = sample_bookstore();
        let all_titles = query(&root, "$..title").unwrap();
        assert_eq!(all_titles.len(), 4);

        let all_prices = query(&root, "$..price").unwrap();
        // 4 books + 1 bicycle = 5 prices
        assert_eq!(all_prices.len(), 5);
    }

    #[test]
    fn test_filter_expressions() {
        let root = sample_bookstore();

        // Books with price < 10
        let cheap_books = query(&root, "$.store.books[?(@.price < 10.0)].title").unwrap();
        assert_eq!(cheap_books.len(), 2);
        assert_eq!(cheap_books[0].as_str(), Some("Sayings of the Century"));
        assert_eq!(cheap_books[1].as_str(), Some("Moby Dick"));

        // Books with isbn property
        let books_with_isbn = query(&root, "$.store.books[?(@.isbn)].title").unwrap();
        assert_eq!(books_with_isbn.len(), 2);

        // Books where active is true and category == 'fiction'
        let active_fiction = query(
            &root,
            "$.store.books[?(@.active == true && @.category == 'fiction')].title",
        )
        .unwrap();
        assert_eq!(active_fiction.len(), 2);
        assert_eq!(active_fiction[0].as_str(), Some("Moby Dick"));
        assert_eq!(active_fiction[1].as_str(), Some("The Lord of the Rings"));
    }

    #[test]
    fn test_mutable_query() {
        let mut root = sample_bookstore();
        {
            let mut matches = query_mut(&mut root, "$.store.bicycle.color").unwrap();
            assert_eq!(matches.len(), 1);
            *matches[0] = Value::String("blue".to_string());
        }

        let updated_color = query(&root, "$.store.bicycle.color").unwrap();
        assert_eq!(updated_color[0].as_str(), Some("blue"));
    }

    #[test]
    fn test_mutable_query_with_filter() {
        let mut root = sample_bookstore();
        {
            let mut cheap = query_mut(&mut root, "$.store.books[?(@.price < 10.0)].price").unwrap();
            assert_eq!(cheap.len(), 2);
            for p in &mut cheap {
                **p = Value::Float(5.0);
            }
        }
        let still_under_6 = query(&root, "$.store.books[?(@.price < 6.0)].title").unwrap();
        assert_eq!(still_under_6.len(), 2);
    }

    #[test]
    fn test_rfc9535_functions_match_search_value() {
        let root = sample_bookstore();
        let matched = query(
            &root,
            "$.store.books[?match(@.author, '^Evelyn.*')].title",
        )
        .unwrap();
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].as_str(), Some("Sword of Honour"));

        let searched = query(
            &root,
            "$.store.books[?search(@.title, 'Lord|Dick')].author",
        )
        .unwrap();
        assert_eq!(searched.len(), 2);
        assert_eq!(searched[0].as_str(), Some("Herman Melville"));
        assert_eq!(searched[1].as_str(), Some("J. R. R. Tolkien"));

        let val_fn = query(
            &root,
            "$.store.books[?(value(@.category) == 'reference')].author",
        )
        .unwrap();
        assert_eq!(val_fn.len(), 1);
        assert_eq!(val_fn[0].as_str(), Some("Nigel Rees"));
    }

    #[test]
    fn test_located_query_and_union() {
        let root = sample_bookstore();
        let located = query_located(&root, "$.store.books[0, 2].title").unwrap();
        assert_eq!(located.len(), 2);
        assert_eq!(located[0].path, "$['store']['books'][0]['title']");
        assert_eq!(located[0].pointer, "/store/books/0/title");
        assert_eq!(located[0].value.as_str(), Some("Sayings of the Century"));
        assert_eq!(located[1].path, "$['store']['books'][2]['title']");
        assert_eq!(located[1].pointer, "/store/books/2/title");
        assert_eq!(located[1].value.as_str(), Some("Moby Dick"));
    }

    #[test]
    fn test_transform_pipeline() {
        let root = sample_bookstore();
        let avg_price = transform(&root, "$.store.books[*].price | sum").unwrap();
        assert!((avg_price.as_f64().unwrap() - 53.92).abs() < 1e-6);

        let sorted_titles = transform(
            &root,
            "$.store.books[*] | select(.category == 'fiction') | sort_by(.price) | map(.title)",
        )
        .unwrap();
        let arr = sorted_titles.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_str(), Some("Moby Dick"));
        assert_eq!(arr[1].as_str(), Some("Sword of Honour"));
        assert_eq!(arr[2].as_str(), Some("The Lord of the Rings"));
    }
}
