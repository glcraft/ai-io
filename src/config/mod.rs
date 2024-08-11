pub mod prompt;


use std::{borrow::Cow, collections::HashMap};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{ser::SerializeMap, Deserialize, Serialize};

use prompt::Prompts as PromptsConfig;
use crate::{serde_io::DeserializeExt, utils::hashmap};
#[cfg(feature = "local-llm")]
use crate::generators::llama::config::Config as LlamaConfig;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub prompts: PromptsConfig,
    #[cfg(feature = "local-llm")]
    pub local: LlamaConfig,
    #[serde(default = "Config::default_endpoints")]
    pub endpoints: HashMap<String, Endpoint>,
}


impl Config {
    pub fn default_endpoints() -> HashMap<String, Endpoint> {
        hashmap!(
            openai => Endpoint { 
                url: "https://api.openai.com/v1".to_string(), 
                default_model: Some("gpt-4o-mini".to_string()) 
            }
        )
    }
    pub fn insert_default_endpoints(&mut self) {
        for (k, v) in Self::default_endpoints() {
            if !self.endpoints.contains_key(&k) {
                self.endpoints.insert(k.clone(), v);
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prompts: PromptsConfig::default(),
            #[cfg(feature = "local-llm")]
            local: LlamaConfig::default(),
            endpoints: Self::default_endpoints(),
        }
    }
}

impl DeserializeExt for Config {}

pub fn format_content<'a>(content: &'a str, args: &HashMap<String, String>) -> Cow<'a, str> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?P<prefix>\$\$?)(?P<name>\w+)").expect("Failed to compile regex"));
    RE.replace_all(content, |caps: &regex::Captures| {
        let prefix = &caps["prefix"];
        if prefix == "$$" {
            return format!("${}", &caps["name"]);
        }
        let name = &caps["name"];
        args.get(name).cloned().unwrap_or_default()
    })
}

impl Config {
    pub fn from_yaml_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, String> {
        let found_path = crate::filesystem::config_path(path.as_ref());
        let config = match found_path {
            Some(found_path) => {
                <Self as DeserializeExt>::from_yaml_file(found_path).map_err(|e| e.to_string())?
            }
            None => {
                use std::io::Write;
                let default_config = Self::default();
                let yaml = serde_yaml::to_string(&default_config).map_err(|e| e.to_string())?;
                std::fs::File::create(path).unwrap().write_all(yaml.as_bytes()).map_err(|e| e.to_string())?;
                default_config
            }
        };
        Ok(config)
    }
}

#[derive(Default, Debug)]
pub struct Endpoint {
    pub url: String,
    pub default_model: Option<String>,
}

impl<'de> Deserialize<'de> for Endpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        enum UrlOrModel {
            Url(String),
            Model { url: String, default_model: Option<String> }
        }

        let url_or_model = UrlOrModel::deserialize(deserializer)?;
        match url_or_model {
            UrlOrModel::Url(url) => Ok(Endpoint { url, default_model: None }),
            UrlOrModel::Model { url, default_model } => Ok(Endpoint { url, default_model })
        }
    }
}

impl Serialize for Endpoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(default_model) = &self.default_model {
            let mut state = serializer.serialize_map(None)?;
            state.serialize_entry("url", &self.url)?;
            state.serialize_entry("default_model", default_model)?;
            state.end()
        } else {
            serializer.serialize_str(&self.url)
        }
    }
}