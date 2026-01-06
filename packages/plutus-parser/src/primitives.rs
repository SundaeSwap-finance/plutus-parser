use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use indexmap::IndexMap;

use crate::{
    AsPlutus, BigInt, BoundedBytes, DecodeError, PlutusData, create_array, create_constr,
    create_map, parse_constr, parse_map, parse_tuple, parse_variant, type_name,
};

impl AsPlutus for PlutusData {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
        Ok(data)
    }

    fn to_plutus(self) -> PlutusData {
        self
    }
}

impl AsPlutus for BigInt {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
        let PlutusData::BigInt(int) = data else {
            return Err(DecodeError::unexpected_type("BigInt", type_name(&data)));
        };
        Ok(int)
    }

    fn to_plutus(self) -> PlutusData {
        PlutusData::BigInt(self)
    }
}

impl AsPlutus for BoundedBytes {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
        let PlutusData::BoundedBytes(bytes) = data else {
            return Err(DecodeError::unexpected_type(
                "BoundedBytes",
                type_name(&data),
            ));
        };
        Ok(bytes)
    }

    fn to_plutus(self) -> PlutusData {
        PlutusData::BoundedBytes(self)
    }
}

impl AsPlutus for bool {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
        let (variant, fields) = parse_constr(data)?;
        if variant == 0 {
            let [] = parse_variant(variant, fields)?;
            return Ok(false);
        }
        if variant == 1 {
            let [] = parse_variant(variant, fields)?;
            return Ok(true);
        }
        Err(DecodeError::unexpected_variant(variant))
    }

    fn to_plutus(self) -> PlutusData {
        match self {
            false => create_constr(0, vec![]),
            true => create_constr(1, vec![]),
        }
    }
}

macro_rules! impl_number {
    () => {
        fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
            let PlutusData::BigInt(value) = data else {
                return Err(DecodeError::unexpected_type("BigInt", type_name(&data)));
            };
            match value {
                BigInt::Int(value) => {
                    let value: i128 = value.into();
                    Self::try_from(value).map_err(|_| DecodeError::out_of_range(value))
                }
                BigInt::BigUInt(value) => Err(DecodeError::out_of_range(format!(
                    "0x{}",
                    hex::encode(&*value)
                ))),
                BigInt::BigNInt(value) => Err(DecodeError::out_of_range(format!(
                    "-1 - 0x{}",
                    hex::encode(&*value)
                ))),
            }
        }

        fn to_plutus(self) -> PlutusData {
            let val = self as i128;
            PlutusData::BigInt(BigInt::Int(val.try_into().unwrap()))
        }
    };
}

impl AsPlutus for u8 {
    impl_number!();

    // Vec<u8> should be BoundedBytes
    fn vec_from_plutus(data: PlutusData) -> Result<Vec<Self>, DecodeError> {
        let bytes = BoundedBytes::from_plutus(data)?;
        Ok(bytes.into())
    }

    fn vec_to_plutus(value: Vec<Self>) -> PlutusData {
        let bytes = BoundedBytes::from(value);
        PlutusData::BoundedBytes(bytes)
    }

    // [u8; N] should be BoundedBytes
    fn array_from_plutus<const N: usize>(data: PlutusData) -> Result<[Self; N], DecodeError> {
        let bytes = BoundedBytes::from_plutus(data)?;
        let vec: Vec<u8> = bytes.into();
        match vec.try_into() {
            Ok(array) => Ok(array),
            Err(v) => Err(DecodeError::wrong_length(N, v.len())),
        }
    }

    fn array_to_plutus<const N: usize>(value: [Self; N]) -> PlutusData {
        let bytes = BoundedBytes::from(value.to_vec());
        PlutusData::BoundedBytes(bytes)
    }
}
impl AsPlutus for u16 {
    impl_number!();
}
impl AsPlutus for u32 {
    impl_number!();
}
impl AsPlutus for u64 {
    impl_number!();
}
impl AsPlutus for i8 {
    impl_number!();
}
impl AsPlutus for i16 {
    impl_number!();
}
impl AsPlutus for i32 {
    impl_number!();
}
impl AsPlutus for i64 {
    impl_number!();
}

macro_rules! impl_tuple {
    ($($param:ident $index:expr),*) => {
        impl<$($param),*> AsPlutus for ($($param),*)
        where
            $($param: AsPlutus),*
        {
            #[allow(non_snake_case)]
            fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
                let [$($param),*] = parse_tuple(data)?;
                Ok(($(AsPlutus::from_plutus($param).map_err(|e| e.with_field_name($index))?),*))
            }

            #[allow(non_snake_case)]
            fn to_plutus(self) -> PlutusData {
                let ($($param),*) = self;
                create_array(vec![$($param.to_plutus()),*])
            }
        }
    };
}

impl_tuple!(T1 0, T2 1);
impl_tuple!(T1 0, T2 1, T3 2);
impl_tuple!(T1 0, T2 1, T3 2, T4 3);
impl_tuple!(T1 0, T2 1, T3 2, T4 3, T5 4);
impl_tuple!(T1 0, T2 1, T3 2, T4 3, T5 4, T6 5);
impl_tuple!(T1 0, T2 1, T3 2, T4 3, T5 4, T6 5, T7 6);
impl_tuple!(T1 0, T2 1, T3 2, T4 3, T5 4, T6 5, T7 6, T8 7);

impl AsPlutus for String {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
        let bytes = BoundedBytes::from_plutus(data)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|err| DecodeError::custom(format!("error decoding string: {err}")))
    }

    fn to_plutus(self) -> PlutusData {
        let bytes = BoundedBytes::from(self.into_bytes());
        bytes.to_plutus()
    }
}

impl<T: AsPlutus> AsPlutus for Option<T> {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
        let (variant, fields) = parse_constr(data)?;
        if variant == 0 {
            let [value] = parse_variant(variant, fields)?;
            return Ok(Some(T::from_plutus(value)?));
        }
        if variant == 1 {
            let [] = parse_variant(variant, fields)?;
            return Ok(None);
        }
        Err(DecodeError::unexpected_variant(variant))
    }

    fn to_plutus(self) -> PlutusData {
        match self {
            Some(value) => create_constr(0, vec![value.to_plutus()]),
            None => create_constr(1, vec![]),
        }
    }
}

impl<T: AsPlutus, const N: usize> AsPlutus for [T; N] {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
        T::array_from_plutus(data)
    }

    fn to_plutus(self) -> PlutusData {
        T::array_to_plutus(self)
    }
}

impl<T: AsPlutus> AsPlutus for Vec<T> {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
        T::vec_from_plutus(data)
    }

    fn to_plutus(self) -> PlutusData {
        T::vec_to_plutus(self)
    }
}

macro_rules! impl_map {
    () => {
        fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
            let mut map = Self::new();
            for (index, (key, value)) in parse_map(data)?.into_iter().enumerate() {
                let key = TKey::from_plutus(key)
                    .map_err(|e| e.with_field_name(format!("[(key #{index})]")))?;
                let value = TVal::from_plutus(value)
                    .map_err(|e| e.with_field_name(format!("[(value #{index})]")))?;
                map.insert(key, value);
            }
            Ok(map)
        }

        fn to_plutus(self) -> PlutusData {
            let kvps = self
                .into_iter()
                .map(|(k, v)| (k.to_plutus(), v.to_plutus()))
                .collect();
            create_map(kvps)
        }
    };
}

impl<TKey: AsPlutus + Hash + Eq, TVal: AsPlutus> AsPlutus for IndexMap<TKey, TVal> {
    impl_map!();
}

impl<TKey: AsPlutus + Hash + Eq, TVal: AsPlutus> AsPlutus for HashMap<TKey, TVal> {
    impl_map!();
}

impl<TKey: AsPlutus + PartialOrd + Ord, TVal: AsPlutus> AsPlutus for BTreeMap<TKey, TVal> {
    impl_map!();
}
