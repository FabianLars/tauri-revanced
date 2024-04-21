use anyhow::Result;

use crate::Args;

pub mod linux;

pub fn run(args: Args, bundle_types: Vec<String>) -> Result<()> {
    let all = bundle_types.is_empty();

    Ok(())
}
