//! Wire-format encoding for flat `key=value&…` query bodies (domain-agnostic).

pub mod flat_query_codec;

pub use flat_query_codec::{SubLevel, SubLevelKv, TopLevel};
