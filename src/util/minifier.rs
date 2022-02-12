use clap::ArgEnum;
use minify_html::{minify, Cfg};

#[derive(Clone, ArgEnum)]
pub enum MinificationLevel {
    Disabled,
    SpecCompliant,
    Maximal,
}

pub fn minify_string(input: &str, minification: &MinificationLevel) -> String {
    match minification {
        MinificationLevel::Disabled => input.to_string(),
        MinificationLevel::SpecCompliant => {
            String::from_utf8(minify(input.as_bytes(), &Cfg::spec_compliant())).unwrap()
        }
        MinificationLevel::Maximal => {
            String::from_utf8(minify(input.as_bytes(), &Cfg::new())).unwrap()
        }
    }
}
