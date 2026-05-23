use crate::ports::url_fragment::UrlFragmentPort;

#[derive(Clone, Copy, Default)]
pub(super) struct NativeUrlNavigator;

impl UrlFragmentPort for NativeUrlNavigator {
    fn current_fragment_body(&self) -> Option<String> {
        None
    }

    fn replace_fragment_body(&self, _fragment_body: &str) -> Result<(), String> {
        Ok(())
    }

    fn fragment_equals(&self, _fragment_body: &str) -> bool {
        false
    }
}
