use std::sync::Arc;

use bevy::prelude::*;

use crate::ports::url_fragment::UrlFragmentPort;

/// Injected platform navigator for URL fragment I/O.
#[derive(Resource, Clone)]
pub struct UrlNavigation(pub Arc<dyn UrlFragmentPort + Send + Sync>);
