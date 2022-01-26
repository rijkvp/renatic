mod config;
mod feed;
mod gen;
mod meta;
mod rss;
mod templating;

use crate::templating::TemplateEngine;
use anyhow::Result;
use clap::Parser;
use gen::generate;
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
}

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let cli = Cli::parse();
    let source_dir = cli.source.unwrap_or(Path::new(".").to_path_buf());

    let template_engine = TemplateEngine::load(source_dir.clone())?;

    generate(&source_dir, &cli.output, &template_engine)?;

    Ok(())
}
