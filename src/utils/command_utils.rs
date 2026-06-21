use std::process::Command;
use log::{warn};
use shlex::split;
use which::which;

pub fn to_quoted_string(args: Vec<String>) -> String {
    format!("\"{}\"", args.iter().map(|arg| {
                let val = if arg.starts_with('"') && arg.ends_with('"') {
                    arg[1..arg.len() - 1].to_string()
                } else {
                    arg.to_string()
                };

                val.replace("\"", "\\\"")
            }).collect::<Vec<String>>().join("\" \""))
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

pub trait UpdateEnvVar {
    fn update_env_var<F>(&mut self, var: &str, updater: F)
    where
        F: Fn(Option<String>) -> String;

    fn add_parameter_to_var(&mut self, separator: &str, var: &str, value: &str);
}

impl UpdateEnvVar for Command {
    fn update_env_var<F>(&mut self, var: &str, updater: F)
    where
        F: Fn(Option<String>) -> String
    {
        let current_value = self.get_envs()
            .find(|(key, _)| key.to_str() == Some(var))
            .and_then(|(_, value)| value)
            .and_then(|value| value.to_str())
            .map(String::from);

        self.env(var, updater(current_value));
    }

    fn add_parameter_to_var(&mut self, separator: &str, var: &str, value: &str) {
        self.update_env_var(var, |prev_var| {
            if let Some(prev_value) = prev_var {
                let mut splitted = prev_value.split(separator).collect::<Vec<&str>>();
                splitted.push(value);
                splitted.join(separator)
            } else {
                return value.to_string();
            }
        })
    }
}
