use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct InstallableLens {
    pub author: String,
    pub description: String,
    pub name: String,
    pub sha: String,
    pub download_url: String,
    pub html_url: String,
    #[serde(skip)]
    pub path: PathBuf,
}
