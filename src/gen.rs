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

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum IndexType {
    Dir,
    File,
    Feed,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct FileIndex {
    path: PathBuf,
    index_type: IndexType,
}

pub fn generate(
    source_dir: &PathBuf,
    out_dir: &PathBuf,
    template_engine: &TemplateEngine,
) -> Result<()> {
    let config_path = source_dir.join("renatic.yaml");
    let config = Config::load(&config_path).with_context(|| {
        format!(
            "Failed to load configuration file from '{}'",
            config_path.display()
        )
    })?;

    if out_dir.exists() {
        fs::remove_dir_all(out_dir).with_context(|| "Failed to remove previous output")?;
    }
    fs::create_dir_all(out_dir).with_context(|| "Failed to create output directory")?;

    let file_index = index_files(source_dir, 0).with_context(|| "Failed to index files")?;
    info!("Indexed {} items (files/dirs/feeds)", file_index.len());

    for index in file_index {
        let child_path = index.path.strip_prefix(source_dir)?;
        // Ignore hidden files starting with '.'
        if config.ignore_hidden && child_path.starts_with(".") {
            continue;
        }
        // Ignore paths according to config
        let mut is_ignored = false;
        for ignore_path in config.ignore_paths.iter() {
            if child_path.starts_with(ignore_path) {
                is_ignored = true;
                break;
            }
        }
        if is_ignored {
            continue;
        }

        let out_path = out_dir.join(&child_path);
        match index.index_type {
            IndexType::Dir => {
                fs::create_dir(out_path)?;
            }
            IndexType::File => {
                fs::copy(index.path, out_path)?;
            }
            IndexType::Feed => {
                let feed_path = index.path.join("feed.yaml");
                let feed_cfg = FeedConfig::load(&feed_path).with_context(|| {
                    format!(
                        "Failed to load feed configuration from '{}'",
                        feed_path.display()
                    )
                })?;
                info!("Generating feed '{}'", child_path.display());
                let files = generate_feed(
                    &index.path,
                    &child_path.to_path_buf(),
                    &config,
                    &feed_cfg,
                    &template_engine,
                )?;
                info!("Writing {} generated files", files.len());
                fs::create_dir(&out_path)?;
                for (feed_child_path, contents) in files {
                    let feed_out_path = out_dir.join(feed_child_path);
                    fs::write(feed_out_path, contents)?;
                }
            }
        }
    }
    info!("Generation successful");

    Ok(())
}

fn index_files(dir: &PathBuf, depth: u32) -> Result<Vec<FileIndex>> {
    let mut actions = Vec::new();
    for file in fs::read_dir(dir)? {
        let path = file?.path();
        // Directory
        if path.is_dir() {
            let feed_path = path.join("feed.yaml");
            // Feed directory
            if feed_path.exists() {
                actions.push(FileIndex {
                    path,
                    index_type: IndexType::Feed,
                });
            }
            // Normal directory
            else {
                actions.push(FileIndex {
                    path: path.clone(),
                    index_type: IndexType::Dir,
                });
                // Recurse
                let mut child_actions = index_files(&path, depth + 1)?;
                actions.append(&mut child_actions);
            }
        }
        // File
        else if path.is_file() {
            actions.push(FileIndex {
                path,
                index_type: IndexType::File,
            });
        }
    }
    Ok(actions)
}

fn generate_feed(
    parent_dir: &PathBuf,
    child_dir: &PathBuf,
    config: &Config,
    feed_cfg: &FeedConfig,
    template_engine: &TemplateEngine,
) -> Result<HashMap<PathBuf, String>> {
    let mut result_files = HashMap::<PathBuf, String>::new();

    let posts = load_posts(&parent_dir, &child_dir)?;

    // Generate content
    if let Some(templates) = &feed_cfg.templates {
        for template in templates.iter() {
            let template_path = parent_dir.join(template);
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
            let template_path = parent_dir.join(template);
            let result = template_engine.render_file(&template_path, &context)?;
            let out_path = child_dir.join(template);
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
