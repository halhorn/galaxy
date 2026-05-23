//! Wire-format encoding for flat `key=value&…` query bodies (domain-agnostic).

mod flat_query_codec;

pub(crate) use flat_query_codec::{SubLevel, TopLevel};
