use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Serialize, Serializer};
use serde_yaml::{Mapping, Value};

const DATE_FORMAT: &'static str = "%Y-%m-%d";

#[derive(Debug, Clone)]
pub struct Meta {
    pub title: String,
    pub date: Option<NaiveDate>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub template: Option<PathBuf>,
    custom_fields: Mapping,
}

fn get_str_value(key: &str, map: &Mapping) -> Result<String> {
    let value = map
        .get(&Value::String(key.to_string()))
        .with_context(|| format!("Failed to get required key '{key}'"))?;
    let str = value.as_str().context(format!(
        "Failed to get required value string for key '{key}'"
    ))?;
    Ok(str.to_string())
}

impl Meta {
    pub fn from_str(input: &str) -> Result<Self> {
        let mut meta = serde_yaml::from_str::<Mapping>(input)
            .with_context(|| format!("Failed to read YAML input: '{input}'"))?;
        let title = get_str_value("title", &meta)?;
        let date = {
            if let Some(value) = meta.get(&Value::String("date".to_string())) {
                let date_str = value.as_str().context("Failed to read date as a string.")?;
                Some(NaiveDate::parse_from_str(&date_str, DATE_FORMAT)?)
            } else {
                None
            }
        };
        let category = {
            if let Some(value) = meta.get(&Value::String("category".to_string())) {
                let category_str = value
                    .as_str()
                    .context("Failed read category as a string.")?;
                Some(category_str.to_string())
            } else {
                None
            }
        };
        let tags = {
            if let Some(value) = meta.get(&Value::String("tags".to_string())) {
                let tags_squence = value
                    .as_sequence()
                    .context("Failed to read tags as a sequence")?;
                let mut tags = Vec::new();
                for tag_val in tags_squence {
                    tags.push(
                        tag_val
                            .as_str()
                            .with_context(|| "Failed to convert tag value to string.")?
                            .to_string(),
                    );
                }
                Some(tags)
            } else {
                None
            }
        };
        let template = {
            if let Some(value) = meta.get(&Value::String("template".to_string())) {
                let template_str = value
                    .as_str()
                    .context("Failed to read template as a string.")?;
                Some(PathBuf::from(template_str))
            } else {
                None
            }
        };

        for key in vec!["title", "date", "category", "tags"] {
            meta.remove(&Value::String(key.to_string()));
        }

        Ok(Self {
            title,
            date,
            category,
            tags,
            template,
            custom_fields: meta,
        })
    }
}

impl Serialize for Meta {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut values = self.custom_fields.clone();
        values.insert(
            Value::String("title".to_string()),
            Value::String(self.title.clone()),
        );
        if let Some(date) = &self.date {
            values.insert(
                Value::String("date".to_string()),
                Value::String(date.format(DATE_FORMAT).to_string()),
            );
        };
        if let Some(category) = &self.category {
            values.insert(
                Value::String("category".to_string()),
                Value::String(category.clone()),
            );
        }
        if let Some(tags) = &self.tags {
            values.insert(
                Value::String("tags".to_string()),
                Value::Sequence(tags.iter().map(|t| Value::String(t.to_string())).collect()),
            );
        }
        values.serialize(serializer)
    }
}
