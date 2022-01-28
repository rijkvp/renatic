mod config;
mod feed;
mod gen;
mod meta;
mod post;
mod rss;
mod templating;

use anyhow::Result;
use clap::Parser;
use gen::generate;
use log::error;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(long, short = 'o')]
    /// Path to output directory
    output: PathBuf,
    #[clap(long, short = 's')]
    /// Path to source directory
    source: Option<PathBuf>,
    #[clap(long, short = 'v')]
    /// Verbose output
    verbose: bool,
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

    let source_dir = cli.source.unwrap_or(Path::new(".").to_path_buf());
    if source_dir == cli.output {
        error!("The source directory can't be the destination directory!");
    } else {
        generate(&source_dir, &cli.output)?;
    }

    Ok(())
}
