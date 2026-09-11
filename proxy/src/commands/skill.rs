use crate::caller::OutputMode;
use crate::output;

/// Print the full SKILL.md of an active skill (stub).
pub fn render(name: &str, mode: OutputMode) -> String {
    output::single_stub("skill", "name", name, mode)
}
