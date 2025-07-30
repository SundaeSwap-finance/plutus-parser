mod primitives;

#[cfg(feature = "derive")]
pub use plutus_parser_derive::*;

#[cfg(feature = "pallas-v0_32")]
pub use pallas_v0_32::{
    BigInt, BoundedBytes, Constr, Int, KeyValuePairs, MaybeIndefArray, PlutusData,
};

#[cfg(feature = "pallas-v0_33")]
pub use pallas_v0_33::{
    BigInt, BoundedBytes, Constr, Int, KeyValuePairs, MaybeIndefArray, PlutusData,
};

#[cfg(feature = "pallas-v1")]
pub use pallas_v1::{
    BigInt, BoundedBytes, Constr, Int, KeyValuePairs, MaybeIndefArray, PlutusData,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("unexpected variant {variant}")]
    UnexpectedVariant { variant: u64 },
    #[error("unexpected type (expected {expected}, found {actual})")]
    UnexpectedType { expected: String, actual: String },
    #[error("unexpected field count for tuple (expected {expected}, found {actual})")]
    WrongTupleFieldCount { expected: usize, actual: usize },
    #[error("unexpected field count for variant {variant} (expected {expected}, found {actual})")]
    WrongVariantFieldCount {
        variant: u64,
        expected: usize,
        actual: usize,
    },
    #[error("{0}")]
    Custom(String),
}

pub trait AsPlutus: Sized {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError>;
    fn to_plutus(self) -> PlutusData;

    fn vec_from_plutus(data: PlutusData) -> Result<Vec<Self>, DecodeError> {
        let items = parse_array(data)?;
        items.into_iter().map(Self::from_plutus).collect()
    }

    fn vec_to_plutus(value: Vec<Self>) -> PlutusData {
        create_array(value.into_iter().map(Self::to_plutus).collect())
    }
}

pub fn parse_array(data: PlutusData) -> Result<Vec<PlutusData>, DecodeError> {
    let array = match data {
        PlutusData::Array(array) => array,
        other => {
            return Err(DecodeError::UnexpectedType {
                expected: "Array".to_string(),
                actual: type_name(&other),
            });
        }
    };
    Ok(array.to_vec())
}

pub fn parse_tuple<const N: usize>(data: PlutusData) -> Result<[PlutusData; N], DecodeError> {
    let array = parse_array(data)?;
    array
        .try_into()
        .map_err(|f: Vec<PlutusData>| DecodeError::WrongTupleFieldCount {
            expected: N,
            actual: f.len(),
        })
}

pub fn parse_constr(data: PlutusData) -> Result<(u64, Vec<PlutusData>), DecodeError> {
    let constr = match data {
        PlutusData::Constr(constr) => constr,
        other => {
            return Err(DecodeError::UnexpectedType {
                expected: "Constr".to_string(),
                actual: type_name(&other),
            });
        }
    };
    let Some(variant) = constr.constructor_value() else {
        return Err(DecodeError::Custom("value has invalid tag".to_string()));
    };
    Ok((variant, constr.fields.to_vec()))
}

pub fn parse_variant<const N: usize>(
    variant: u64,
    fields: Vec<PlutusData>,
) -> Result<[PlutusData; N], DecodeError> {
    fields
        .try_into()
        .map_err(|f: Vec<PlutusData>| DecodeError::WrongVariantFieldCount {
            variant,
            expected: N,
            actual: f.len(),
        })
}

pub fn parse_map(data: PlutusData) -> Result<Vec<(PlutusData, PlutusData)>, DecodeError> {
    let kvps = match data {
        PlutusData::Map(kvps) => kvps,
        other => {
            return Err(DecodeError::UnexpectedType {
                expected: "Map".to_string(),
                actual: type_name(&other),
            });
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

pub(crate) fn type_name(data: &PlutusData) -> String {
    match data {
        PlutusData::Array(_) => "Array",
        PlutusData::BigInt(_) => "BigInt",
        PlutusData::BoundedBytes(_) => "BoundedBytes",
        PlutusData::Constr(_) => "Constr",
        PlutusData::Map(_) => "Map",
    }
    .to_string()
}
