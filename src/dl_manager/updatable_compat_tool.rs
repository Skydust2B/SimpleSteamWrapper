use std::path::PathBuf;
use tokio::fs;
use crate::compatibility_tools::steam::list_steam_compat_tools;

#[derive(Debug,Clone)]
pub struct UpdatableCompatTool {
    pub name: String,
    pub folder_prefix: String,
    pub local_version: Option<String>,
}

async fn get_version_from_path(path: PathBuf) -> Option<String> {
    let version_file_path = path.join("version");
    let content = fs::read_to_string(&version_file_path).await;
    if let Ok(content) = content {
        return Some(content.trim().to_string())
    }
    None
}

impl UpdatableCompatTool {
    pub async fn from_tool_name(name: &str, folder_prefix: &str) -> UpdatableCompatTool {
        let compat_tools = list_steam_compat_tools();
        let compat_tool = compat_tools.iter().find(|x| x.name == format!("{}-Updatable", folder_prefix));

        let version: Option<String> = if let Some(content) = compat_tool {
            get_version_from_path(PathBuf::from(&content.dir_path)).await
        } else {
            None
        };

        UpdatableCompatTool {
            name: name.to_string(),
            folder_prefix: folder_prefix.to_string(),
            local_version: version
        }
    }
}
