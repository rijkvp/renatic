use crate::consts::COLLECTION_CONFIG_FN;
use anyhow::{Context, Result};
use log::info;
use std::{fs, path::PathBuf};

// Type of indexed item, order determines generation order
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum IndexType {
    Collection,
    Directory,
    File,
}

#[derive(Debug)]
pub struct IndexItem {
    pub path: PathBuf,
    pub index_type: IndexType,
}

pub fn index(directory: &PathBuf) -> Result<Vec<IndexItem>> {
    let mut index_items = index_files(directory, 0).with_context(|| "Failed to index files")?;
    index_items.sort_by(|a, b| a.index_type.cmp(&b.index_type));
    info!("Indexed {} items (files/dirs/feeds)", index_items.len());
    Ok(index_items)
}

fn index_files(directory: &PathBuf, depth: u32) -> Result<Vec<IndexItem>> {
    let mut actions = Vec::new();
    for file in fs::read_dir(directory)? {
        let path = file?.path();
        // Directory
        if path.is_dir() {
            let feed_path = path.join(COLLECTION_CONFIG_FN);
            // Collection directory
            if feed_path.exists() {
                actions.push(IndexItem {
                    path,
                    index_type: IndexType::Collection,
                });
            }
            // Normal directory
            else {
                actions.push(IndexItem {
                    path: path.clone(),
                    index_type: IndexType::Directory,
                });
                // Recurse
                let mut child_actions = index_files(&path, depth + 1)?;
                actions.append(&mut child_actions);
            }
        }
        // File
        else if path.is_file() {
            actions.push(IndexItem {
                path,
                index_type: IndexType::File,
            });
        }
    }
    Ok(actions)
}
