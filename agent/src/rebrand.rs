pub fn rebrand_text(text: &str) -> String {
    text.replace(" goose ", " hirn ")
        .replace(" goose\n", " hirn\n")
        .replace(" goose\r", " hirn\r")
        .replace(" goose\"", " hirn\"")
        .replace(" goose`", " hirn`")
        .replace("goose.exe", "hirn")
        .replace("goose", "hirn")
        .replace("(default: ~/.config/goose)", "(default: ~/.config/hirn)")
        .replace("~/.config/goose/", "~/.config/hirn/")
        .replace(" GOOSE_", " HIRN_")
        .replace("\nGOOSE_", "\nHIRN_")
        .replace("\r\nGOOSE_", "\r\nHIRN_")
        .replace("Goose ", "Hirn ")
        .replace("Goose:", "Hirn:")
}

pub fn rebrand_tty_text(text: &str) -> String {
    text.replace("    __( O)>", "    .--@~^~@-.")
        .replace("   \\____)", "   ( @)(@ )(@ )")
        .replace("     L L", "    `-.@_@.-'")
        .replace("goose", "hirn")
        .replace("Goose", "Hirn")
        .replace("GOOSE", "HIRN")
}
