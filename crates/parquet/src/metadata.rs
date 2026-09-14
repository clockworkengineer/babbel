//! Parquet schema, metadata structures, and Thrift codec.

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::error::ParquetError;
use crate::thrift::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Boolean = 0,
    Int32 = 1,
    Int64 = 2,
    Int96 = 3,
    Float = 4,
    Double = 5,
    ByteArray = 6,
    FixedLenByteArray = 7,
}

impl Type {
    pub fn from_i32(val: i32) -> Result<Self, ParquetError> {
        match val {
            0 => Ok(Type::Boolean),
            1 => Ok(Type::Int32),
            2 => Ok(Type::Int64),
            3 => Ok(Type::Int96),
            4 => Ok(Type::Float),
            5 => Ok(Type::Double),
            6 => Ok(Type::ByteArray),
            7 => Ok(Type::FixedLenByteArray),
            _ => Err(ParquetError::UnsupportedType(alloc::format!("Type {}", val))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldRepetitionType {
    Required = 0,
    Optional = 1,
    Repeated = 2,
}

impl FieldRepetitionType {
    pub fn from_i32(val: i32) -> Result<Self, ParquetError> {
        match val {
            0 => Ok(FieldRepetitionType::Required),
            1 => Ok(FieldRepetitionType::Optional),
            2 => Ok(FieldRepetitionType::Repeated),
            _ => Err(ParquetError::UnsupportedType(alloc::format!("RepetitionType {}", val))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Plain = 0,
    PlainDictionary = 2,
    Rle = 3,
}

impl Encoding {
    pub fn from_i32(val: i32) -> Result<Self, ParquetError> {
        match val {
            0 => Ok(Encoding::Plain),
            2 => Ok(Encoding::PlainDictionary),
            3 => Ok(Encoding::Rle),
            _ => Ok(Encoding::Plain),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionCodec {
    Uncompressed = 0,
    Snappy = 1,
    Gzip = 2,
    Zstd = 6,
}

impl CompressionCodec {
    pub fn from_i32(val: i32) -> Result<Self, ParquetError> {
        match val {
            0 => Ok(CompressionCodec::Uncompressed),
            1 => Ok(CompressionCodec::Snappy),
            2 => Ok(CompressionCodec::Gzip),
            6 => Ok(CompressionCodec::Zstd),
            _ => Ok(CompressionCodec::Uncompressed),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaElement {
    pub type_: Option<Type>,
    pub repetition_type: Option<FieldRepetitionType>,
    pub name: String,
    pub num_children: Option<i32>,
    pub converted_type: Option<i32>, // e.g. 0 = UTF8
}

impl SchemaElement {
    pub fn encode(&self, w: &mut ThriftWriter) {
        w.write_struct_begin();
        if let Some(t) = self.type_ {
            w.write_field_header(1, TYPE_I32);
            w.write_i32(t as i32);
        }
        if let Some(r) = self.repetition_type {
            w.write_field_header(3, TYPE_I32);
            w.write_i32(r as i32);
        }
        w.write_field_header(4, TYPE_BINARY);
        w.write_string(&self.name);
        if let Some(c) = self.num_children {
            w.write_field_header(5, TYPE_I32);
            w.write_i32(c);
        }
        if let Some(ct) = self.converted_type {
            w.write_field_header(6, TYPE_I32);
            w.write_i32(ct);
        }
        w.write_struct_end();
    }

    pub fn decode(r: &mut ThriftReader) -> Result<Self, ParquetError> {
        r.read_struct_begin();
        let mut type_ = None;
        let mut repetition_type = None;
        let mut name = String::new();
        let mut num_children = None;
        let mut converted_type = None;

        loop {
            let (field_id, field_type) = r.read_field_header()?;
            if field_type == TYPE_STOP {
                break;
            }
            match field_id {
                1 => type_ = Some(Type::from_i32(r.read_i32()?)?),
                3 => repetition_type = Some(FieldRepetitionType::from_i32(r.read_i32()?)?),
                4 => name = r.read_string()?,
                5 => num_children = Some(r.read_i32()?),
                6 => converted_type = Some(r.read_i32()?),
                _ => r.skip(field_type)?,
            }
        }
        r.read_struct_end();

        Ok(Self {
            type_,
            repetition_type,
            name,
            num_children,
            converted_type,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnMetaData {
    pub type_: Type,
    pub encodings: Vec<Encoding>,
    pub path_in_schema: Vec<String>,
    pub codec: CompressionCodec,
    pub num_values: i64,
    pub total_uncompressed_size: i64,
    pub total_compressed_size: i64,
    pub data_page_offset: i64,
}

impl ColumnMetaData {
    pub fn encode(&self, w: &mut ThriftWriter) {
        w.write_struct_begin();
        w.write_field_header(1, TYPE_I32);
        w.write_i32(self.type_ as i32);

        w.write_field_header(2, TYPE_LIST);
        w.write_list_header(TYPE_I32, self.encodings.len());
        for enc in &self.encodings {
            w.write_i32(*enc as i32);
        }

        w.write_field_header(3, TYPE_LIST);
        w.write_list_header(TYPE_BINARY, self.path_in_schema.len());
        for p in &self.path_in_schema {
            w.write_string(p);
        }

        w.write_field_header(4, TYPE_I32);
        w.write_i32(self.codec as i32);

        w.write_field_header(5, TYPE_I64);
        w.write_i64(self.num_values);

        w.write_field_header(6, TYPE_I64);
        w.write_i64(self.total_uncompressed_size);

        w.write_field_header(7, TYPE_I64);
        w.write_i64(self.total_compressed_size);

        w.write_field_header(9, TYPE_I64);
        w.write_i64(self.data_page_offset);

        w.write_struct_end();
    }

    pub fn decode(r: &mut ThriftReader) -> Result<Self, ParquetError> {
        r.read_struct_begin();
        let mut type_ = Type::Int64;
        let mut encodings = Vec::new();
        let mut path_in_schema = Vec::new();
        let mut codec = CompressionCodec::Uncompressed;
        let mut num_values = 0;
        let mut total_uncompressed_size = 0;
        let mut total_compressed_size = 0;
        let mut data_page_offset = 0;

        loop {
            let (field_id, field_type) = r.read_field_header()?;
            if field_type == TYPE_STOP {
                break;
            }
            match field_id {
                1 => type_ = Type::from_i32(r.read_i32()?)?,
                2 => {
                    let (_, len) = r.read_list_header()?;
                    for _ in 0..len {
                        encodings.push(Encoding::from_i32(r.read_i32()?)?);
                    }
                }
                3 => {
                    let (_, len) = r.read_list_header()?;
                    for _ in 0..len {
                        path_in_schema.push(r.read_string()?);
                    }
                }
                4 => codec = CompressionCodec::from_i32(r.read_i32()?)?,
                5 => num_values = r.read_i64()?,
                6 => total_uncompressed_size = r.read_i64()?,
                7 => total_compressed_size = r.read_i64()?,
                9 => data_page_offset = r.read_i64()?,
                _ => r.skip(field_type)?,
            }
        }
        r.read_struct_end();

        Ok(Self {
            type_,
            encodings,
            path_in_schema,
            codec,
            num_values,
            total_uncompressed_size,
            total_compressed_size,
            data_page_offset,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnChunk {
    pub file_offset: i64,
    pub meta_data: Option<ColumnMetaData>,
}

impl ColumnChunk {
    pub fn encode(&self, w: &mut ThriftWriter) {
        w.write_struct_begin();
        w.write_field_header(2, TYPE_I64);
        w.write_i64(self.file_offset);
        if let Some(ref meta) = self.meta_data {
            w.write_field_header(3, TYPE_STRUCT);
            meta.encode(w);
        }
        w.write_struct_end();
    }

    pub fn decode(r: &mut ThriftReader) -> Result<Self, ParquetError> {
        r.read_struct_begin();
        let mut file_offset = 0;
        let mut meta_data = None;

        loop {
            let (field_id, field_type) = r.read_field_header()?;
            if field_type == TYPE_STOP {
                break;
            }
            match field_id {
                2 => file_offset = r.read_i64()?,
                3 => meta_data = Some(ColumnMetaData::decode(r)?),
                _ => r.skip(field_type)?,
            }
        }
        r.read_struct_end();

        Ok(Self {
            file_offset,
            meta_data,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowGroup {
    pub columns: Vec<ColumnChunk>,
    pub total_byte_size: i64,
    pub num_rows: i64,
}

impl RowGroup {
    pub fn encode(&self, w: &mut ThriftWriter) {
        w.write_struct_begin();
        w.write_field_header(1, TYPE_LIST);
        w.write_list_header(TYPE_STRUCT, self.columns.len());
        for col in &self.columns {
            col.encode(w);
        }
        w.write_field_header(2, TYPE_I64);
        w.write_i64(self.total_byte_size);
        w.write_field_header(3, TYPE_I64);
        w.write_i64(self.num_rows);
        w.write_struct_end();
    }

    pub fn decode(r: &mut ThriftReader) -> Result<Self, ParquetError> {
        r.read_struct_begin();
        let mut columns = Vec::new();
        let mut total_byte_size = 0;
        let mut num_rows = 0;

        loop {
            let (field_id, field_type) = r.read_field_header()?;
            if field_type == TYPE_STOP {
                break;
            }
            match field_id {
                1 => {
                    let (_, len) = r.read_list_header()?;
                    for _ in 0..len {
                        columns.push(ColumnChunk::decode(r)?);
                    }
                }
                2 => total_byte_size = r.read_i64()?,
                3 => num_rows = r.read_i64()?,
                _ => r.skip(field_type)?,
            }
        }
        r.read_struct_end();

        Ok(Self {
            columns,
            total_byte_size,
            num_rows,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileMetaData {
    pub version: i32,
    pub schema: Vec<SchemaElement>,
    pub num_rows: i64,
    pub row_groups: Vec<RowGroup>,
    pub created_by: Option<String>,
}

impl FileMetaData {
    pub fn encode(&self, w: &mut ThriftWriter) {
        w.write_struct_begin();
        w.write_field_header(1, TYPE_I32);
        w.write_i32(self.version);

        w.write_field_header(2, TYPE_LIST);
        w.write_list_header(TYPE_STRUCT, self.schema.len());
        for s in &self.schema {
            s.encode(w);
        }

        w.write_field_header(3, TYPE_I64);
        w.write_i64(self.num_rows);

        w.write_field_header(4, TYPE_LIST);
        w.write_list_header(TYPE_STRUCT, self.row_groups.len());
        for rg in &self.row_groups {
            rg.encode(w);
        }

        if let Some(ref cb) = self.created_by {
            w.write_field_header(6, TYPE_BINARY);
            w.write_string(cb);
        }

        w.write_struct_end();
    }

    pub fn decode(r: &mut ThriftReader) -> Result<Self, ParquetError> {
        r.read_struct_begin();
        let mut version = 1;
        let mut schema = Vec::new();
        let mut num_rows = 0;
        let mut row_groups = Vec::new();
        let mut created_by = None;

        loop {
            let (field_id, field_type) = r.read_field_header()?;
            if field_type == TYPE_STOP {
                break;
            }
            match field_id {
                1 => version = r.read_i32()?,
                2 => {
                    let (_, len) = r.read_list_header()?;
                    for _ in 0..len {
                        schema.push(SchemaElement::decode(r)?);
                    }
                }
                3 => num_rows = r.read_i64()?,
                4 => {
                    let (_, len) = r.read_list_header()?;
                    for _ in 0..len {
                        row_groups.push(RowGroup::decode(r)?);
                    }
                }
                6 => created_by = Some(r.read_string()?),
                _ => r.skip(field_type)?,
            }
        }
        r.read_struct_end();

        Ok(Self {
            version,
            schema,
            num_rows,
            row_groups,
            created_by,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataPageHeader {
    pub num_values: i32,
    pub encoding: Encoding,
    pub definition_level_encoding: Encoding,
    pub repetition_level_encoding: Encoding,
}

impl DataPageHeader {
    pub fn encode(&self, w: &mut ThriftWriter) {
        w.write_struct_begin();
        w.write_field_header(1, TYPE_I32);
        w.write_i32(self.num_values);
        w.write_field_header(2, TYPE_I32);
        w.write_i32(self.encoding as i32);
        w.write_field_header(3, TYPE_I32);
        w.write_i32(self.definition_level_encoding as i32);
        w.write_field_header(4, TYPE_I32);
        w.write_i32(self.repetition_level_encoding as i32);
        w.write_struct_end();
    }

    pub fn decode(r: &mut ThriftReader) -> Result<Self, ParquetError> {
        r.read_struct_begin();
        let mut num_values = 0;
        let mut encoding = Encoding::Plain;
        let mut def = Encoding::Rle;
        let mut rep = Encoding::Rle;

        loop {
            let (field_id, field_type) = r.read_field_header()?;
            if field_type == TYPE_STOP {
                break;
            }
            match field_id {
                1 => num_values = r.read_i32()?,
                2 => encoding = Encoding::from_i32(r.read_i32()?)?,
                3 => def = Encoding::from_i32(r.read_i32()?)?,
                4 => rep = Encoding::from_i32(r.read_i32()?)?,
                _ => r.skip(field_type)?,
            }
        }
        r.read_struct_end();

        Ok(Self {
            num_values,
            encoding,
            definition_level_encoding: def,
            repetition_level_encoding: rep,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageHeader {
    pub type_: i32, // 0 = DATA_PAGE
    pub uncompressed_page_size: i32,
    pub compressed_page_size: i32,
    pub data_page_header: Option<DataPageHeader>,
}

impl PageHeader {
    pub fn encode(&self, w: &mut ThriftWriter) {
        w.write_struct_begin();
        w.write_field_header(1, TYPE_I32);
        w.write_i32(self.type_);
        w.write_field_header(2, TYPE_I32);
        w.write_i32(self.uncompressed_page_size);
        w.write_field_header(3, TYPE_I32);
        w.write_i32(self.compressed_page_size);
        if let Some(ref d) = self.data_page_header {
            w.write_field_header(5, TYPE_STRUCT);
            d.encode(w);
        }
        w.write_struct_end();
    }

    pub fn decode(r: &mut ThriftReader) -> Result<Self, ParquetError> {
        r.read_struct_begin();
        let mut type_ = 0;
        let mut uncompressed_page_size = 0;
        let mut compressed_page_size = 0;
        let mut data_page_header = None;

        loop {
            let (field_id, field_type) = r.read_field_header()?;
            if field_type == TYPE_STOP {
                break;
            }
            match field_id {
                1 => type_ = r.read_i32()?,
                2 => uncompressed_page_size = r.read_i32()?,
                3 => compressed_page_size = r.read_i32()?,
                5 => data_page_header = Some(DataPageHeader::decode(r)?),
                _ => r.skip(field_type)?,
            }
        }
        r.read_struct_end();

        Ok(Self {
            type_,
            uncompressed_page_size,
            compressed_page_size,
            data_page_header,
        })
    }
}
