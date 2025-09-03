use std::error::Error;
use std::process::Command;

pub fn to_quoted_string(args: Vec<String>) -> String {
    format!("\"{}\"", args.join("\" \""))
}

pub fn has_command(cmd: String) -> Result<bool, Box<dyn Error>> {
    Ok(Command::new("command").args(&["-v", cmd.as_str()]).status()?.success())
}

pub fn find_terminal_emulator() -> Option<String> {
    let terminals = [
        "x-terminal-emulator", "gnome-terminal", "konsole",
        "xfce4-terminal", "tilix", "mate-terminal",
        "lxterminal", "terminator", "xterm"
    ];

    for term in terminals {
        if has_command(term.to_string()).is_ok() {
            return Some(term.to_string())
        }
    }

    eprintln!("No terminal emulator found.");
    None
}
