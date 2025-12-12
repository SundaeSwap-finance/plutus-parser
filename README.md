# plutus-parser

A simple crate to derive `from_plutus` and `to_plutus` on Rust types.

This crate matches Aiken semantics as closely as possible.

## API

Derive `AsPlutus` on your types, no mandatory annotations.
```rs
use plutus_parser::{AsPlutus, BigInt, BoundedBytes, Constr, Int, MaybeIndefArray, PlutusData};

#[derive(AsPlutus, Debug, PartialEq, Eq)]
pub struct MyType {
    pub token_name: String,
    pub token_value: u64,
    pub block_ref: Option<BlockReference>,
}

#[derive(AsPlutus, Debug, PartialEq, Eq)]
pub enum BlockReference {
    Origin,
    Point(Vec<u8>),
}
```

This library derives a set of conventions:
 - Structs convert to a `PlutusData::Constr` using variant `0`. You can add `#[variant = 2]` to use a different variant.
 - Enums convert to a `PlutusData::Constr`. The first variant uses variant 0, the second uses variant 1, nad so on. You can add `#[variant = 2]` to an individual variant to override this.
 - Tuples convert to an `PlutusData::Array`.
 - Arrays and vectors both convert to a `PlutusData::Array`, except for `Vec<u8>` which converts to a `PlutusData::BoundedBytes`.
 - Numeric fields convert to a `PlutusData::Integer`.
 - `bool` fields convert to a `PlutusData::Constr` enum, following Aiken semantics: `false` uses variant `0`, `true` uses variant `1`.
 - `Option<T>` fields convert to a `PlutusData::Constr` enum, following Aiken semantics: `Some(x)` uses variant `0`, `None` uses variant `1`.
 - `String` fields convert to a `PlutusData::BoundedBytes` containing the string contents.
 - `Vec<u8>` fields convert to a `PlutusData::BoundedBytes`.

## Usage

Use `T::from_plutus` to convert a `PlutusData` instance into your type, and use `T::to_plutus` to convert your type into a `PlutusData` instance.

```rs
use plutus_parser::{BigInt, BoundedBytes, Constr, Int, MaybeIndefArray, PlutusData};

fn to_from_plutus() {
    let data = MyType {
        token_name: "SUNDAE".to_string(),
        token_value: 13379001,
        block_ref: Some(BlockReference::Point(vec![0x12, 0x24])),
    };

    let plutus = PlutusData::Constr(Constr {
        tag: 121,
        any_constructor: None,
        fields: MaybeIndefArray::Indef(vec![
            PlutusData::BoundedBytes(BoundedBytes::from("SUNDAE".as_bytes().to_vec())),
            PlutusData::BigInt(BigInt::Int(Int::from(13379001))),
            PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: MaybeIndefArray::Indef(vec![PlutusData::Constr(Constr {
                    tag: 122,
                    any_constructor: None,
                    fields: MaybeIndefArray::Indef(vec![PlutusData::BoundedBytes(
                        BoundedBytes::from(vec![0x12, 0x24]),
                    )]),
                })]),
            }),
        ]),
    });

    assert_eq!(MyType::from_plutus(plutus.clone()).unwrap(), data);
    assert_eq!(data.to_plutus(), plutus);
```

To avoid a direct `pallas` dependency, you can also use `T::from_plutus_bytes` and `T::to_plutus_bytes` to work directly with bytes.
```rs
fn to_from_plutus_bytes() {
    let data = MyType {
        token_name: "SUNDAE".to_string(),
        token_value: 13379001,
        block_ref: Some(BlockReference::Point(vec![0x12, 0x24])),
    };

    let plutus_bytes: Vec<u8> =
        hex::decode("d8799f4653554e4441451a00cc25b9d8799fd87a9f421224ffffff").unwrap();

    assert_eq!(MyType::from_plutus_bytes(&plutus_bytes).unwrap(), data);
    assert_eq!(data.to_plutus_bytes(), plutus_bytes);
}
```