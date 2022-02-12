use crate::{
    config::{CollectionConfig, Config},
    consts,
    index::{self, IndexType},
    renderer::ContentRenderer,
    sources::content::{CollectionBinding, Entry, Location},
    util::{
        minifier::{self, MinificationLevel},
        rss::{self, RssChannel, RssFeed, RssGuid, RssItem},
    },
};
use anyhow::{Context, Result};
use chrono::{NaiveDateTime, NaiveTime};
use log::{info, trace, warn};
use std::{ffi::OsStr, fs, path::PathBuf};

pub fn generate(
    source_dir: &PathBuf,
    out_dir: &PathBuf,
    mfc_level: &MinificationLevel,
) -> Result<()> {
    // Load main configuration
    let config_path = source_dir.join(consts::CONFIG_FN);
    let config = Config::load(&config_path).with_context(|| {
        format!(
            "Failed to load configuration file from '{}'",
            config_path.display()
        )
    })?;

    let renderer = ContentRenderer::load(source_dir.clone(), &config, mfc_level.clone())?;

    if out_dir.exists() {
        fs::remove_dir_all(out_dir).with_context(|| "Failed to remove previous output")?;
    }
    fs::create_dir_all(out_dir).with_context(|| "Failed to create output directory")?;

    let file_index = index::index(source_dir)?;

    // Loop over the index for the genration
    for index_item in file_index {
        let child_path = index_item.path.strip_prefix(source_dir)?;
        // Ignore hidden files starting with a '.'
        if config.ignore_hidden && child_path.starts_with(".") {
            continue;
        }
        // Ignore paths according to the configuration
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
        if out_path.exists() {
            warn!(
                "The file '{}' was skipped because it was already generated in an earlier stage! \
                Make sure you don't have dupplicate files or configure to ignore them",
                child_path.display()
            );
            continue;
        }

        match index_item.index_type {
            IndexType::Directory => {
                trace!("Create direcory '{}'", &out_path.display());
                fs::create_dir(out_path)?;
            }
            IndexType::File => {
                let ext = index_item
                    .path
                    .extension()
                    .unwrap_or(OsStr::new(""))
                    .to_str()
                    .unwrap();
                // Content file
                if ext == &config.content_ext {
                    let content = Entry::load(
                        Location::new(
                            &index_item.path,
                            source_dir,
                            out_dir,
                            &config.target_ext,
                            None,
                        )?,
                        None,
                    )?;
                    generate_inclusive_template(&content, &renderer)?;
                }
                // Template file without source
                else if ext == &config.template_ext {
                    trace!(
                        "Generate template without source '{}'",
                        &child_path.display()
                    );
                    let html = renderer.render(&index_item.path, None)?;
                    fs::write(out_path, &html)?;
                }
                // Minifiable file
                else if consts::MINIFY_EXTS.contains(&ext) {
                    trace!(
                        "Minify & copy non-template file '{}'",
                        index_item.path.display()
                    );
                    let contents = fs::read_to_string(index_item.path)?;
                    fs::write(out_path, minifier::minify_string(&contents, mfc_level))?;
                }
                // None of the above. A 'normal' file
                else {
                    fs::copy(index_item.path, out_path)?;
                }
            }
            IndexType::Collection => {
                // Collection configuration
                let collection_cfg_path = index_item.path.join(consts::COLLECTION_CONFIG_FN);
                let collection_cfg =
                    CollectionConfig::load(&collection_cfg_path).with_context(|| {
                        format!(
                            "Failed to load collection configuration from '{}'",
                            collection_cfg_path.display()
                        )
                    })?;
                let collection_dir = &index_item.path;

                fs::create_dir(&out_path)?;

                info!("Generating collection '{}'", child_path.display());

                // 1. Load all content entries of the collection
                let mut entries = Vec::<Entry>::new();
                for entry in fs::read_dir(collection_dir)? {
                    let entry_path = entry?.path().to_owned();
                    if entry_path.is_file()
                        && entry_path
                            .extension()
                            .context("")?
                            .eq_ignore_ascii_case(&config.content_ext)
                        && entry_path.file_stem().context("")? != consts::INDEX_SOURCE_FS
                    {
                        let content = Entry::load(
                            Location::new(
                                &entry_path,
                                &source_dir,
                                &out_dir,
                                &config.target_ext,
                                None,
                            )?,
                            None,
                        )
                        .with_context(|| {
                            format!("Failed to load content item '{}'", entry_path.display())
                        })?;
                        entries.push(content);
                    }
                }

                // 2. Sort the collection ascending by date
                // TODO: Make this a optional feature
                entries.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));

                // 3. Generate templates
                if let Some(template_path) = &collection_cfg.template {
                    for entry in entries.iter() {
                        generate_template(&template_path, entry, &renderer)?;
                    }
                }

                // 4. Generate index and other connections
                let binding = CollectionBinding::new(entries.clone(), &collection_cfg);
                // Standard index connection
                let index_path = collection_dir
                    .join(consts::INDEX_SOURCE_FS)
                    .with_extension(&config.content_ext);
                if index_path.exists() {
                    let entry = Entry::load(
                        Location::new(
                            &index_path,
                            &source_dir,
                            &out_dir,
                            &config.target_ext,
                            Some(consts::INDEX_TARGET_FS),
                        )?,
                        Some(binding.clone()),
                    )?;
                    generate_inclusive_template(&entry, &renderer)?;
                }
                // Custom connections
                for conn_path in collection_cfg.connections.iter() {
                    let conn_path = source_dir.join(conn_path);
                    let content = Entry::load(
                        Location::new(
                            &conn_path,
                            &source_dir,
                            &out_dir,
                            &config.template_ext,
                            None,
                        )?,
                        Some(binding.clone()),
                    )?;
                    generate_inclusive_template(&content, &renderer)?;
                }

                // 5. Generate RSS if enabled
                if let Some(rss_path) = &collection_cfg.rss {
                    let rss_out = out_dir.join(rss_path);
                    generate_rss_feed(&entries, rss_path, &rss_out, &config, &collection_cfg)?;
                }
            }
        }
    }
    info!("Generation successfully completed!");

    Ok(())
}

fn generate_inclusive_template(entry: &Entry, renderer: &ContentRenderer) -> Result<()> {
    let template_path = entry.meta.template.as_ref().context(format!(
        "Unspecified required template option for '{}'",
        entry.location.source_child_path.display()
    ))?;
    generate_template(template_path, entry, renderer)
}

fn generate_template(
    template_path: &PathBuf,
    entry: &Entry,
    renderer: &ContentRenderer,
) -> Result<()> {
    trace!(
        "Generate content for {}",
        entry.location.source_child_path.display()
    );
    let html = renderer
        .render(&template_path, Some(entry))
        .with_context(|| {
            format!(
                "Failed to generate content for '{}' using template '{}'",
                entry.location.source_child_path.display(),
                template_path.display(),
            )
        })?;
    fs::write(&entry.location.target_path, html).with_context(|| {
        format!(
            "Failed to write generated content to {}",
            entry.location.target_path.display()
        )
    })?;
    Ok(())
}

fn generate_rss_feed(
    entries: &[Entry],
    rss_path: &PathBuf,
    rss_out_path: &PathBuf,
    main_cfg: &Config,
    collection_cfg: &CollectionConfig,
) -> Result<()> {
    trace!("Generating RSS feed '{}'", rss_path.display());
    let has_content = collection_cfg.template.is_some();
    let mut rss_items = Vec::new();
    for entry in entries {
        let link = {
            if has_content {
                main_cfg.base_url.clone() + entry.location.short_route.to_str().unwrap()
            } else {
                main_cfg.base_url.clone() + "#" + &entry.location.target_file_stem
            }
        };
        let pub_date = {
            if let Some(date) = entry.meta.date {
                date.and_time(NaiveTime::from_hms(12, 0, 0))
            } else {
                warn!("The entry {} item has no date! This can cause issues with templates and RSS feed generation.", entry.meta.title);
                NaiveDateTime::from_timestamp(0, 0)
            }
        };
        rss_items.push(RssItem {
            title: entry.meta.title.clone(),
            link: link.clone(),
            description: entry.content.to_string(),
            guid: RssGuid {
                value: link,
                is_permalink: has_content,
            },
            pub_date,
        })
    }
    let link = format!("{}/{}", main_cfg.base_url, rss_path.display());
    let channel = RssChannel::new(
        collection_cfg.title.clone(),
        link,
        collection_cfg.description.clone(),
        rss_items,
    );
    let feed = RssFeed::from_channel(channel);
    let rss_str = rss::to_str(feed)?;
    fs::write(rss_out_path, rss_str)?;
    Ok(())
}
