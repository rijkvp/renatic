mod config;
mod consts;
mod generator;
mod index;
mod sources;
mod renderer;
mod util;

use anyhow::Result;
use clap::Parser;
use log::error;
use util::minifier::MinificationLevel;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// Use verbose output
    #[clap(long, short = 'v')]
    verbose: bool,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Generate a site
    Generate {
        /// Path to the output directory
        output: PathBuf,

        /// Select a source directory, defaults to current directory
        #[clap(long, short = 's')]
        source: Option<PathBuf>,

        /// Set the minification level
        #[clap(long, short = 'm', arg_enum, default_value_t = MinificationLevel::SpecCompliant)]
        minification: MinificationLevel,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let level = {
        if cli.verbose {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        }
    };
    env_logger::builder()
        .format_module_path(false)
        .format_timestamp(None)
        .filter_level(level)
        .init();

    match cli.command {
        Commands::Generate {
            source,
            output,
            minification,
        } => {
            let source_dir = source
                .unwrap_or(Path::new(".").to_path_buf())
                .canonicalize()?;
            let out_dir = output.canonicalize()?;
            if source_dir == out_dir {
                error!("The source directory can't be the destination directory!");
            } else {
                generator::generate(&source_dir, &out_dir, &minification)?;
            }
        }
    }

    Ok(())
}
