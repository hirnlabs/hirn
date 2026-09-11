use crate::caller::OutputMode;
use crate::output;

/// Flip the `active` flag off in `tools.json` (stub).
pub fn render(name: &str, mode: OutputMode) -> String {
    output::single_stub("deactivate", "name", name, mode)
}
