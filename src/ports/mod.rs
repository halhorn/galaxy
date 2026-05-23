//! Browser URL fragment read/write (trait only).

#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

pub mod url_fragment;

pub use url_fragment::UrlFragmentPort;
