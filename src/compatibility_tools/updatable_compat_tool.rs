use std::path::PathBuf;
use tokio::fs;
use crate::compatibility_tools::compat_tools_list::CompatToolsList;
use crate::steam::steam::get_steam_compat_tools_path;

#[derive(Debug,Clone)]
pub struct UpdatableCompatTool {
    #[allow(dead_code)]
    pub name: String,
    pub display_name: String,
    pub path: PathBuf,
    #[allow(dead_code)]
    pub local_version: Option<String>,
}

async fn get_version_from_path(path: PathBuf) -> Option<String> {
    let version_file_path = path.join("ssw_version");
    let content = fs::read_to_string(&version_file_path).await;
    if let Ok(content) = content {
        return Some(content.trim().to_string())
    }
    None
}

impl UpdatableCompatTool {
    pub async fn from_tool_name(folder_prefix: &str) -> UpdatableCompatTool {
        let compat_tools = CompatToolsList::get();
        let tool_name = format!("{}-updatable", folder_prefix);
        let compat_tool = compat_tools.iter().find(|x| x.name == tool_name);

        let dir_path = PathBuf::from(&get_steam_compat_tools_path().join(tool_name.clone()));
        let version: Option<String> = if let Some(content) = compat_tool {
            get_version_from_path(PathBuf::from(&content.dir_path)).await
        } else {
            None
        };

        UpdatableCompatTool {
            display_name: format!("{} ({})", tool_name, &version.clone().unwrap_or("None".to_string())),
            name: tool_name,
            local_version: version,
            path: dir_path,
        }
    }
}
