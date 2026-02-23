use std::{env, io};
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use log::info;
use tokio::process::Command;
use crate::utils::command_utils::{find_terminal_emulator, to_quoted_string};
use crate::compatibility_tools::compat_tool::{CompatTool};

pub struct AppPrefix {
    path: PathBuf,
    proton: bool
}

impl AppPrefix {
    pub fn from_env() -> Self {
        let data_path = env::var("STEAM_COMPAT_DATA_PATH").expect("STEAM_COMPAT_DATA_PATH must be set");
        Self {
            path: PathBuf::from(data_path),
            proton: true
        }
    }

    pub fn as_proton_path(&self) -> Result<PathBuf, &str> {
        if !self.proton {
            return Err("Unable to transform path, not a proton prefix");
        }
        Ok(self.path.clone())
    }

    pub fn as_wine_path(&self) -> PathBuf {
        if self.proton {
            return self.path.clone().join("pfx")
        }
        self.path.clone()
    }

    pub fn as_path(&self) -> PathBuf {
        match self.proton {
            true => self.as_proton_path().unwrap(),
            false => self.as_wine_path()
        }
    }

    pub fn open_folder(&self) {
        open::that_detached(self.path.as_path()).expect("Unable to open folder");
    }

    pub async fn run_wiretricks(&self, compat_tool: &CompatTool) -> io::Result<ExitStatus> {
        let wine_path = self.as_wine_path();
        let wine_bin_path = compat_tool.find_wine_bin()
            .expect(&format!("Unable to find wine binary in compat tool {}", compat_tool.name));

        info!("Running winetricks with {} at {}", compat_tool.name, wine_path.display());
        let mut process = Command::new("winetricks");

        process
            .arg("--gui")
            .env("WINEPREFIX", wine_path)
            .env("WINE", wine_bin_path)
            .env("WINEDEBUG", "-a")
            .env("LD_PRELOAD", "")
            .env("LD_LIBRARY_PATH", "")
            .status().await
    }
    
    pub async fn reset_prefix(&self, compat_tool: &CompatTool) -> io::Result<ExitStatus> {
        let prefix_top_folder = self.as_path();

        info!("Recreating prefix with {} at {}", compat_tool.name, prefix_top_folder.display());
        if prefix_top_folder.exists() {
            info!("Removing prefix {}", prefix_top_folder.display());
            tokio::fs::remove_dir_all(&prefix_top_folder).await.expect("Could not remove prefix");
        }
        tokio::fs::create_dir_all(&prefix_top_folder).await.expect("Unable to create prefix");

        let mut process = Command::new(compat_tool.clone().path);

        if self.proton {
            process
                .args(&["run", "/bin/true"])
                .env("STEAM_COMPAT_DATA_PATH", prefix_top_folder.to_str().unwrap())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status().await
        } else {
            process
                .args(&["wineboot", "--init"])
                .env("WINEPREFIX", prefix_top_folder.to_str().unwrap())
                .env("WINEARCH", "64")
                .env("WINEDEBUG", "-a")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status().await
        }
    }

    pub async fn run_in_prefix(&self, compat_tool: &CompatTool, command: std::process::Command, in_terminal: bool) -> io::Result<ExitStatus> {
        let wine_bin_path = compat_tool.find_wine_bin()
            .expect(&format!("Unable to find wine binary in compat tool {}", compat_tool.name));

        let mut process = Command::new("sh");
        process.arg("-c")
            .env("WINEDEBUG", "-a")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit());

        if in_terminal {
            if let Some(terminal) = find_terminal_emulator() {
                process = Command::new(terminal);
                process.arg("-e");
                process.env("WINEDEBUG", "");
            } else {
                info!("Unable to find terminal emulator, falling back to sh");
            }
        }

        process
            .envs(command.get_envs().map(|(k, v)| (k, v.unwrap_or_default())))
            .env("WINEPREFIX", self.as_wine_path())
            .env("LD_PRELOAD", "")
            .env("LD_LIBRARY_PATH", "");

        let mut cmd = Vec::new();

        cmd.push(wine_bin_path.to_str().unwrap().to_string());
        cmd.push(command.get_program().to_str().unwrap().to_string());
        cmd = [
            cmd,
            command.get_args().map(|v| v.to_str().unwrap().to_string()).collect()
        ].concat();

        let joined_cmd = to_quoted_string(cmd);
        info!("Running cmd in prefix: {}", joined_cmd);

        process
            .arg(joined_cmd);

        process.status().await
    }
}
