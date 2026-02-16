#[derive(Debug)]
pub struct RemoteCompatToolsProvider {
    pub name: &'static str,
    pub remote_path: &'static str
}

pub const REMOTE_COMPAT_TOOL_PROVIDERS: &[RemoteCompatToolsProvider] = &[
    RemoteCompatToolsProvider{
        name: "proton-cachyos",
        remote_path: "CachyOS/proton-cachyos"
    },
    RemoteCompatToolsProvider{
        name: "GE-Proton",
        remote_path: "GloriousEggroll/proton-ge-custom"
    }
];
