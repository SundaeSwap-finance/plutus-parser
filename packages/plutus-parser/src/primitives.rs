use crate::{
    AsPlutus, BigInt, BoundedBytes, DecodeError, PlutusData, create_array, create_constr,
    parse_constr, parse_tuple, parse_variant, type_name,
};

impl AsPlutus for BigInt {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
        let PlutusData::BigInt(int) = data else {
            return Err(DecodeError::UnexpectedType {
                expected: "BigInt".to_string(),
                actual: type_name(&data),
            });
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
            return Err(DecodeError::UnexpectedType {
                expected: "BoundedBytes".to_string(),
                actual: type_name(&data),
            });
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
        Err(DecodeError::UnexpectedVariant { variant })
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
            let PlutusData::BigInt(BigInt::Int(value)) = data else {
                return Err(DecodeError::UnexpectedType {
                    expected: "BigInt".into(),
                    actual: type_name(&data),
                });
            };
            let value: i128 = value.into();
            Ok(value as _)
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
        Ok(bytes.to_vec())
    }

    fn vec_to_plutus(value: Vec<Self>) -> PlutusData {
        let bytes = BoundedBytes::from(value);
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
    ($($param:ident),*) => {
        impl<$($param),*> AsPlutus for ($($param),*)
        where
            $($param: AsPlutus),*
        {
            #[allow(non_snake_case)]
            fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
                let [$($param),*] = parse_tuple(data)?;
                Ok(($(AsPlutus::from_plutus($param)?),*))
            }

            #[allow(non_snake_case)]
            fn to_plutus(self) -> PlutusData {
                let ($($param),*) = self;
                create_array(vec![$($param.to_plutus()),*])
            }
        }
    };
}

impl_tuple!(T1, T2);
impl_tuple!(T1, T2, T3);
impl_tuple!(T1, T2, T3, T4);
impl_tuple!(T1, T2, T3, T4, T5);
impl_tuple!(T1, T2, T3, T4, T5, T6);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);

impl AsPlutus for String {
    fn from_plutus(data: PlutusData) -> Result<Self, DecodeError> {
        let bytes = BoundedBytes::from_plutus(data)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|err| DecodeError::Custom(format!("error decoding string: {err}")))
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
        Err(DecodeError::UnexpectedVariant { variant })
    }

    fn to_plutus(self) -> PlutusData {
        match self {
            Some(value) => create_constr(0, vec![value.to_plutus()]),
            None => create_constr(1, vec![]),
        }
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
