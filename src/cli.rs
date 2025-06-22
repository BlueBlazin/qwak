use clap::{CommandFactory, Parser, Subcommand};
use std::env;
use std::fs;
use std::process::Command;

use crate::completion::{
    generate_completions, handle_first_run, setup_completion_for_current_shell,
};
use crate::config::{
    create_aliases_backup, get_agent, get_aliases_file, load_aliases, save_aliases, set_agent,
};
use crate::utils::{confirm_reset, parse_agent_command, read_prompt_from_stdin, truncate_prompt};

#[derive(Parser)]
#[command(name = "qwk")]
#[command(about = "A CLI tool for creating aliases for AI agents")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(help = "Run a stored shortcut")]
    pub shortcut: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
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

pub fn list_aliases() {
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

pub fn execute_shortcut(shortcut: &str, args: &[String]) {
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

pub fn run() {
    let args: Vec<String> = env::args().collect();

    // Handle first run setup (but not for completion calls)
    if args.len() < 2 || !args[1].contains("complete") {
        handle_first_run();
    }

    // Handle direct shortcut execution (qwk foo) or (qwk foo -- agent-args)
    if args.len() >= 2 && !args[1].starts_with("--") {
        let shortcut = &args[1];
        execute_shortcut(shortcut, &args);
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
