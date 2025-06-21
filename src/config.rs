use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub hooks: Vec<Hook>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub name: String,
    pub regex: String,
    pub command: String,
    #[serde(default)]
    pub cooldown_ms: Option<u64>,
}

pub fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .expect("Failed to get config directory")
        .join("cc-hook")
        .join("config.toml")
}

pub fn load_config(path: Option<PathBuf>) -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = path.unwrap_or_else(default_config_path);
    
    if !config_path.exists() {
        return Ok(Config { hooks: vec![] });
    }
    
    let content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;
    
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_config() {
        let toml_str = r#"
            [[hooks]]
            name = "permission_prompt"
            regex = "Do you want to proceed\\?"
            command = "notify-send '[cc-wrap] Permission confirmation'"
            
            [[hooks]]
            name = "task_finished"
            regex = "^●.*"
            command = "notify-send '[cc-wrap] Task finished'"
        "#;
        
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.hooks.len(), 2);
        assert_eq!(config.hooks[0].name, "permission_prompt");
        assert_eq!(config.hooks[1].name, "task_finished");
    }
}