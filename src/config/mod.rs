pub mod prompt;


use std::{borrow::Cow, collections::HashMap};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use prompt::Prompts as PromptsConfig;
use crate::serde_io::DeserializeExt;
#[cfg(feature = "local-llm")]
use crate::generators::llama::config::Config as LlamaConfig;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Config {
    pub prompts: PromptsConfig,
    #[cfg(feature = "local-llm")]
    pub local: LlamaConfig,
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