use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::utils::get_current_datetime;

pub fn get_config_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join(".config").join("qwk")
}

pub fn ensure_config_dir() -> io::Result<PathBuf> {
    let config_dir = get_config_dir();
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir)
}

pub fn get_aliases_file() -> PathBuf {
    get_config_dir().join("aliases.json")
}

pub fn get_agent_file() -> PathBuf {
    get_config_dir().join("agent")
}

pub fn load_aliases() -> HashMap<String, String> {
    let aliases_file = get_aliases_file();
    if aliases_file.exists() {
        let content = fs::read_to_string(&aliases_file).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

pub fn save_aliases(aliases: &HashMap<String, String>) -> io::Result<()> {
    ensure_config_dir()?;
    let aliases_file = get_aliases_file();
    let content = serde_json::to_string_pretty(aliases)?;
    fs::write(aliases_file, content)
}

pub fn get_agent() -> String {
    let agent_file = get_agent_file();
    if agent_file.exists() {
        fs::read_to_string(&agent_file)
            .unwrap_or_else(|_| "claude".to_string())
            .trim()
            .to_string()
    } else {
        "claude".to_string()
    }
}

pub fn set_agent(command: &str) -> io::Result<()> {
    ensure_config_dir()?;
    let agent_file = get_agent_file();
    fs::write(agent_file, command)
}

pub fn create_aliases_backup() -> io::Result<Option<String>> {
    let aliases_file = get_aliases_file();
    if !aliases_file.exists() {
        return Ok(None);
    }

    let config_dir = ensure_config_dir()?;
    let datetime = get_current_datetime();
    let backup_file = config_dir.join(format!("aliases_backup_{}.json", datetime));

    fs::copy(&aliases_file, &backup_file)?;
    Ok(Some(backup_file.to_string_lossy().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn setup_test_config(temp_dir: &TempDir) -> PathBuf {
        temp_dir.path().join("qwk")
    }

    #[test]
    fn test_alias_storage_and_retrieval() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = setup_test_config(&temp_dir);
        fs::create_dir_all(&config_dir).unwrap();

        let aliases_file = config_dir.join("aliases.json");

        // Test empty case
        let empty_aliases = load_aliases_from_file(&aliases_file);
        assert!(empty_aliases.is_empty());

        // Test saving and loading
        let mut test_aliases = HashMap::new();
        test_aliases.insert("test1".to_string(), "prompt1".to_string());
        test_aliases.insert("test2".to_string(), "prompt2".to_string());

        save_aliases_to_file(&aliases_file, &test_aliases).unwrap();

        let loaded_aliases = load_aliases_from_file(&aliases_file);
        assert_eq!(loaded_aliases.len(), 2);
        assert_eq!(loaded_aliases.get("test1"), Some(&"prompt1".to_string()));
        assert_eq!(loaded_aliases.get("test2"), Some(&"prompt2".to_string()));
    }

    // Helper functions for testing
    fn load_aliases_from_file(file_path: &PathBuf) -> HashMap<String, String> {
        if file_path.exists() {
            let content = fs::read_to_string(file_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }

    fn save_aliases_to_file(
        file_path: &PathBuf,
        aliases: &HashMap<String, String>,
    ) -> io::Result<()> {
        let content = serde_json::to_string_pretty(aliases)?;
        fs::write(file_path, content)
    }
}
