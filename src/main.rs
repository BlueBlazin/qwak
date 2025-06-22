use clap::{CommandFactory, Parser, Subcommand};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::Command;

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
    #[command(
        long_about = "Set an alias for a prompt. If no prompt is provided, it will be read from stdin."
    )]
    Set {
        #[arg(help = "The alias name to set")]
        alias: String,
        #[arg(help = "The prompt text (optional, will read from stdin if not provided)")]
        prompt: Option<String>,
    },
    #[command(long_flag = "agent")]
    #[command(about = "Set the agent command to use")]
    #[command(
        long_about = "Set the agent command to use when executing shortcuts. Can include default arguments that will be passed on every call. Defaults to 'claude'."
    )]
    Agent {
        #[arg(help = "The command to use as the agent (can include default arguments in quotes)")]
        command: String,
    },
    #[command(long_flag = "list")]
    #[command(about = "List all available shortcuts")]
    #[command(
        long_about = "List all available shortcuts with their alias names and a preview of their associated prompts."
    )]
    List,
    #[command(long_flag = "remove")]
    #[command(about = "Remove a specific shortcut")]
    #[command(
        long_about = "Remove a specific shortcut by alias name. The shortcut will be permanently deleted from the aliases file."
    )]
    Remove {
        #[arg(help = "The alias name to remove")]
        alias: String,
    },
    #[command(long_flag = "reset")]
    #[command(about = "Reset all shortcuts (creates backup)")]
    #[command(
        long_about = "Reset all shortcuts by clearing the aliases file. A backup will be created automatically. The agent setting is preserved."
    )]
    Reset,
    #[command(long_flag = "complete")]
    #[command(about = "Generate completions (internal use)")]
    #[command(
        long_about = "Generate completions for the given partial input. This is used internally by shell completion scripts."
    )]
    #[command(hide = true)]
    Complete {
        #[arg(help = "Partial input to complete")]
        partial: Option<String>,
    },
    #[command(long_flag = "setup-completion")]
    #[command(about = "Set up shell autocompletion")]
    #[command(
        long_about = "Set up autocompletion for your current shell. This will modify your shell's configuration file."
    )]
    SetupCompletion,
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
        fs::read_to_string(&agent_file)
            .unwrap_or_else(|_| "claude".to_string())
            .trim()
            .to_string()
    } else {
        "claude".to_string()
    }
}

fn parse_agent_command(agent_str: &str) -> (String, Vec<String>) {
    match shlex::split(agent_str) {
        Some(parts) if !parts.is_empty() => {
            let command = parts[0].clone();
            let args = parts[1..].to_vec();
            (command, args)
        }
        _ => (agent_str.to_string(), vec![]),
    }
}

fn truncate_prompt(prompt: &str, max_length: usize) -> String {
    // Replace newlines and multiple spaces with single spaces for display
    let cleaned = prompt
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if cleaned.len() <= max_length {
        cleaned
    } else {
        format!("{}...", &cleaned[..max_length.saturating_sub(3)])
    }
}

fn list_aliases() {
    let aliases = load_aliases();

    if aliases.is_empty() {
        println!("No shortcuts available.");
        return;
    }

    println!("Available shortcuts:");

    // Sort aliases by name for consistent output
    let mut sorted_aliases: Vec<_> = aliases.iter().collect();
    sorted_aliases.sort_by_key(|(name, _)| *name);

    for (alias, prompt) in sorted_aliases {
        let truncated_prompt = truncate_prompt(prompt, 60);
        println!("  {} - {}", alias, truncated_prompt);
    }
}

fn generate_completions(partial: Option<String>) {
    let aliases = load_aliases();
    let mut completions = Vec::new();

    // Add command completions
    let commands = vec![
        "--set",
        "--agent",
        "--list",
        "--remove",
        "--reset",
        "--setup-completion",
        "--help",
    ];

    // Add alias completions
    for alias in aliases.keys() {
        completions.push(alias.as_str());
    }

    // Add command completions
    completions.extend(commands);

    // Filter by partial input if provided
    if let Some(partial_input) = partial {
        if !partial_input.is_empty() {
            completions.retain(|completion| completion.starts_with(&partial_input));
        }
    }

    // Sort and output
    completions.sort();
    for completion in completions {
        println!("{}", completion);
    }
}

#[derive(Debug)]
enum Shell {
    Bash,
    Zsh,
    Fish,
}

fn detect_shell() -> Option<Shell> {
    if let Ok(shell) = env::var("SHELL") {
        if shell.contains("bash") {
            Some(Shell::Bash)
        } else if shell.contains("zsh") {
            Some(Shell::Zsh)
        } else if shell.contains("fish") {
            Some(Shell::Fish)
        } else {
            None
        }
    } else {
        None
    }
}

fn get_completion_script(shell: &Shell) -> String {
    match shell {
        Shell::Bash => r#"
_qwk_complete() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    COMPREPLY=($(qwk --complete "$cur" 2>/dev/null))
}
complete -F _qwk_complete qwk
"#
        .to_string(),
        Shell::Zsh => r#"
_qwk_complete() {
    local completions
    completions=($(qwk --complete "$1" 2>/dev/null))
    compadd -a completions
}
compdef _qwk_complete qwk
"#
        .to_string(),
        Shell::Fish => r#"
function __qwk_complete
    qwk --complete (commandline -ct) 2>/dev/null
end
complete -c qwk -f -a "(__qwk_complete)"
"#
        .to_string(),
    }
}

fn get_shell_rc_file(shell: &Shell) -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    let home_path = PathBuf::from(home);

    match shell {
        Shell::Bash => {
            // Try .bashrc first, then .bash_profile
            let bashrc = home_path.join(".bashrc");
            if bashrc.exists() {
                Some(bashrc)
            } else {
                Some(home_path.join(".bash_profile"))
            }
        }
        Shell::Zsh => Some(home_path.join(".zshrc")),
        Shell::Fish => {
            let fish_config_dir = home_path.join(".config/fish");
            fs::create_dir_all(&fish_config_dir).ok()?;
            Some(fish_config_dir.join("config.fish"))
        }
    }
}

fn is_completion_installed(shell: &Shell) -> bool {
    let rc_file = match get_shell_rc_file(shell) {
        Some(file) => file,
        None => return false,
    };

    if !rc_file.exists() {
        return false;
    }

    let content = fs::read_to_string(&rc_file).unwrap_or_default();
    content.contains("_qwk_complete") || content.contains("__qwk_complete")
}

fn install_completion(shell: &Shell) -> io::Result<()> {
    let rc_file = get_shell_rc_file(shell).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Could not determine shell RC file")
    })?;

    let completion_script = get_completion_script(shell);
    let comment = format!("# qwk autocompletion setup");
    let full_addition = format!("{}\n{}", comment, completion_script);

    // Append to RC file
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc_file)?;

    writeln!(file, "{}", full_addition)?;

    Ok(())
}

fn setup_completion_for_current_shell() -> io::Result<()> {
    let shell = detect_shell()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not detect current shell"))?;

    if is_completion_installed(&shell) {
        println!("Autocompletion is already set up for {:?}", shell);
        return Ok(());
    }

    install_completion(&shell)?;

    let shell_name = match shell {
        Shell::Bash => "bash",
        Shell::Zsh => "zsh",
        Shell::Fish => "fish",
    };

    println!("Autocompletion set up for {}!", shell_name);
    match shell {
        Shell::Fish => {
            println!("Restart your shell or run 'source ~/.config/fish/config.fish' to activate.")
        }
        _ => println!(
            "Restart your shell or run 'source ~/.{}rc' to activate.",
            shell_name
        ),
    }

    Ok(())
}

fn is_first_run() -> bool {
    let config_dir = get_config_dir();
    let first_run_marker = config_dir.join(".first_run_complete");
    !first_run_marker.exists()
}

fn mark_first_run_complete() -> io::Result<()> {
    ensure_config_dir()?;
    let first_run_marker = get_config_dir().join(".first_run_complete");
    fs::write(first_run_marker, "")?;
    Ok(())
}

fn handle_first_run() {
    if is_first_run() {
        println!("Welcome to qwk! Setting up autocompletion...");
        if let Err(e) = setup_completion_for_current_shell() {
            eprintln!("Note: Could not set up autocompletion automatically: {}", e);
            eprintln!("You can set it up manually later with: qwk --setup-completion");
        }
        if let Err(e) = mark_first_run_complete() {
            eprintln!("Warning: Could not mark first run as complete: {}", e);
        }
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
    let datetime =
        chrono::DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_else(|| chrono::Utc::now());
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

    // Handle first run setup (but not for completion calls)
    if args.len() < 2 || !args[1].contains("complete") {
        handle_first_run();
    }

    // Handle direct shortcut execution (qwk foo) or (qwk foo -- agent-args)
    if args.len() >= 2 && !args[1].starts_with("--") {
        let shortcut = &args[1];
        let aliases = load_aliases();

        if let Some(prompt) = aliases.get(shortcut) {
            let agent_str = get_agent();
            let (agent_command, agent_default_args) = parse_agent_command(&agent_str);

            // Check for -- separator and collect per-call agent arguments
            let mut per_call_args = Vec::new();
            if args.len() > 2 {
                if let Some(separator_pos) = args.iter().position(|arg| arg == "--") {
                    // Everything after -- are per-call agent arguments
                    per_call_args.extend_from_slice(&args[separator_pos + 1..]);
                } else {
                    // Invalid format - too many args without --
                    eprintln!(
                        "Invalid usage. Use 'qwk {} -- <agent-args>' to pass arguments to the agent",
                        shortcut
                    );
                    std::process::exit(1);
                }
            }

            // Build command: agent [default_args] [per_call_args] prompt
            let mut cmd = Command::new(&agent_command);
            for arg in &agent_default_args {
                cmd.arg(arg);
            }
            for arg in &per_call_args {
                cmd.arg(arg);
            }
            cmd.arg(prompt);

            let status = cmd.status();

            match status {
                Ok(exit_status) => {
                    std::process::exit(exit_status.code().unwrap_or(0));
                }
                Err(e) => {
                    eprintln!("Error executing agent '{}': {}", agent_command, e);
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

        Some(Commands::List) => {
            list_aliases();
        }

        Some(Commands::Complete { partial }) => {
            generate_completions(partial);
        }

        Some(Commands::SetupCompletion) => {
            if let Err(e) = setup_completion_for_current_shell() {
                eprintln!("Error setting up autocompletion: {}", e);
                std::process::exit(1);
            }
        }

        Some(Commands::Remove { alias }) => {
            let mut aliases = load_aliases();

            if aliases.remove(&alias).is_some() {
                if let Err(e) = save_aliases(&aliases) {
                    eprintln!("Error saving aliases after removal: {}", e);
                    std::process::exit(1);
                }
                println!("Shortcut '{}' removed successfully", alias);
            } else {
                println!("Shortcut '{}' does not exist", alias);
            }
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
    fn test_truncate_prompt() {
        // Test short prompt (no truncation)
        assert_eq!(truncate_prompt("Short prompt", 50), "Short prompt");

        // Test exact length (no truncation)
        assert_eq!(
            truncate_prompt("Exactly fifty characters long for testing here", 47),
            "Exactly fifty characters long for testing here"
        );

        // Test long prompt (with truncation)
        assert_eq!(
            truncate_prompt("This is a very long prompt that should be truncated", 20),
            "This is a very lo..."
        );

        // Test prompt with newlines (should be cleaned)
        assert_eq!(
            truncate_prompt("Line one\nLine two\nLine three", 50),
            "Line one Line two Line three"
        );

        // Test prompt with multiple spaces (should be cleaned)
        assert_eq!(
            truncate_prompt("Multiple    spaces   should   be   cleaned", 50),
            "Multiple spaces should be cleaned"
        );

        // Test empty prompt
        assert_eq!(truncate_prompt("", 10), "");

        // Test very short max_length
        assert_eq!(truncate_prompt("Hello world", 5), "He...");
    }

    #[test]
    fn test_parse_agent_command() {
        let test_cases = vec![
            ("claude", ("claude".to_string(), vec![])),
            (
                "claude --flag",
                ("claude".to_string(), vec!["--flag".to_string()]),
            ),
            (
                "claude --opt=value --flag",
                (
                    "claude".to_string(),
                    vec!["--opt=value".to_string(), "--flag".to_string()],
                ),
            ),
            (
                "\"quoted command\" arg",
                ("quoted command".to_string(), vec!["arg".to_string()]),
            ),
            ("", ("".to_string(), vec![])),
        ];

        for (input, (expected_command, expected_args)) in test_cases {
            let (command, args) = parse_agent_command(input);
            assert_eq!(
                command, expected_command,
                "Command mismatch for input: {}",
                input
            );
            assert_eq!(args, expected_args, "Args mismatch for input: {}", input);
        }
    }

    #[test]
    fn test_shell_detection() {
        // Test bash detection
        unsafe {
            std::env::set_var("SHELL", "/bin/bash");
        }
        if let Some(shell) = detect_shell() {
            assert!(matches!(shell, Shell::Bash));
        }

        // Test zsh detection
        unsafe {
            std::env::set_var("SHELL", "/usr/local/bin/zsh");
        }
        if let Some(shell) = detect_shell() {
            assert!(matches!(shell, Shell::Zsh));
        }

        // Test fish detection
        unsafe {
            std::env::set_var("SHELL", "/usr/bin/fish");
        }
        if let Some(shell) = detect_shell() {
            assert!(matches!(shell, Shell::Fish));
        }
    }

    #[test]
    fn test_completion_script_generation() {
        let bash_script = get_completion_script(&Shell::Bash);
        assert!(bash_script.contains("_qwk_complete"));
        assert!(bash_script.contains("COMP_WORDS"));

        let zsh_script = get_completion_script(&Shell::Zsh);
        assert!(zsh_script.contains("_qwk_complete"));
        assert!(zsh_script.contains("compdef"));

        let fish_script = get_completion_script(&Shell::Fish);
        assert!(fish_script.contains("__qwk_complete"));
        assert!(fish_script.contains("commandline"));
    }

    #[test]
    fn test_agent_args_parsing() {
        // Test cases for argument parsing logic
        let test_cases = vec![
            (
                vec!["qwk".to_string(), "shortcut".to_string()],
                (vec![], true),
            ),
            (
                vec![
                    "qwk".to_string(),
                    "shortcut".to_string(),
                    "--".to_string(),
                    "--flag".to_string(),
                ],
                (vec!["--flag".to_string()], true),
            ),
            (
                vec![
                    "qwk".to_string(),
                    "shortcut".to_string(),
                    "--".to_string(),
                    "--opt=val".to_string(),
                    "--flag".to_string(),
                ],
                (vec!["--opt=val".to_string(), "--flag".to_string()], true),
            ),
            (
                vec![
                    "qwk".to_string(),
                    "shortcut".to_string(),
                    "extra".to_string(),
                ],
                (vec![], false),
            ), // Should be invalid
        ];

        for (args, (expected_agent_args, should_be_valid)) in test_cases {
            let result = parse_agent_args(&args);
            match result {
                Ok(agent_args) => {
                    assert!(should_be_valid, "Expected invalid args to fail: {:?}", args);
                    assert_eq!(agent_args, expected_agent_args);
                }
                Err(_) => {
                    assert!(
                        !should_be_valid,
                        "Expected valid args to succeed: {:?}",
                        args
                    );
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

    fn save_aliases_to_file(
        file_path: &PathBuf,
        aliases: &HashMap<String, String>,
    ) -> io::Result<()> {
        let content = serde_json::to_string_pretty(aliases)?;
        fs::write(file_path, content)
    }
}
