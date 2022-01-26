use crate::{config::FeedConfig, meta::Meta};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone, Serialize)]
pub struct Post {
    /// The path of the original file
    #[serde(skip)]
    pub source_path: PathBuf,
    /// The file name of the original file
    pub file_name: String,
    /// The relative destination path
    #[serde(rename = "path")]
    pub target_path: PathBuf,
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
