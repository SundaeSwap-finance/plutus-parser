use plutus_parser::{AsPlutus, BoundedBytes};

#[derive(AsPlutus, Debug, PartialEq, Eq)]
pub enum IntervalBoundType {
    NegativeInfinity,
    Finite(u64),
    PositiveInfinity,
}

#[derive(AsPlutus, Debug, PartialEq, Eq)]
pub struct IntervalBound {
    pub bound_type: IntervalBoundType,
    pub is_inclusive: bool,
}

#[derive(AsPlutus, Debug, PartialEq, Eq)]
pub struct Interval {
    pub lower_bound: IntervalBound,
    pub upper_bound: IntervalBound,
}

#[derive(AsPlutus, Debug, PartialEq, Eq)]
struct Tuple(BoundedBytes, u64);

#[derive(AsPlutus)]
pub enum MultisigScript {
    Signature { key_hash: Vec<u8> },
}

#[derive(AsPlutus)]
pub enum Destination {
    #[variant = 1]
    Self_,
}
