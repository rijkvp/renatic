use anyhow::Result;
use chrono::NaiveDateTime;
use quick_xml::{de, se::Serializer, DeError, Writer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct RssFeed {
    pub version: String,
    #[serde(rename = "channel")]
    pub channels: Vec<RssChannel>,
}

impl RssFeed {
    pub fn from_channel(channel: RssChannel) -> Self {
        Self {
            version: "2.0".to_string(),
            channels: vec![channel],
        }
    }
}
#[derive(Clone, Deserialize, Serialize)]
pub struct RssChannel {
    #[serde(rename = "$unflatten=title")]
    pub title: String,
    #[serde(rename = "$unflatten=link")]
    pub link: String,
    #[serde(rename = "$unflatten=description")]
    pub description: String,
    #[serde(rename = "item")]
    pub items: Vec<RssItem>,
}

impl RssChannel {
    pub fn new(title: String, link: String, description: String, items: Vec<RssItem>) -> Self {
        Self {
            title,
            link: link.clone(),
            description,
            items,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RssItem {
    #[serde(rename = "$unflatten=title")]
    pub title: String,
    #[serde(rename = "$unflatten=link")]
    pub link: String,
    #[serde(rename = "$unflatten=description")]
    pub description: String,

    #[serde(rename = "$unflatten=guid")]
    pub guid: String,
    #[serde(rename = "$unflatten=pubDate")]
    #[serde(with = "rss_date_format")]
    pub pub_date: NaiveDateTime,
}

mod rss_date_format {
    use chrono::{DateTime, NaiveDateTime};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%a, %d %b %Y %H:%M:%s +%T";

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = DateTime::parse_from_rfc2822(&s).map_err(serde::de::Error::custom)?;
        Ok(dt.naive_utc())
    }
}

pub fn to_str(feed: RssFeed) -> Result<String> {
    let mut buffer = Vec::new();
    let writer = Writer::new_with_indent(&mut buffer, b' ', 2);
    let mut ser = Serializer::with_root(writer, Some("rss"));

    feed.serialize(&mut ser)?;
    let string = String::from_utf8(buffer)?;
    Ok(string)
}

pub fn _from_str(xml: &str) -> Result<RssFeed, DeError> {
    let feed: RssFeed = de::from_str(xml)?;
    Ok(feed)
}
