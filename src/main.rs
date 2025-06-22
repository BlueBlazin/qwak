use clap::{Parser, Subcommand, CommandFactory};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "qwk")]
#[command(about = "A CLI tool for creating aliases for AI agents")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    #[arg(help = "Run a stored shortcut")]
    shortcut: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(long_flag = "set")]
    #[command(about = "Set an alias for a prompt")]
    #[command(long_about = "Set an alias for a prompt. If no prompt is provided, it will be read from stdin.")]
    Set {
        #[arg(help = "The alias name to set")]
        alias: String,
        #[arg(help = "The prompt text (optional, will read from stdin if not provided)")]
        prompt: Option<String>,
    },
    #[command(long_flag = "agent")]
    #[command(about = "Set the agent command to use")]
    #[command(long_about = "Set the agent command to use when executing shortcuts. Defaults to 'claude'.")]
    Agent {
        #[arg(help = "The command to use as the agent")]
        command: String,
    },
    #[command(long_flag = "reset")]
    #[command(about = "Reset all shortcuts (creates backup)")]
    #[command(long_about = "Reset all shortcuts by clearing the aliases file. A backup will be created automatically. The agent setting is preserved.")]
    Reset,
}

fn get_config_dir() -> PathBuf {
    let home = env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join(".config").join("qwk")
}

fn ensure_config_dir() -> io::Result<PathBuf> {
    let config_dir = get_config_dir();
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir)
}

fn get_aliases_file() -> PathBuf {
    get_config_dir().join("aliases.json")
}

fn get_agent_file() -> PathBuf {
    get_config_dir().join("agent")
}

fn load_aliases() -> HashMap<String, String> {
    let aliases_file = get_aliases_file();
    if aliases_file.exists() {
        let content = fs::read_to_string(&aliases_file).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

fn save_aliases(aliases: &HashMap<String, String>) -> io::Result<()> {
    ensure_config_dir()?;
    let aliases_file = get_aliases_file();
    let content = serde_json::to_string_pretty(aliases)?;
    fs::write(aliases_file, content)
}

fn get_agent() -> String {
    let agent_file = get_agent_file();
    if agent_file.exists() {
        fs::read_to_string(&agent_file).unwrap_or_else(|_| "claude".to_string()).trim().to_string()
    } else {
        "claude".to_string()
    }
}

fn set_agent(command: &str) -> io::Result<()> {
    ensure_config_dir()?;
    let agent_file = get_agent_file();
    fs::write(agent_file, command)
}

fn read_prompt_from_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

fn get_current_datetime() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Convert to a simple datetime format: YYYYMMDD_HHMMSS
    let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    datetime.format("%Y%m%d_%H%M%S").to_string()
}

fn create_aliases_backup() -> io::Result<Option<String>> {
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

fn confirm_reset() -> bool {
    print!("This will remove all shortcuts (a backup will be created). Are you sure? (y/N): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let input = input.trim().to_lowercase();
        input == "y" || input == "yes"
    } else {
        false
    }
}

#[cfg(test)]
fn parse_agent_args(args: &[String]) -> Result<Vec<String>, String> {
    if args.len() < 2 {
        return Err("Not enough arguments".to_string());
    }
    
    if args.len() == 2 {
        return Ok(vec![]);
    }
    
    if let Some(separator_pos) = args.iter().position(|arg| arg == "--") {
        Ok(args[separator_pos + 1..].to_vec())
    } else {
        Err("Invalid format - use -- to separate agent args".to_string())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Handle direct shortcut execution (qwk foo) or (qwk foo -- agent-args)
    if args.len() >= 2 && !args[1].starts_with("--") {
        let shortcut = &args[1];
        let aliases = load_aliases();
        
        if let Some(prompt) = aliases.get(shortcut) {
            let agent = get_agent();
            
            // Check for -- separator and collect agent arguments
            let mut agent_args = Vec::new();
            if args.len() > 2 {
                if let Some(separator_pos) = args.iter().position(|arg| arg == "--") {
                    // Everything after -- are agent arguments
                    agent_args.extend_from_slice(&args[separator_pos + 1..]);
                } else {
                    // Invalid format - too many args without --
                    eprintln!("Invalid usage. Use 'qwk {} -- <agent-args>' to pass arguments to the agent", shortcut);
                    std::process::exit(1);
                }
            }
            
            // Build command: agent [agent_args] prompt
            let mut cmd = Command::new(&agent);
            for arg in &agent_args {
                cmd.arg(arg);
            }
            cmd.arg(prompt);
            
            let status = cmd.status();
            
            match status {
                Ok(exit_status) => {
                    std::process::exit(exit_status.code().unwrap_or(0));
                }
                Err(e) => {
                    eprintln!("Error executing agent '{}': {}", agent, e);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Shortcut '{}' not found", shortcut);
            std::process::exit(1);
        }
    }
    
    // Parse with clap for other commands
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Set { alias, prompt }) => {
            let prompt_text = if let Some(p) = prompt {
                p
            } else {
                match read_prompt_from_stdin() {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Error reading prompt: {}", e);
                        std::process::exit(1);
                    }
                }
            };
            
            let mut aliases = load_aliases();
            aliases.insert(alias.clone(), prompt_text);
            
            if let Err(e) = save_aliases(&aliases) {
                eprintln!("Error saving alias: {}", e);
                std::process::exit(1);
            }
            
            println!("Alias '{}' set successfully", alias);
        }
        
        Some(Commands::Agent { command }) => {
            if let Err(e) = set_agent(&command) {
                eprintln!("Error setting agent: {}", e);
                std::process::exit(1);
            }
            
            println!("Agent set to '{}'", command);
        }
        
        Some(Commands::Reset) => {
            if !confirm_reset() {
                println!("Reset cancelled.");
                return;
            }
            
            match create_aliases_backup() {
                Ok(Some(backup_path)) => {
                    println!("Backup created: {}", backup_path);
                }
                Ok(None) => {
                    println!("No existing aliases file to backup.");
                }
                Err(e) => {
                    eprintln!("Error creating backup: {}", e);
                    std::process::exit(1);
                }
            }
            
            let aliases_file = get_aliases_file();
            if aliases_file.exists() {
                if let Err(e) = fs::remove_file(&aliases_file) {
                    eprintln!("Error removing aliases file: {}", e);
                    std::process::exit(1);
                }
            }
            
            println!("All shortcuts have been reset.");
        }
        
        None => {
            if let Some(shortcut) = cli.shortcut {
                // This case is handled above, but included for completeness
                eprintln!("Shortcut '{}' not found", shortcut);
                std::process::exit(1);
            } else {
                // Show help if no command provided
                let mut cmd = Cli::command();
                cmd.print_help().unwrap();
            }
        }
    }
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
    fn test_get_current_datetime_format() {
        let datetime = get_current_datetime();
        // Should match YYYYMMDD_HHMMSS format
        assert_eq!(datetime.len(), 15);
        assert!(datetime.contains('_'));
        
        let parts: Vec<&str> = datetime.split('_').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 8); // YYYYMMDD
        assert_eq!(parts[1].len(), 6); // HHMMSS
        
        // Should be all digits except the underscore
        let digits_only = datetime.replace('_', "");
        assert!(digits_only.chars().all(|c| c.is_ascii_digit()));
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

    #[test]
    fn test_agent_args_parsing() {
        // Test cases for argument parsing logic
        let test_cases = vec![
            (vec!["qwk".to_string(), "shortcut".to_string()], (vec![], true)),
            (vec!["qwk".to_string(), "shortcut".to_string(), "--".to_string(), "--flag".to_string()], (vec!["--flag".to_string()], true)),
            (vec!["qwk".to_string(), "shortcut".to_string(), "--".to_string(), "--opt=val".to_string(), "--flag".to_string()], (vec!["--opt=val".to_string(), "--flag".to_string()], true)),
            (vec!["qwk".to_string(), "shortcut".to_string(), "extra".to_string()], (vec![], false)), // Should be invalid
        ];

        for (args, (expected_agent_args, should_be_valid)) in test_cases {
            let result = parse_agent_args(&args);
            match result {
                Ok(agent_args) => {
                    assert!(should_be_valid, "Expected invalid args to fail: {:?}", args);
                    assert_eq!(agent_args, expected_agent_args);
                }
                Err(_) => {
                    assert!(!should_be_valid, "Expected valid args to succeed: {:?}", args);
                }
            }
        }
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

    fn save_aliases_to_file(file_path: &PathBuf, aliases: &HashMap<String, String>) -> io::Result<()> {
        let content = serde_json::to_string_pretty(aliases)?;
        fs::write(file_path, content)
    }
}
