use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct InstallableLens {
    pub author: String,
    pub description: String,
    /// Unique ID for lens
    pub name: String,
    /// Human readable string for lens.
    pub label: String,
    pub sha: String,
    pub download_url: String,
    pub html_url: String,
    #[serde(skip)]
    pub path: PathBuf,
}
