use crate::{
    config::{CollectionConfig, Config},
    consts,
    index::{self, IndexType},
    renderer::ContentRenderer,
    sources::entry::{CollectionBinding, Entry},
    util::{
        minifier::{self, MinificationLevel},
        rss::{self, RssChannel, RssFeed, RssGuid, RssItem},
    },
};
use anyhow::{Context, Result};
use chrono::{NaiveDateTime, NaiveTime};
use log::{error, info, trace, warn};
use std::{collections::HashMap, ffi::OsStr, fs, path::PathBuf};

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

    let renderer = ContentRenderer::load(source_dir.clone(), &config)?;

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
                trace!("Create direcory '{}'", &out_dir.display());
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
                    info!("Redering content of '{}'", &child_path.display());
                    let content = Entry::load(
                        source_dir,
                        source_dir,
                        &index_item.path,
                        &config.template_ext,
                    )?;
                    if let Some(template) = content.meta.template.clone() {
                        let template_path = source_dir.join(template);
                        let content_out = out_path.with_extension(&config.target_ext);
                        trace!(
                            "Rendering template '{}' to '{}'",
                            template_path.display(),
                            content_out.display()
                        );
                        let html = renderer.render(&template_path, Some(&content))?;
                        fs::write(content_out, minifier::minify_string(&html, mfc_level))?;
                    } else {
                        error!(
                            "Unkown template file for '{}'. Please specify a template.",
                            child_path.display()
                        );
                    }
                }
                // Template file without source
                else if ext == &config.template_ext {
                    trace!("Render HTML '{}'", &child_path.display());
                    let html = renderer.render(&index_item.path, None)?;
                    fs::write(out_path, minifier::minify_string(&html, mfc_level))?;
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
                    trace!("Copy '{}'", &child_path.display());
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
                let mut collection_files = HashMap::<PathBuf, String>::new();

                // 1. Load all content entries of the collection
                let mut entries = Vec::<Entry>::new();
                for entry in fs::read_dir(collection_dir)? {
                    let path = entry?.path().to_owned();
                    if let Some(ext) = path.extension() {
                        if path.is_file() && ext.eq_ignore_ascii_case(&config.content_ext) {
                            let content = Entry::load(
                                &child_path.to_path_buf(),
                                collection_dir,
                                &path,
                                &config.template_ext,
                            )
                            .with_context(|| {
                                format!("Failed to load content item '{}'", path.display())
                            })?;
                            entries.push(content);
                        }
                    }
                }

                // 2. Sort the collection ascending by date
                // TODO: Make this a optional feature
                entries.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));

                // 3. Generate templates
                if let Some(templates) = &collection_cfg.templates {
                    for template in templates.iter() {
                        let template_path = source_dir.join(template);
                        for entry in entries.iter() {
                            let out_path = entry
                                .location
                                .child_path
                                .with_extension(&config.template_ext);
                            trace!("Render template '{}'", out_path.display());
                            let result = renderer.render(&template_path, Some(entry))?;
                            // Write
                            collection_files.insert(out_path, result);
                        }
                    }
                }

                // 4. Generate connections
                if let Some(index_templates) = &collection_cfg.connections {
                    let binding = CollectionBinding::new(entries.clone(), &collection_cfg);
                    for template in index_templates.iter() {
                        let template_path = source_dir.join(template);
                        let content = Entry::load(
                            source_dir,
                            &collection_dir,
                            &template_path,
                            &config.template_ext,
                        )?;
                        if let Some(template) = content.meta.template.clone() {
                            let template_path = source_dir.join(template);
                            let content_out = out_path.with_extension(&config.target_ext);
                            trace!(
                                "Rendering template '{}' to '{}'",
                                template_path.display(),
                                content_out.display()
                            );
                            let html = renderer.render(&template_path, Some(&content))?;
                            fs::write(content_out, minifier::minify_string(&html, mfc_level))?;
                        } else {
                            error!(
                                "Unkown template file for '{}'. Please specify a template.",
                                child_path.display()
                            );
                        }
                        trace!("Render index template '{}'", template_path.display());
                        let result = renderer.render(&template_path, Some(&binding))?;
                        let out_path = source_dir.join(template);
                        collection_files.insert(out_path, result);
                    }
                }

                // 5. Generate RSS if enabled
                if let Some(rss_path) = &collection_cfg.rss {
                    let rss = generate_rss(&entries, rss_path, &config, &collection_cfg)?;
                    trace!("Render RSS feed '{}'", rss_path.display());
                    collection_files.insert(rss_path.to_path_buf(), rss);
                }

                // 6. Write generated files
                trace!("Writing {} generated files", collection_files.len());
                for (c_child_path, html) in collection_files {
                    let c_out_path = out_dir.join(c_child_path);
                    fs::write(c_out_path, minifier::minify_string(&html, &mfc_level))?;
                }
            }
        }
    }
    info!("Generation successfully completed!");

    Ok(())
}

fn generate_rss(
    entries: &[Entry],
    rss_path: &PathBuf,
    main_cfg: &Config,
    collection_cfg: &CollectionConfig,
) -> Result<String> {
    let has_content = collection_cfg.templates.as_ref().unwrap_or(&vec![]).len() > 0;
    let mut rss_items = Vec::new();
    for entry in entries {
        let link = {
            if has_content {
                main_cfg.base_url.clone() + entry.location.short_route.to_str().unwrap()
            } else {
                main_cfg.base_url.clone() + "#" + &entry.location.file_stem
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
            description: entry.source.to_string(),
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
    Ok(rss_str)
}
