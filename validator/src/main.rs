use blake2::{Blake2s256, Digest};
use std::fs;
use std::io;
use std::process::ExitCode;

mod entity;
use entity::InstallableLens;
use spyglass_lens::LensConfig;

const INDEX_FILE: &str = "../index.ron";
const LENS_FOLDER: &str = "../lenses";
const HTML_URL_PREFIX: &str = "https://github.com/spyglass-search/lens-box/blob/main/lenses";
const DL_URL_PREFIX: &str =
    "https://raw.githubusercontent.com/spyglass-search/lens-box/main/lenses";

fn validate_lens(lens: &InstallableLens) -> anyhow::Result<()> {
    println!("validating {} lens", lens.name);

    let file = lens
        .download_url
        .split('/')
        .collect::<Vec<&str>>()
        .pop()
        .expect("Unable to get file path");

    // Attempt to parse lens
    let contents = fs::read_to_string(format!("../lenses/{}", file)).expect("Unable to read lens");
    ron::from_str::<LensConfig>(&contents)?;

    Ok(())
}

fn check_lenses() -> io::Result<Vec<InstallableLens>> {
    let mut updated_lenses = Vec::new();

    for path in std::fs::read_dir(LENS_FOLDER)? {
        let path = path?.path();
        let file_contents = fs::read_to_string(path.clone())?;
        let lens = ron::from_str::<LensConfig>(&file_contents).expect("Unable to parse lens");

        let file_name = path
            .file_name()
            .expect("Should have a filename")
            .to_str()
            .expect("Unable to convert filename to &str");

        let mut hasher = Blake2s256::new();
        hasher.update(file_contents);
        let res = hasher.finalize();

        updated_lenses.push(InstallableLens {
            author: lens.author,
            description: lens.description.unwrap_or_else(||"".to_string()),
            name: lens.name,
            sha: hex::encode(res),
            download_url: format!("{}/{}", DL_URL_PREFIX, file_name),
            html_url: format!("{}/{}", HTML_URL_PREFIX, file_name),
        })
    }

    // Sort by name
    updated_lenses.sort_by(|a, b| a.name.cmp(&b.name));
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

fn main() -> ExitCode {
    match check_lenses() {
        Ok(updated) => {
            let ser = ron::ser::to_string_pretty(&updated, Default::default())
                .expect("Unable to serialize index");

            // Write out new index file.
            fs::write(INDEX_FILE, ser).expect("Unable to write index");

            // Validate index file.
            if let Err(e) = validate_index_file() {
                println!("index validation failed: {}", e);
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(_) => ExitCode::FAILURE,
    }
}
