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

fn main() {
    to_from_plutus();
    to_from_plutus_bytes();
}

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
}

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
