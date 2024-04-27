use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use clap_cargo::{Features, Manifest, Workspace};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use human_panic::setup_panic;
use std::path::PathBuf;

mod commands;

#[derive(Debug, Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
struct Cli {
    // "shared" arguments moved into their own struct so we don't have to clone the command field.
    #[command(flatten)]
    args: Args,

    // our actual commands
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Args)]
struct Args {
    // clap_cargo
    #[command(flatten)]
    features: Features,
    #[command(flatten)]
    manifest: Manifest,
    #[command(flatten)]
    workspace: Workspace,

    // cargo-verbosity-flag
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

    // These may have to be moved into the respective commands later.
    #[arg(
        long,
        value_name = "TRIPLE",
        env = "CARGO_BUILD_TARGET",
        help = "The target triple the binary was built for. Defaults to the currently running platform."
    )]
    target: Vec<String>,
    /// Directory for all generated artifacts
    #[arg(
        long,
        value_name = "DIRECTORY",
        env = "CARGO_TARGET_DIR",
        help = "Must match the --target-dir flag used when the binary was built (if used). Required to resolve the correct target dir."
    )]
    pub target_dir: Option<PathBuf>,
    #[arg(
        long,
        value_name = "PROFILE-NAME",
        env = "CARGO_TARGET_DIR",
        help = "The cargo profile the binary was built with. Required to resolve the correct target dir."
    )]
    pub profile: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Dev,
    Build,
    Bundle {
        #[arg(
            value_name = "BUNDLE_TYPE",
            help = "List of bundle types to build. If none are provided, all available targets will be build."
        )]
        bundle_types: Vec<String>,
    },
    Icon,
    Android,
    Ios,
}

// TODO: ctrl-c handling

fn main() -> Result<()> {
    setup_panic!();

    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.args.verbose.log_level_filter())
        .init();

    match cli.command {
        Commands::Bundle { bundle_types } => commands::bundle::run(cli.args, bundle_types)?,
        _ => {
            return Err(anyhow!("Only the Bundle command is currently supported."));
        }
    }

    Ok(())
}
