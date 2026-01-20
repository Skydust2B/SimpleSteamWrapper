#[derive(Debug)]
pub struct DownloadableCompatTool {
    pub name: &'static str,
    pub remote_path: &'static str
}

pub const DOWNLOADABLE_COMPAT_TOOLS: &[DownloadableCompatTool] = &[
    DownloadableCompatTool{
        name: "proton-cachyos",
        remote_path: "CachyOS/proton-cachyos"
    },
    DownloadableCompatTool{
        name: "GE-Proton",
        remote_path: "GloriousEggroll/proton"
    }
];

