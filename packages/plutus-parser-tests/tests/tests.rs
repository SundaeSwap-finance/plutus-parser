use plutus_parser::{AsPlutus, BigInt, BoundedBytes, MaybeIndefArray, PlutusData, create_constr};
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

    let plutus = PlutusData::Array(MaybeIndefArray::Def(vec![
        PlutusData::BoundedBytes(BoundedBytes::from(vec![0x13, 0x37])),
        PlutusData::BigInt(BigInt::Int(9001.into())),
    ]));

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
        vec![PlutusData::Array(MaybeIndefArray::Def(vec![
            PlutusData::BoundedBytes(BoundedBytes::from(vec![0xca, 0xfe])),
        ]))],
    );

    assert_encoded(data, plutus);
}
