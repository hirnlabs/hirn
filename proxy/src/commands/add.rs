use crate::caller::OutputMode;
use crate::output;

/// Fetch a tool/skill source and install it inactive (stub).
pub fn render(source: &str, mode: OutputMode) -> String {
    output::single_stub("add", "source", source, mode)
}
