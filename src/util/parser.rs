use anyhow::{anyhow, Result};
use pulldown_cmark::{html, Options, Parser};

pub fn parse_markdown_with_meta(input: &str) -> Result<(String, String)> {
    let splits: Vec<&str> = input.split("---").collect();
    if splits.len() != 3 {
        return Err(anyhow!("Invalid meta section!"));
    }
    Ok((splits[1].to_string(), markdown_to_html(&splits[2])))
}

fn markdown_to_html(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(input, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}
