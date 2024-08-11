use std::collections::HashMap;
use serde::Deserialize;
use crate::serde_io::DeserializeExt;


#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub apikey: HashMap<String, String>
}

impl DeserializeExt for Credentials {}

impl Credentials {
    /// Get API key from environment variable or YAML file.
    /// 
    /// The environment variable name is formatted as `<NAME>_API_KEY`.
    pub fn get_api_key(path: &str, name: &str) -> Option<String> {
        let env_var_name = format!("{}_API_KEY", name.to_uppercase());
        std::env::var(env_var_name)
            .ok()
            .or_else(|| Self::from_yaml_file(path)
                .ok()
                .and_then(|c| c.apikey.get(name).cloned())
            )
    }
}