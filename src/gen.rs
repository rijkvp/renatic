use crate::{
    config::{Config, FeedConfig},
    feed::{Feed, Post},
    meta::Meta,
    rss::{self, RssChannel, RssFeed, RssItem},
    templating::TemplateEngine,
};
use anyhow::{anyhow, Context, Result};
use chrono::NaiveTime;
use log::info;
use pulldown_cmark::{html, Options, Parser};
use std::{collections::HashMap, fs, path::PathBuf};
use tera::Context as TemplateContext;
use walkdir::{DirEntry, WalkDir};

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn generate(root_dir: &PathBuf, template_engine: &TemplateEngine) -> Result<()> {
    let config_path = root_dir.join("renatic.yaml");
    let config = Config::load(&config_path).with_context(|| {
        format!(
            "Failed to load configuration file from '{}'",
            config_path.display()
        )
    })?;

    for entry in WalkDir::new(&root_dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
    {
        let entry = entry?;
        if entry.path().is_dir() {
            let feed_path = entry.path().join("feed.yaml");
            // Generate and copy as feed
            if feed_path.exists() {
                let feed_cfg = FeedConfig::load(&feed_path).with_context(|| {
                    format!(
                        "Failed to load feed configuration from '{}'",
                        feed_path.display()
                    )
                })?;
                let template_dir = entry.path().strip_prefix(root_dir)?;
                info!("Generating feed '{}'", template_dir.display());
                let files = generate_feed(
                    &entry.path().to_path_buf(),
                    &template_dir.to_path_buf(),
                    &config,
                    &feed_cfg,
                    &template_engine,
                )?;
                println!("FILES: {:#?}", files.keys());
            }
        }
    }

    Ok(())
}

fn generate_feed(
    dir: &PathBuf,
    template_dir: &PathBuf,
    config: &Config,
    feed_cfg: &FeedConfig,
    template_engine: &TemplateEngine,
) -> Result<HashMap<PathBuf, String>> {
    let mut result_files = HashMap::<PathBuf, String>::new();

    let posts = load_posts(&dir, &template_dir)?;

    // Generate content
    if let Some(templates) = &feed_cfg.templates {
        for template in templates.iter() {
            let template_path = dir.join(template);
            for post in posts.iter() {
                let out_path = post.target_path.with_extension("html");
                let context = TemplateContext::from_serialize(post)?;
                let result = template_engine.render_file(&template_path, &context)?;
                result_files.insert(out_path, result);
            }
        }
    }

    // Generate index
    if let Some(index_templates) = &feed_cfg.index_templates {
        let feed = Feed::new(posts.clone(), &feed_cfg);
        let context = TemplateContext::from_serialize(feed)?;
        for template in index_templates.iter() {
            let template_path = dir.join(template);
            let result = template_engine.render_file(&template_path, &context)?;
            let out_path = template_dir.join(template);
            result_files.insert(out_path, result);
        }
    }

    // Generate RSS
    if let Some(rss_path) = &feed_cfg.rss_path {
        let rss = generate_rss(&posts, rss_path, &config, &feed_cfg)?;
        result_files.insert(rss_path.to_path_buf(), rss);
    }

    Ok(result_files)
}

fn load_posts(dir: &PathBuf, template_dir: &PathBuf) -> Result<Vec<Post>> {
    let mut posts = Vec::<Post>::new();

    // Iterate all markdown files
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path().to_owned();
        if let Some(ext) = path.extension() {
            if path.is_file() && ext.eq_ignore_ascii_case("md") {
                let file_str = fs::read_to_string(&path)?;
                let (meta, content) = split_md_meta(&file_str)
                    .with_context(|| format!("Failed to read file '{}'", path.display()))?;
                let file_name = path.file_name().unwrap();
                posts.push(Post {
                    source_path: path.clone(),
                    file_name: file_name.to_str().unwrap().to_string(),
                    target_path: template_dir.join(path.strip_prefix(dir)?.to_path_buf()),
                    meta,
                    content,
                });
            }
        }
    }

    // Sort the feed ascending by date
    posts.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));

    Ok(posts)
}

fn split_md_meta(input: &str) -> Result<(Meta, String)> {
    let splits: Vec<&str> = input.split("---").collect();
    if splits.len() != 3 {
        return Err(anyhow!("Invalid meta section!"));
    }
    let meta = serde_yaml::from_str::<Meta>(&splits[1])?;
    let contents = md_to_html(&splits[2]);
    Ok((meta, contents))
}

fn md_to_html(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(input, options);

    // Write to String buffer.
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

fn generate_rss(
    posts: &[Post],
    rss_path: &PathBuf,
    config: &Config,
    feed_cfg: &FeedConfig,
) -> Result<String> {
    let mut rss_items = Vec::new();
    for post in posts {
        rss_items.push(RssItem {
            title: post.meta.title.clone(),
            link: post.target_path.to_str().unwrap().to_string(),
            description: post.content.to_string(),
            guid: post.target_path.to_str().unwrap().to_string(),
            pub_date: post.meta.date.and_time(NaiveTime::from_hms(12, 0, 0)),
        })
    }
    let link = format!("{}/{}", config.base_url, rss_path.display());
    let channel = RssChannel::new(
        feed_cfg.title.clone(),
        link,
        feed_cfg.description.clone(),
        rss_items,
    );
    let feed = RssFeed::from_channel(channel);
    let rss_str = rss::to_str(feed)?;
    Ok(rss_str)
}
