use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
struct MetaLink {
    content: String,
    url: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct MetaImage {
    alt: String,
    file_name: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Meta {
    pub title: String,
    subtitle: Option<String>,
    #[serde(with = "date_format")]
    pub date: NaiveDate,
    date_label: Option<String>,
    tags: Option<Vec<String>>,
    image: Option<MetaImage>,
    links: Option<Vec<MetaLink>>,
}

mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(dt)
    }
}
