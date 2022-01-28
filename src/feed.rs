use crate::{config::FeedConfig, post::Post};
use serde::Serialize;
use std::path::PathBuf;
#[derive(Debug, Serialize)]
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
