use std::fs;
use std::process::ExitCode;

mod entity;
use entity::InstallableLens;

fn validate_lens(lens: &InstallableLens) -> Result<(), String> {
    println!("validating {} lens", lens.name);

    let file = lens.download_url.split('/').collect::<Vec<&str>>().pop().expect("Unable to get file path");

    // Attempt to parse lens
    let contents = fs::read_to_string(format!("../lenses/{}", file)).expect("Unable to read lens");
    if let Err(e) = ron::from_str::<entity::Lens>(&contents) {
        return Err(format!("{}", e));
    }

    Ok(())
}

fn main() -> ExitCode {
    let file_contents = fs::read_to_string("../index.ron").unwrap();
    if let Ok(index) = ron::from_str::<Vec<InstallableLens>>(&file_contents) {
        println!("index.ron successfully parsed");

        // Loop through index and validate len files
        for lens in index {
            if let Err(e) = validate_lens(&lens) {
                println!("Unable to validate <{}> due to: {}", lens.name, e);
                return ExitCode::FAILURE;
            }
        }

        return ExitCode::SUCCESS;
    } else {
        return ExitCode::FAILURE;
    }
}
