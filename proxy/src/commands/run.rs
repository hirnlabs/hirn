use crate::caller::OutputMode;
use crate::output;

/// Resolve a tool call into summary + policy verdict (stub: no execution).
pub fn render(tool: &str, args: &str, mode: OutputMode) -> String {
    output::run_stub(tool, args, mode)
}
