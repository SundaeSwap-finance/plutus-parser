mod primitives;

#[cfg(feature = "derive")]
pub use plutus_parser_derive::*;

#[cfg(feature = "pallas-v0_32")]
pub use minicbor_v0_25 as minicbor;
#[cfg(feature = "pallas-v0_32")]
pub use pallas_v0_32::{
    BigInt, BoundedBytes, Constr, Hash, Int, KeyValuePairs, MaybeIndefArray, PlutusData,
};

#[cfg(feature = "pallas-v0_33")]
pub use minicbor_v0_25 as minicbor;
#[cfg(feature = "pallas-v0_33")]
pub use pallas_v0_33::{
    BigInt, BoundedBytes, Constr, Hash, Int, KeyValuePairs, MaybeIndefArray, PlutusData,
};

#[cfg(feature = "pallas-v0_34")]
pub use minicbor_v0_25 as minicbor;
#[cfg(feature = "pallas-v0_34")]
pub use pallas_v0_34::{
    BigInt, BoundedBytes, Constr, Hash, Int, KeyValuePairs, MaybeIndefArray, PlutusData,
};

#[cfg(feature = "pallas-v1")]
pub use minicbor_v0_26 as minicbor;
#[cfg(feature = "pallas-v1")]
pub use pallas_v1::{
    BigInt, BoundedBytes, Constr, Hash, Int, KeyValuePairs, MaybeIndefArray, PlutusData,
};

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
#[error("decode error at {path}: {kind}")]
pub struct DecodeError {
    kind: DecodeErrorKind,
    path: String,
}

impl DecodeError {
    pub fn new(kind: DecodeErrorKind) -> Self {
        Self {
            kind,
            path: String::new(),
        }
    }

    pub fn unexpected_variant(variant: u64) -> Self {
        Self::new(DecodeErrorKind::UnexpectedVariant { variant })
    }

    pub fn unexpected_type<E: Into<String>, A: Into<String>>(expected: E, actual: A) -> Self {
        Self::new(DecodeErrorKind::UnexpectedType {
            expected: expected.into(),
            actual: actual.into(),
        })
    }

    pub fn wrong_length(expected: usize, actual: usize) -> Self {
        Self::new(DecodeErrorKind::WrongLength { expected, actual })
    }

    pub fn wrong_tuple_field_count(expected: usize, actual: usize) -> Self {
        Self::new(DecodeErrorKind::WrongTupleFieldCount { expected, actual })
    }

    pub fn wrong_variant_field_count(variant: u64, expected: usize, actual: usize) -> Self {
        Self::new(DecodeErrorKind::WrongVariantFieldCount {
            variant,
            expected,
            actual,
        })
    }

    pub fn out_of_range(value: impl std::fmt::Display) -> Self {
        Self::new(DecodeErrorKind::OutOfRange {
            value: value.to_string(),
        })
    }

    pub fn invalid_cbor(error: minicbor::decode::Error) -> Self {
        Self::new(DecodeErrorKind::InvalidCbor(MinicborDecodeError(error)))
    }

    pub fn custom(message: impl Into<String>) -> Self {
        Self::new(DecodeErrorKind::Custom(message.into()))
    }

    pub fn with_field_name(mut self, name: impl std::fmt::Display) -> Self {
        if self.path.is_empty() {
            self.path = name.to_string();
        } else if self.path.starts_with("[") || self.path.starts_with(":") {
            self.path = format!("{name}{}", self.path);
        } else {
            self.path = format!("{name}.{}", self.path);
        }
        self
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DecodeErrorKind {
    #[error("unexpected variant {variant}")]
    UnexpectedVariant { variant: u64 },
    #[error("unexpected type (expected {expected}, found {actual})")]
    UnexpectedType { expected: String, actual: String },
    #[error("unexpected length for array (expected {expected}, found {actual})")]
    WrongLength { expected: usize, actual: usize },
    #[error("unexpected field count for tuple (expected {expected}, found {actual})")]
    WrongTupleFieldCount { expected: usize, actual: usize },
    #[error("unexpected field count for variant {variant} (expected {expected}, found {actual})")]
    WrongVariantFieldCount {
        variant: u64,
        expected: usize,
        actual: usize,
    },
    #[error("value {value} out of range")]
    OutOfRange { value: String },
    #[error("invalid cbor: {0}")]
    InvalidCbor(MinicborDecodeError),
    #[error("{0}")]
    Custom(String),
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct MinicborDecodeError(minicbor::decode::Error);
impl PartialEq for MinicborDecodeError {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_string() == other.0.to_string()
    }
}
impl Eq for MinicborDecodeError {}

pub trait AsPlutus: Sized {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError>;
    fn to_plutus(self) -> PlutusData;

    fn from_plutus_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let data = minicbor::decode::<PlutusData>(bytes).map_err(DecodeError::invalid_cbor)?;
        Self::from_plutus(data)
    }

    fn to_plutus_bytes(self) -> Vec<u8> {
        let data = self.to_plutus();
        minicbor::to_vec(data).expect("infallible")
    }

    fn vec_from_plutus(data: PlutusData) -> Result<Vec<Self>, DecodeError> {
        let items = parse_array(data)?;
        items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                Self::from_plutus(item).map_err(|e| e.with_field_name(format!("[{index}]")))
            })
            .collect()
    }

    fn vec_to_plutus(value: Vec<Self>) -> PlutusData {
        create_array(value.into_iter().map(Self::to_plutus).collect())
    }

    fn array_from_plutus<const N: usize>(data: PlutusData) -> Result<[Self; N], DecodeError> {
        let items = parse_array(data)?;
        if items.len() != N {
            return Err(DecodeError::wrong_length(N, items.len()));
        }
        let result = items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                Self::from_plutus(item).map_err(|e| e.with_field_name(format!("[{index}]")))
            })
            .collect::<Result<Vec<Self>, DecodeError>>()?;

        result
            .try_into()
            .map_err(|e: Vec<_>| DecodeError::wrong_length(N, e.len()))
    }

    fn array_to_plutus<const N: usize>(value: [Self; N]) -> PlutusData {
        create_array(value.into_iter().map(Self::to_plutus).collect())
    }
}

pub fn parse_array(data: PlutusData) -> Result<Vec<PlutusData>, DecodeError> {
    let array = match data {
        PlutusData::Array(array) => array,
        other => {
            return Err(DecodeError::unexpected_type("Array", type_name(&other)));
        }
    };
    Ok(array.to_vec())
}

pub fn parse_tuple<const N: usize>(data: PlutusData) -> Result<[PlutusData; N], DecodeError> {
    let array = parse_array(data)?;
    array
        .try_into()
        .map_err(|f: Vec<PlutusData>| DecodeError::wrong_tuple_field_count(N, f.len()))
}

pub fn parse_constr(data: PlutusData) -> Result<(u64, Vec<PlutusData>), DecodeError> {
    let constr = match data {
        PlutusData::Constr(constr) => constr,
        other => {
            return Err(DecodeError::unexpected_type("Constr", type_name(&other)));
        }
    };
    let Some(variant) = constr.constructor_value() else {
        return Err(DecodeError::custom("value has invalid tag"));
    };
    Ok((variant, constr.fields.to_vec()))
}

pub fn parse_variant<const N: usize>(
    variant: u64,
    fields: Vec<PlutusData>,
) -> Result<[PlutusData; N], DecodeError> {
    fields
        .try_into()
        .map_err(|f: Vec<PlutusData>| DecodeError::wrong_variant_field_count(variant, N, f.len()))
}

pub fn parse_map(data: PlutusData) -> Result<Vec<(PlutusData, PlutusData)>, DecodeError> {
    let kvps = match data {
        PlutusData::Map(kvps) => kvps,
        other => {
            return Err(DecodeError::unexpected_type("Map", type_name(&other)));
        }
    };
    Ok(kvps.to_vec())
}

pub fn create_constr(variant: u64, fields: Vec<PlutusData>) -> PlutusData {
    let (tag, any_constructor) = match variant {
        0..=6 => (variant + 121, None),
        7..=127 => (variant + 1280 - 7, None),
        x => (102, Some(x)),
    };
    PlutusData::Constr(Constr {
        tag,
        any_constructor,
        fields: if !fields.is_empty() {
            MaybeIndefArray::Indef(fields)
        } else {
            MaybeIndefArray::Def(Vec::new())
        },
    })
}

pub fn create_array(fields: Vec<PlutusData>) -> PlutusData {
    PlutusData::Array(if !fields.is_empty() {
        MaybeIndefArray::Indef(fields)
    } else {
        MaybeIndefArray::Def(Vec::new())
    })
}

pub fn create_map(kvps: Vec<(PlutusData, PlutusData)>) -> PlutusData {
    PlutusData::Map(KeyValuePairs::Def(kvps))
}

pub(crate) fn type_name(data: &PlutusData) -> &str {
    match data {
        PlutusData::Array(_) => "Array",
        PlutusData::BigInt(_) => "BigInt",
        PlutusData::BoundedBytes(_) => "BoundedBytes",
        PlutusData::Constr(_) => "Constr",
        PlutusData::Map(_) => "Map",
    }
}
