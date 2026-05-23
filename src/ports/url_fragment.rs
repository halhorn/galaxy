/// Read and write the `#` fragment body (`v=…&…`) without encoding domain state.
pub trait UrlFragmentPort: Send + Sync {
    /// Current fragment body without the leading `#`. Returns `None` when absent or empty.
    fn current_fragment_body(&self) -> Option<String>;

    /// Replace the fragment with `body` (no leading `#`).
    fn replace_fragment_body(&self, fragment_body: &str) -> Result<(), String>;

    /// Whether the current fragment already equals `fragment_body`.
    fn fragment_equals(&self, fragment_body: &str) -> bool;
}
