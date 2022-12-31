use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::entity::InstallableLens;
use crate::ValidatorCli;

#[derive(Serialize)]
struct RepoItem {
    title: String,
    date: String,
    description: String,
    taxonomies: HashMap<String, Vec<String>>,
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

pub fn generate_page(cli: &ValidatorCli, base_path: &Path, lens: &InstallableLens) {
    let file_name = format!("{}.md", lens.name);

    let mut taxos = HashMap::new();
    taxos.insert("author".to_string(), vec![lens.author.to_string()]);

    let repo_item = RepoItem {
        title: lens.name.to_string(),
        description: lens.description.to_string(),
        date: "2022-01-01".to_string(),
        taxonomies: taxos,
    };

    if let Ok(res) = toml::ser::to_string_pretty(&repo_item) {
        if !cli.dry_run {
            let _ = fs::write(base_path.join(file_name), format!("+++\n{}+++\n", res));
        }
    }
}
