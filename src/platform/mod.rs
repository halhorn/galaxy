//! Platform-specific [`UrlFragmentPort`] implementations.

#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use std::sync::Arc;

use crate::ports::url_fragment::UrlFragmentPort;

#[cfg(not(target_arch = "wasm32"))]
mod url_nav_native;
#[cfg(target_arch = "wasm32")]
mod url_nav_wasm;

/// Platform implementation for [`UrlFragmentPort`], injected at startup.
pub fn url_navigation_arc() -> Arc<dyn UrlFragmentPort + Send + Sync> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Arc::new(url_nav_native::NativeUrlNavigator)
    }
    #[cfg(target_arch = "wasm32")]
    {
        Arc::new(url_nav_wasm::WasmUrlNavigator)
    }
}
