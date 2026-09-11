use hirn_proxy::caller::{OutputMode, detect_output_mode, resolve_output_mode};

// Flag truth table: explicit flags always beat TTY detection.
#[test]
fn json_flag_wins_over_everything() {
    assert_eq!(resolve_output_mode(true, false, true), OutputMode::Json);
    assert_eq!(resolve_output_mode(true, false, false), OutputMode::Json);
    assert_eq!(resolve_output_mode(true, true, true), OutputMode::Json);
}

#[test]
fn interactive_flag_wins_over_tty() {
    assert_eq!(resolve_output_mode(false, true, true), OutputMode::Pretty);
    assert_eq!(resolve_output_mode(false, true, false), OutputMode::Pretty);
}

#[test]
fn without_flags_tty_decides() {
    assert_eq!(resolve_output_mode(false, false, true), OutputMode::Pretty);
    assert_eq!(resolve_output_mode(false, false, false), OutputMode::Json);
}

// Thin wrapper: --json forces JSON even on a TTY-less test runner.
#[test]
fn detect_honors_json_flag() {
    assert_eq!(detect_output_mode(true, false), OutputMode::Json);
}
