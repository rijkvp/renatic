mod config;
mod feed;
mod gen;
mod meta;
mod post;
mod rss;
mod templating;

use anyhow::Result;
use clap::{ArgEnum, Parser};
use log::error;
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

#[derive(Clone, ArgEnum)]
pub enum MinificationLevel {
    Disabled,
    SpecCompliant,
    Maximal,
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
                gen::generate(&source_dir, &out_dir, &minification)?;
            }
        }
    }

    Ok(())
}
