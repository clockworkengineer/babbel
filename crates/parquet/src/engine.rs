//! Parquet Format Engine implementation adhering to OCP and DIP.

use babbel_core::{
    io::{IDestination, ISource},
    BabbelError, FormatEngine, FormatOptions, Value,
};

/// Apache Parquet format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy)]
pub struct ParquetEngine;

impl FormatEngine for ParquetEngine {
    fn format_id(&self) -> &'static str {
        "parquet"
    }

    fn mime_type(&self) -> &'static str {
        "application/vnd.apache.parquet"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["parquet", "pq"]
    }

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let bytes = babbel_core::io::read_all_bytes(source);
        self.parse_bytes(&bytes)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        crate::read_parquet(input)
            .map_err(|err| BabbelError::from(err).with_format("parquet"))
    }

    fn serialize(
        &self,
        value: &Value,
        dest: &mut dyn IDestination,
        _options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        let bytes = crate::write_parquet(value)
            .map_err(|err| BabbelError::from(err).with_format("parquet"))?;
        dest.add_raw_bytes(&bytes);
        Ok(())
    }
}
