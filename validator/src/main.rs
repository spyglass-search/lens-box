use blake2::{Blake2s256, Digest};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

mod entity;
mod repo;

use entity::InstallableLens;
use spyglass_lens::LensConfig;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct ValidatorCli {
    #[arg(short, long)]
    pub dry_run: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate lens explorer site
    GenerateExplorer,
    /// (Default) Validates lenses & generate index file
    Validate,
}

const INDEX_FILE: &str = "../index.ron";
const LENS_FOLDER: &str = "../lenses";
const HTML_URL_PREFIX: &str = "https://github.com/spyglass-search/lens-box/blob/main/lenses";
const DL_URL_PREFIX: &str =
    "https://raw.githubusercontent.com/spyglass-search/lens-box/main/lenses";

fn validate_lens(lens: &InstallableLens) -> anyhow::Result<()> {
    println!("validating {} - {:?}", lens.name, lens.label);

    let mut components = lens.download_url.split('/').collect::<Vec<&str>>();

    let file = components.pop().expect("Unable to get file path");

    let parent = components.pop().expect("Unable to get parent file path");

    // Attempt to parse lens
    let contents =
        fs::read_to_string(format!("../lenses/{}/{}", parent, file)).expect("Unable to read lens");
    ron::from_str::<LensConfig>(&contents)?;

    Ok(())
}

fn find_lens(path: &PathBuf) -> Option<(PathBuf, String)> {
    for file in std::fs::read_dir(path)
        .expect("Unable to read directory")
        .flatten()
    {
        let path = file.path();
        if path.is_file() && path.extension() == Some(OsStr::new("ron")) {
            if let Ok(file_contents) = std::fs::read_to_string(file.path()) {
                return Some((path, file_contents));
            }
        }
    }

    None
}

fn check_lenses() -> anyhow::Result<Vec<InstallableLens>> {
    let mut updated_lenses = Vec::new();

    // Find folders to check for lenses
    let mut lenses = Vec::new();
    for path in std::fs::read_dir(LENS_FOLDER)?.flatten() {
        if path.path().is_dir() {
            lenses.push(path);
        }
    }

    let mut categories: HashMap<String, usize> = HashMap::new();

    for path in lenses {
        if let Some((file_path, file_contents)) = find_lens(&path.path()) {
            let lens = match ron::from_str::<LensConfig>(&file_contents) {
                Ok(lens) => lens,
                Err(err) => {
                    eprintln!("Unable to parse {} - {}", path.path().display(), err);
                    continue;
                }
            };

            for cat in &lens.categories {
                let num = categories.entry(cat.to_string()).or_insert(0);
                *num += 1;
            }

            let parent = file_path
                .parent()
                .expect("No parent path")
                .components()
                .last()
                .expect("Last component")
                .as_os_str()
                .to_str()
                .expect("Unable to convert parent directory to &str");

            let file_name = file_path
                .file_name()
                .expect("Should have a filename")
                .to_str()
                .expect("Unable to convert filename to &str");

            let mut hasher = Blake2s256::new();
            hasher.update(file_contents);
            let res = hasher.finalize();

            if file_name != format!("{}.ron", lens.name) {
                return Err(anyhow::anyhow!(
                    "{} lens file should match lens name",
                    lens.name
                ));
            }

            let label = lens.label();
            updated_lenses.push(InstallableLens {
                author: lens.author,
                description: lens.description.unwrap_or_default(),
                name: lens.name,
                label,
                sha: hex::encode(res),
                path: file_path.clone(),
                categories: lens.categories,
                download_url: format!("{}/{}/{}", DL_URL_PREFIX, parent, file_name),
                html_url: format!("{}/{}/{}", HTML_URL_PREFIX, parent, file_name),
            });
        }
    }

    println!("Category Stats");
    println!("--------------");
    let mut sorted = categories.iter().collect::<Vec<_>>();
    sorted.sort();
    for (key, count) in sorted.iter() {
        println!("{key}: {count}")
    }

    // Sort by name
    updated_lenses.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    println!("\nFound {} lenses\n", updated_lenses.len());
    Ok(updated_lenses)
}

fn validate_index_file() -> anyhow::Result<()> {
    let file_contents = fs::read_to_string(INDEX_FILE)?;
    if let Ok(index) = ron::from_str::<Vec<InstallableLens>>(&file_contents) {
        println!("index.ron successfully parsed");

        // Loop through index and validate len files
        for lens in index {
            validate_lens(&lens)?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = ValidatorCli::parse();
    match check_lenses() {
        Ok(updated) => {
            match &cli.command {
                None | Some(Commands::Validate) => {
                    let ser = ron::ser::to_string_pretty(&updated, Default::default())
                        .expect("Unable to serialize index");

                    // Write out new index file.
                    if !cli.dry_run {
                        fs::write(INDEX_FILE, ser).expect("Unable to write index");
                    }

                    // Validate index file.
                    if let Err(e) = validate_index_file() {
                        println!("index validation failed: {}", e);
                        return ExitCode::FAILURE;
                    }

                    ExitCode::SUCCESS
                }
                Some(Commands::GenerateExplorer) => {
                    // Generate docs
                    println!("\ngenerating lens explorer site...");
                    let base_path = repo::get_and_clean_doc_path(&cli);
                    for lens in &updated {
                        if let Err(e) = repo::generate_page(&cli, &base_path, lens) {
                            println!("page generation failure: {}", e);
                            return ExitCode::FAILURE;
                        }
                    }
                    println!("generated {} pages", updated.len());

                    ExitCode::SUCCESS
                }
            }
        }
        Err(err) => {
            println!("Error checking lens {:?}", err);
            ExitCode::FAILURE
        }
    }
}
