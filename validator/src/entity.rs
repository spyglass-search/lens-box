use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct InstallableLens {
    pub author: String,
    pub description: String,
    pub name: String,
    pub sha: String,
    pub download_url: String,
    pub html_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub enum LensRule {
    SkipURL(String),
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Lens {
    #[serde(default = "Lens::default_author")]
    pub author: String,
    pub name: String,
    pub description: Option<String>,
    pub domains: Vec<String>,
    pub urls: Vec<String>,
    pub version: String,
    #[serde(default = "Lens::default_is_enabled")]
    pub is_enabled: bool,
    #[serde(default)]
    pub rules: Vec<LensRule>,
}

impl Lens {
    fn default_author() -> String {
        "Unknown".to_string()
    }

    fn default_is_enabled() -> bool {
        true
    }
}