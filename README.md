# plutus-parser

A simple create to derive `from_plutus` and `to_plutus` on Rust types.

```rs
use plutus_parser::{AsPlutus, Constr, MaybeIndefArray, PlutusData};

#[derive(AsPlutus, Debug, PartialEq, Eq)]
pub struct MyType {
    pub bool_field: bool,
}

fn main() {
    let data = MyType { bool_field: true };

    let plutus = data.to_plutus();
    assert_eq!(
        plutus,
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: MaybeIndefArray::Def(vec![PlutusData::Constr(Constr {
                tag: 122,
                any_constructor: None,
                fields: MaybeIndefArray::Def(vec![]),
            })])
        })
    );

    assert_eq!(
        MyType::from_plutus(plutus).unwrap(),
        MyType { bool_field: true }
    );
}
```