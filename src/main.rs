use clap::{Parser, Subcommand, CommandFactory};
use std::env;
use std::fs;
use std::io::{self, Read};
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
