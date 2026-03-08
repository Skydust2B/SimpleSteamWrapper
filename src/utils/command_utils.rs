use log::{warn};
use shlex::split;
use which::which;

pub fn to_quoted_string(args: Vec<String>) -> String {
    format!("\"{}\"", args.iter().map(|arg| arg.replace("\"", "\\\"")).collect::<Vec<String>>().join("\" \""))
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

#[derive(Debug)]
pub struct ParsedCmd {
    pub env: Vec<(String, String)>,
    pub progname: String,
    pub args: Vec<String>,
}

pub fn parse_cmdline(input: &str) -> ParsedCmd {
    let tokens = split(input).unwrap_or_default();
    let mut envs = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    // Collect KEY=VALUE pairs
    while let Some(token) = iter.peek() {
        if let Some(eq) = token.find('=') {
            let part = iter.next().unwrap();
            let (k, v) = part.split_at(eq);
            envs.push((k.to_string(), v[1..].to_string()));
        } else {
            break;
        }
    }

    let progname = iter.next().unwrap_or_default();
    let args: Vec<String> = iter.collect();

    ParsedCmd { env: envs, progname, args }
}
