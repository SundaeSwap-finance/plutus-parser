use std::collections::{BTreeMap, HashMap};

use plutus_parser::{
    AsPlutus, BigInt, BoundedBytes, Constr, DecodeError, Hash, KeyValuePairs, MaybeIndefArray,
    PlutusData, create_array, create_constr, create_map,
};
use plutus_parser_tests::{Interval, IntervalBound, IntervalBoundType};

fn assert_encoded<T: AsPlutus + std::fmt::Debug + Eq>(data: T, plutus: PlutusData) {
    assert_eq!(data, T::from_plutus(plutus.clone()).unwrap());
    assert_eq!(data.to_plutus(), plutus);
}

#[test]
fn should_support_simple_struct() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    pub struct SimpleStruct {
        bool_field: bool,
        u64_field: u64,
        bigint_field: BigInt,
        byte_field: BoundedBytes,
    }

    let data = SimpleStruct {
        bool_field: true,
        u64_field: 1337,
        bigint_field: BigInt::Int(9001.into()),
        byte_field: BoundedBytes::from(vec![0xca, 0xfe, 0xd0, 0x0d]),
    };
    let plutus = create_constr(
        0,
        vec![
            create_constr(1, vec![]),
            PlutusData::BigInt(BigInt::Int(1337.into())),
            PlutusData::BigInt(BigInt::Int(9001.into())),
            PlutusData::BoundedBytes(BoundedBytes::from(vec![0xca, 0xfe, 0xd0, 0x0d])),
        ],
    );
    assert_encoded(data, plutus);
}

#[test]
fn should_support_optionals() {
    assert_encoded(
        Some(1337),
        create_constr(0, vec![PlutusData::BigInt(BigInt::Int(1337.into()))]),
    );
    assert_encoded(None::<u64>, create_constr(1, vec![]));
}

#[test]
fn should_support_enums() {
    assert_encoded(
        IntervalBoundType::NegativeInfinity,
        create_constr(0, vec![]),
    );
    assert_encoded(
        IntervalBoundType::Finite(13),
        create_constr(1, vec![PlutusData::BigInt(BigInt::Int(13.into()))]),
    );
    assert_encoded(
        IntervalBoundType::PositiveInfinity,
        create_constr(2, vec![]),
    );
}

#[test]
fn should_support_nested_structs() {
    let data = Interval {
        lower_bound: IntervalBound {
            bound_type: IntervalBoundType::NegativeInfinity,
            is_inclusive: true,
        },
        upper_bound: IntervalBound {
            bound_type: IntervalBoundType::Finite(420),
            is_inclusive: false,
        },
    };
    let plutus = create_constr(
        0,
        vec![
            create_constr(0, vec![create_constr(0, vec![]), create_constr(1, vec![])]),
            create_constr(
                0,
                vec![
                    create_constr(1, vec![PlutusData::BigInt(BigInt::Int(420.into()))]),
                    create_constr(0, vec![]),
                ],
            ),
        ],
    );
    assert_encoded(data, plutus);
}

#[test]
fn should_support_tuple_structs() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Tuple(BoundedBytes, u64);

    let data = Tuple(BoundedBytes::from(vec![0x13, 0x37]), 9001);

    let plutus = create_constr(
        0,
        vec![
            PlutusData::BoundedBytes(BoundedBytes::from(vec![0x13, 0x37])),
            PlutusData::BigInt(BigInt::Int(9001.into())),
        ],
    );

    assert_encoded(data, plutus);
}

#[test]
fn should_support_tuples() {
    let data = (BoundedBytes::from(vec![0x13, 0x37]), 9001);

    let plutus = create_array(vec![
        PlutusData::BoundedBytes(BoundedBytes::from(vec![0x13, 0x37])),
        PlutusData::BigInt(BigInt::Int(9001.into())),
    ]);

    assert_encoded(data, plutus);
}

#[test]
fn should_support_arrays() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct HasArray {
        params: Vec<String>,
    }

    let data = HasArray {
        params: vec!["cafe".to_string()],
    };

    let plutus = create_constr(
        0,
        vec![PlutusData::Array(MaybeIndefArray::Indef(vec![
            PlutusData::BoundedBytes(BoundedBytes::from("cafe".bytes().collect::<Vec<_>>())),
        ]))],
    );

    assert_encoded(data, plutus);
}

#[test]
fn should_support_vec_u8_as_bytes() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct HasBytes {
        byte_vec: Vec<u8>,
        byte_struct: BoundedBytes,
    }

    let data = HasBytes {
        byte_vec: vec![0x69],
        byte_struct: BoundedBytes::from(vec![0x69]),
    };

    let plutus = create_constr(
        0,
        vec![
            PlutusData::BoundedBytes(BoundedBytes::from(vec![0x69])),
            PlutusData::BoundedBytes(BoundedBytes::from(vec![0x69])),
        ],
    );

    assert_encoded(data, plutus);
}

#[test]
fn should_support_hashes() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct HasHash {
        hash: Hash<28>,
    }

    let data = HasHash {
        hash: Hash::new([0x69; 28]),
    };

    let plutus = create_constr(
        0,
        vec![PlutusData::BoundedBytes(BoundedBytes::from(vec![0x69; 28]))],
    );

    assert_encoded(data, plutus);
}

#[test]
fn should_support_maps() {
    let mut data = BTreeMap::new();
    data.insert("bar".to_string(), 9001u64);
    data.insert("foo".to_string(), 1337u64);

    let plutus = create_map(vec![
        (
            PlutusData::BoundedBytes(BoundedBytes::from("bar".as_bytes().to_vec())),
            PlutusData::BigInt(BigInt::Int(9001.into())),
        ),
        (
            PlutusData::BoundedBytes(BoundedBytes::from("foo".as_bytes().to_vec())),
            PlutusData::BigInt(BigInt::Int(1337.into())),
        ),
    ]);

    assert_encoded(data, plutus);
}

#[test]
fn should_support_custom_variants_for_structs() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    #[plutus(constr = 2)]
    pub struct Special {
        is_special: bool,
    }

    let data = Special { is_special: true };
    let plutus = create_constr(2, vec![create_constr(1, vec![])]);

    assert_encoded(data, plutus);
}

#[test]
fn should_support_custom_variants_for_enums() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    pub enum Destination {
        #[plutus(constr = 1)]
        Self_,
    }

    let data = Destination::Self_;
    let plutus = create_constr(1, vec![]);

    assert_encoded(data, plutus);
}

#[test]
fn should_support_structs_as_lists() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    #[plutus(list)]
    struct IsList {
        is_array: bool,
        is_struct: bool,
    }

    let data = IsList {
        is_array: true,
        is_struct: false,
    };
    let plutus = create_array(vec![create_constr(1, vec![]), create_constr(0, vec![])]);

    assert_encoded(data, plutus);
}

#[test]
fn should_support_empty_structs_as_lists() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    #[plutus(list)]
    struct Listy;

    let data = Listy;
    let plutus = create_array(vec![]);

    assert_encoded(data, plutus);
}

#[test]
fn should_support_tuple_structs_as_lists() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    #[plutus(list)]
    struct ListTupley(String, u32);

    let data = ListTupley("foo".to_string(), 1337);
    let plutus = create_array(vec![
        PlutusData::BoundedBytes(BoundedBytes::from("foo".as_bytes().to_vec())),
        PlutusData::BigInt(BigInt::Int(1337.into())),
    ]);

    assert_encoded(data, plutus);
}

#[test]
fn should_match_plutus_conventions() {
    #[derive(AsPlutus, Clone)]
    pub enum Optional {
        Some(u32),
        None,
    }

    #[derive(AsPlutus, Clone)]
    pub struct Fields {
        pub a: u32,
        pub b: u32,
        pub c: Optional,
        pub d: Optional,
    }
    let data = Fields {
        a: 1,
        b: 2,
        c: Optional::Some(3),
        d: Optional::None,
    };
    let expected_bytes = hex::decode("d8799f0102d8799f03ffd87a80ff").unwrap();
    let mut enc_bytes = vec![];
    plutus_parser::minicbor::encode(data.to_plutus(), &mut enc_bytes).expect("infallible");

    assert_eq!(enc_bytes, expected_bytes);
}

#[test]
fn should_support_multiple_enum_fields() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    pub enum Gollum {
        Smeagol(bool, bool),
        Gandalf,
    }

    let data = Gollum::Smeagol(false, true);
    let plutus = create_constr(0, vec![create_constr(0, vec![]), create_constr(1, vec![])]);

    assert_encoded(data, plutus);
}

#[test]
fn should_encode_and_decode_bytes() {
    #[derive(AsPlutus, Clone, Debug, PartialEq, Eq)]
    struct Basic {
        name: String,
    }

    let data = Basic {
        name: "Hello world!".to_string(),
    };

    let bytes = data.clone().to_plutus_bytes();
    assert_eq!(hex::encode(&bytes), "d8799f4c48656c6c6f20776f726c6421ff");

    let data2 = Basic::from_plutus_bytes(&bytes).unwrap();
    assert_eq!(data, data2);
}

#[test]
fn should_support_plutus_data_fields() {
    #[derive(AsPlutus, Clone, Debug, PartialEq, Eq)]
    struct Basic {
        data: PlutusData,
    }

    let data = Basic {
        data: PlutusData::BoundedBytes(vec![0xca, 0xfe, 0xba, 0xbe].into()),
    };
    let plutus = create_constr(
        0,
        vec![PlutusData::BoundedBytes(
            vec![0xca, 0xfe, 0xba, 0xbe].into(),
        )],
    );

    assert_encoded(data, plutus);
}

#[test]
fn should_support_plutus_primitive_fields() {
    #[derive(AsPlutus, Clone, Debug, PartialEq, Eq)]
    struct Everything {
        constr: Constr<PlutusData>,
        map: KeyValuePairs<PlutusData, PlutusData>,
        array: MaybeIndefArray<PlutusData>,
    }

    let data = Everything {
        constr: Constr {
            tag: 1340,
            any_constructor: None,
            fields: MaybeIndefArray::Indef(vec![]),
        },
        map: KeyValuePairs::Def(vec![(
            PlutusData::BigInt(BigInt::Int(1.into())),
            PlutusData::BigInt(BigInt::Int(2.into())),
        )]),
        array: MaybeIndefArray::Def(vec![]),
    };
    let plutus = create_constr(
        0,
        vec![
            PlutusData::Constr(data.constr.clone()),
            PlutusData::Map(data.map.clone()),
            PlutusData::Array(data.array.clone()),
        ],
    );

    assert_encoded(data, plutus);
}

#[test]
fn should_support_array_fields() {
    #[derive(AsPlutus, Clone, Debug, PartialEq, Eq)]
    struct Basic {
        data: [u16; 2],
    }
    let data = Basic {
        data: [0xcafe, 0xbabe],
    };
    let plutus = create_constr(
        0,
        vec![create_array(vec![
            PlutusData::BigInt(BigInt::Int(0xcafe.into())),
            PlutusData::BigInt(BigInt::Int(0xbabe.into())),
        ])],
    );

    assert_encoded(data, plutus);
}

#[test]
fn should_error_when_array_fields_have_wrong_size() {
    #[derive(AsPlutus, Clone, Debug, PartialEq, Eq)]
    struct Basic {
        data: [u16; 2],
    }

    let plutus = create_constr(
        0,
        vec![create_array(vec![
            PlutusData::BigInt(BigInt::Int(0xcafe.into())),
            PlutusData::BigInt(BigInt::Int(0xbabe.into())),
            PlutusData::BigInt(BigInt::Int(0xd00d.into())),
        ])],
    );

    let error = DecodeError::wrong_length(2, 3).with_field_name("data");

    assert_eq!(Basic::from_plutus(plutus), Err(error));
}

#[test]
fn should_support_byte_array_fields() {
    #[derive(AsPlutus, Clone, Debug, PartialEq, Eq)]
    struct Basic {
        data: [u8; 28],
    }
    let data = Basic { data: [0x67; 28] };
    let plutus = create_constr(0, vec![PlutusData::BoundedBytes(vec![0x67; 28].into())]);

    assert_encoded(data, plutus);
}

#[test]
fn should_include_field_names_in_errors() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Basic {
        not_an_int: bool,
    }

    let plutus = create_constr(0, vec![PlutusData::BigInt(BigInt::Int(1.into()))]);

    let error = DecodeError::unexpected_type("Constr", "BigInt").with_field_name("not_an_int");

    assert_eq!(Basic::from_plutus(plutus), Err(error));
}

#[test]
fn should_include_nested_field_names_in_errors() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Outer {
        foo: Option<Inner>,
    }

    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Inner {
        bar: bool,
    }

    let plutus = create_constr(
        0,
        vec![create_constr(
            0,
            vec![create_constr(0, vec![create_constr(2, vec![])])],
        )],
    );

    let error = DecodeError::unexpected_variant(2).with_field_name("foo.bar");

    assert_eq!(Outer::from_plutus(plutus), Err(error));
}

#[test]
fn should_include_nested_field_indices_in_errors() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Outer {
        foo: Option<Inner>,
    }

    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Inner(bool);

    let plutus = create_constr(
        0,
        vec![create_constr(
            0,
            vec![create_constr(0, vec![create_constr(2, vec![])])],
        )],
    );

    let error = DecodeError::unexpected_variant(2).with_field_name("foo.0");

    assert_eq!(Outer::from_plutus(plutus), Err(error));
}

#[test]
fn should_include_array_indices_in_vec_errors() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Collection {
        items: Vec<Item>,
    }

    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Item {
        foo: Option<String>,
    }

    let plutus = create_constr(
        0,
        vec![create_array(vec![
            create_constr(
                0,
                vec![create_constr(
                    0,
                    vec![PlutusData::BoundedBytes(vec![0x01].into())],
                )],
            ),
            create_constr(
                0,
                vec![create_constr(
                    0,
                    vec![PlutusData::BoundedBytes(vec![0x02].into())],
                )],
            ),
            create_constr(
                0,
                vec![create_constr(
                    1,
                    vec![PlutusData::BoundedBytes(vec![0x03].into())],
                )],
            ),
        ])],
    );

    let error = DecodeError::wrong_variant_field_count(1, 0, 1).with_field_name("items[2].foo");

    assert_eq!(Collection::from_plutus(plutus), Err(error));
}

#[test]
fn should_include_array_indices_in_array_errors() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Collection {
        items: [Item; 3],
    }

    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Item {
        foo: Option<String>,
    }

    let plutus = create_constr(
        0,
        vec![create_array(vec![
            create_constr(
                0,
                vec![create_constr(
                    0,
                    vec![PlutusData::BoundedBytes(vec![0x01].into())],
                )],
            ),
            create_constr(
                0,
                vec![create_constr(
                    0,
                    vec![PlutusData::BoundedBytes(vec![0x02].into())],
                )],
            ),
            create_constr(
                0,
                vec![create_constr(
                    1,
                    vec![PlutusData::BoundedBytes(vec![0x03].into())],
                )],
            ),
        ])],
    );

    let error = DecodeError::wrong_variant_field_count(1, 0, 1).with_field_name("items[2].foo");

    assert_eq!(Collection::from_plutus(plutus), Err(error));
}

#[test]
fn should_include_tuple_indices_in_errors() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Collection {
        items: (Item, Item, Item),
    }

    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Item {
        foo: Option<String>,
    }

    let plutus = create_constr(
        0,
        vec![create_array(vec![
            create_constr(
                0,
                vec![create_constr(
                    0,
                    vec![PlutusData::BoundedBytes(vec![0x01].into())],
                )],
            ),
            create_constr(
                0,
                vec![create_constr(
                    0,
                    vec![PlutusData::BoundedBytes(vec![0x02].into())],
                )],
            ),
            create_constr(
                0,
                vec![create_constr(
                    1,
                    vec![PlutusData::BoundedBytes(vec![0x03].into())],
                )],
            ),
        ])],
    );

    let error = DecodeError::wrong_variant_field_count(1, 0, 1).with_field_name("items.2.foo");

    assert_eq!(Collection::from_plutus(plutus), Err(error));
}

#[test]
fn should_include_map_key_indices_in_errors() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Lookup {
        by_name: HashMap<String, u64>,
    }

    let plutus = create_constr(
        0,
        vec![create_map(vec![
            (
                PlutusData::BoundedBytes(vec![0x68, 0x65, 0x6c, 0x6c, 0x6f].into()),
                PlutusData::BigInt(BigInt::Int(69.into())),
            ),
            (
                PlutusData::BoundedBytes(vec![0x00, 0x9f, 0x92, 0x96].into()),
                PlutusData::BigInt(BigInt::Int(67.into())),
            ),
        ])],
    );

    let error = DecodeError::custom(
        "error decoding string: invalid utf-8 sequence of 1 bytes from index 1",
    )
    .with_field_name("by_name[(key #1)]");

    assert_eq!(Lookup::from_plutus(plutus), Err(error));
}

#[test]
fn should_include_map_value_indices_in_errors() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Values {
        by_index: BTreeMap<u64, Value>,
    }

    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Value {
        id: u8,
    }

    let plutus = create_constr(
        0,
        vec![create_map(vec![
            (
                PlutusData::BigInt(BigInt::Int(3.into())),
                create_constr(0, vec![PlutusData::BigInt(BigInt::Int(4.into()))]),
            ),
            (
                PlutusData::BigInt(BigInt::Int(3.into())),
                create_constr(0, vec![create_constr(0, vec![])]),
            ),
        ])],
    );

    let error =
        DecodeError::unexpected_type("BigInt", "Constr").with_field_name("by_index[(value #1)].id");

    assert_eq!(Values::from_plutus(plutus), Err(error));
}

#[test]
fn should_include_enum_variant_field_names_in_errors() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Outer {
        inner: Inner,
    }

    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    enum Inner {
        First { field: u8 },
        Second(u16),
    }

    let plutus = create_constr(
        0,
        vec![create_constr(
            0,
            vec![PlutusData::BigInt(BigInt::Int(257.into()))],
        )],
    );

    let error = DecodeError::out_of_range(257).with_field_name("inner::First.field");

    assert_eq!(Outer::from_plutus(plutus), Err(error));
}

#[test]
fn should_include_enum_variant_field_indices_in_errors() {
    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    struct Outer {
        inner: Inner,
    }

    #[derive(AsPlutus, Debug, PartialEq, Eq)]
    enum Inner {
        First { field: u8 },
        Second(u16),
    }

    let plutus = create_constr(
        0,
        vec![create_constr(
            1,
            vec![PlutusData::BigInt(BigInt::BigUInt(
                vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00].into(),
            ))],
        )],
    );

    let error =
        DecodeError::out_of_range("0x010000000000000000").with_field_name("inner::Second.0");

    assert_eq!(Outer::from_plutus(plutus), Err(error));
}
