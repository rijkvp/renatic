use crate::{config::FeedConfig, meta::Meta};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone, Serialize)]
pub struct Post {
    /// Original source path
    #[serde(skip)]
    pub source_path: PathBuf,
    /// Original child path
    pub child_path: PathBuf,
    /// Original file name
    pub file_name: String,

    /// Path of destination file
    #[serde(rename="path")]
    pub target_path: PathBuf,
    /// Destination path without extension
    pub route: PathBuf,


    pub meta: Meta,
    pub content: String,
}

#[derive(Serialize)]
pub struct Feed {
    pub rss_path: Option<PathBuf>,
    pub posts: Vec<Post>,
}

impl Feed {
    pub fn new(posts: Vec<Post>, config: &FeedConfig) -> Self {
        Self {
            rss_path: config.rss_path.clone(),
            posts,
        }
    }
}
