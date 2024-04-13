use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

pub fn main() -> Result<(), Box<dyn Error>> {
    let schema = schemars::schema_for!(tauri_utils::config::Config);
    let schema_str = serde_json::to_string_pretty(&schema).unwrap();
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    for file in [
        crate_dir.join("schema.json"),
        crate_dir.join("../../tooling/cli/schema.json"),
    ] {
        let mut schema_file = BufWriter::new(File::create(file)?);
        write!(schema_file, "{schema_str}")?;
    }

    Ok(())
}
