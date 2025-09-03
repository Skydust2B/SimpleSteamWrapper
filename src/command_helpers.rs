use log::{warn};
use which::which;

pub fn to_quoted_string(args: Vec<String>) -> String {
    format!("\"{}\"", args.join("\" \""))
}

pub fn find_terminal_emulator() -> Option<String> {
    let terminals = [
        "x-terminal-emulator", "gnome-terminal", "konsole",
        "xfce4-terminal", "tilix", "mate-terminal",
        "lxterminal", "terminator", "xterm"
    ];

    for term in terminals {
        if which(term.to_string()).is_ok() {
            return Some(term.to_string())
        }
    }

    warn!("No terminal emulator found.");
    None
}
