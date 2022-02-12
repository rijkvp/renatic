use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use quick_xml::{de, se::Serializer, DeError, Writer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct RssFeed {
    pub version: String,
    #[serde(rename = "xmlns:atom")]
    pub atom_namespace: String,
    #[serde(rename = "channel")]
    pub channels: Vec<RssChannel>,
}

impl RssFeed {
    pub fn from_channel(channel: RssChannel) -> Self {
        Self {
            version: "2.0".to_string(),
            atom_namespace: "http://www.w3.org/2005/Atom".to_string(),
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
    #[serde(rename = "atom:link")]
    pub atom_link: RssAtomLink,
    #[serde(rename = "$unflatten=description")]
    pub description: String,
    #[serde(rename = "$unflatten=lastBuildDate")]
    #[serde(with = "rfc_2822_date")]
    pub last_build_date: NaiveDateTime,
    #[serde(rename = "$unflatten=generator")]
    pub generator: String,
    #[serde(rename = "item")]
    pub items: Vec<RssItem>,
}

impl RssChannel {
    pub fn new(title: String, link: String, description: String, items: Vec<RssItem>) -> Self {
        Self {
            title,
            link: link.clone(),
            atom_link: RssAtomLink::from_link(link),
            description,
            last_build_date: Utc::now().naive_utc(),
            generator: "renatic".to_string(),
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
    pub guid: RssGuid,
    #[serde(rename = "$unflatten=pubDate")]
    #[serde(with = "rfc_2822_date")]
    pub pub_date: NaiveDateTime,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RssGuid {
    #[serde(rename = "$value")]
    pub value: String,
    #[serde(rename = "isPermaLink")]
    pub is_permalink: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RssAtomLink {
    pub href: String,
    pub rel: String,
    #[serde(rename = "type")]
    pub mime_type: String,
}

impl RssAtomLink {
    pub fn from_link(link: String) -> Self {
        Self {
            href: link,
            rel: "self".to_string(),
            mime_type: "application/rss+xml".to_string(),
        }
    }
}

mod rfc_2822_date {
    use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let date_time: DateTime<Utc> = Utc.from_local_datetime(&date).unwrap();
        serializer.serialize_str(&date_time.to_rfc2822())
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
