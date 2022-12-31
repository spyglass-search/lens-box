use serde::Serialize;
use spyglass_lens::LensConfig;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

use crate::entity::InstallableLens;
use crate::ValidatorCli;

#[derive(Serialize)]
struct RepoItem {
    title: String,
    date: String,
    description: String,
    extra: HashMap<String, Value>,
    taxonomies: HashMap<String, Value>,
}

const DOCS_FOLDER: &str = "../docs";

pub fn get_and_clean_doc_path(cli: &ValidatorCli) -> PathBuf {
    let docs_path = Path::new(DOCS_FOLDER).join("content/lenses");

    // No need to clean up the directory if this is a dry run
    if cli.dry_run {
        return docs_path;
    }

    fs::read_dir(docs_path.clone())
        .expect("Unable to walk DOCS_FOLDER")
        .flatten()
        .collect::<Vec<fs::DirEntry>>()
        .iter()
        .for_each(|file| {
            if file.file_name() != "_index.md" {
                let _ = fs::remove_file(file.path());
            }
        });

    docs_path
}

pub fn generate_page(
    cli: &ValidatorCli,
    base_path: &Path,
    lens: &InstallableLens,
) -> anyhow::Result<()> {
    if let Ok(lens_config) = ron::from_str::<LensConfig>(&std::fs::read_to_string(&lens.path)?) {
        let file_name = format!("{}.md", lens.name);

        let mut taxos: HashMap<String, Value> = HashMap::new();
        taxos.insert("author".to_string(), vec![lens.author.to_string()].into());

        let mut extra: HashMap<String, Value> = HashMap::new();
        extra.insert("domains".to_string(), lens_config.domains.into());
        extra.insert("urls".to_string(), lens_config.urls.into());

        let repo_item = RepoItem {
            title: lens.name.to_string(),
            description: lens.description.to_string(),
            date: "2022-01-01".to_string(),
            extra,
            taxonomies: taxos,
        };

        if let Ok(res) = toml::ser::to_string_pretty(&repo_item) {
            if !cli.dry_run {
                fs::write(base_path.join(file_name), format!("+++\n{}+++\n", res))?;
            }
        }

        Ok(())
    } else {
        Err(anyhow::anyhow!(format!(
            "Unable to parse {}",
            lens.path.display()
        )))
    }
}
