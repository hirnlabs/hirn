use std::io::IsTerminal;

/// How the proxy renders its verdict.
///
/// `Json` is the agent contract (non-TTY callers get standard tool-call
/// JSON). `Pretty` is the human contract (terminal callers get readable
/// output plus interactive prompts).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Json,
    Pretty,
}

/// Pure decision: flags always beat TTY detection.
///
/// Separated from [`detect_output_mode`] so the truth table is unit
/// testable without a real terminal.
pub fn resolve_output_mode(json_flag: bool, interactive_flag: bool, stdout_is_tty: bool) -> OutputMode {
    if json_flag {
        return OutputMode::Json;
    }
    if interactive_flag {
        return OutputMode::Pretty;
    }
    if stdout_is_tty {
        OutputMode::Pretty
    } else {
        OutputMode::Json
    }
}

/// Impure edge: reads the real stdout TTY state, then delegates to
/// [`resolve_output_mode`]. This is the only function that touches the
/// terminal; everything downstream takes an `OutputMode` value.
pub fn detect_output_mode(json_flag: bool, interactive_flag: bool) -> OutputMode {
    resolve_output_mode(json_flag, interactive_flag, std::io::stdout().is_terminal())
}
