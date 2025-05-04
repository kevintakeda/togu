use std::{fs, path::PathBuf};

use anyhow::Ok;
use clap::{Parser, Subcommand};
use translation::extract_translations;
pub mod translation;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::ValueEnum, Clone)]
#[clap(rename_all = "lowercase")]
enum Flavor {
    Laravel,
}

#[derive(Subcommand)]
enum Commands {
    ExtractTranslations {
        #[arg(short, long, value_name = "DIRECTORY")]
        /// The directory to scan for translations
        directory: PathBuf,

        /// The flavor of the translations
        #[arg(short, long, value_enum, value_name = "FLAVOR")]
        #[clap(default_value_t = Flavor::Laravel)]
        flavor: Flavor,
    },
}

fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    match &cli.command {
        Commands::ExtractTranslations { directory, flavor } => {
            let dir: PathBuf = fs::canonicalize(directory).unwrap();
            extract_translations(dir)?;
            Ok(())
        }
    }
}
