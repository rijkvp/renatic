use crate::{config::FeedConfig, content::Content};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct Feed {
    pub posts: Vec<Content>,
    pub rss: Option<RssInfo>,
}

#[derive(Debug, Serialize)]
pub struct RssInfo {
    pub path: PathBuf,
    pub route: PathBuf,
}

impl Feed {
    pub fn new(posts: Vec<Content>, config: &FeedConfig) -> Self {
        let rss = {
            if let Some(rss_path) = &config.rss_path {
                Some(RssInfo {
                    path: rss_path.clone(),
                    route: PathBuf::from("/").join(&rss_path),
                })
            } else {
                None
            }
        };
        Self {
            posts,
            rss,
        }
    }
}
