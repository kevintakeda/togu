use std::{fs, path::PathBuf};

use anyhow::Ok;
use clap::{Parser, Subcommand};
use translation::extract_translations;
pub mod translation;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    name: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ExtractTranslation {
        #[arg(short, long, value_name = "DIRECTORY")]
        /// The directory to scan for translations
        directory: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    match &cli.command {
        Commands::ExtractTranslation { directory } => {
            let dir: PathBuf = fs::canonicalize(directory).unwrap();
            extract_translations(dir)?;
            Ok(())
        }
    }
}
