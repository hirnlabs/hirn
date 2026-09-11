use crate::caller::OutputMode;
use crate::output;

/// Merged view of active tools (stub: always empty).
pub fn render(mode: OutputMode) -> String {
    output::list_stub(mode)
}
