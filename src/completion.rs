use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::config::{ensure_config_dir, get_config_dir, load_aliases};

#[derive(Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

pub fn generate_completions(partial: Option<String>) {
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

pub fn detect_shell() -> Option<Shell> {
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

pub fn get_completion_script(shell: &Shell) -> String {
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

pub fn get_shell_rc_file(shell: &Shell) -> Option<PathBuf> {
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

pub fn is_completion_installed(shell: &Shell) -> bool {
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

pub fn install_completion(shell: &Shell) -> io::Result<()> {
    let rc_file = get_shell_rc_file(shell).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Could not determine shell RC file")
    })?;

    let completion_script = get_completion_script(shell);
    let comment = "# qwk autocompletion setup";
    let full_addition = format!("{}\n{}", comment, completion_script);

    // Append to RC file
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&rc_file)?;

    writeln!(file, "{}", full_addition)?;

    Ok(())
}

pub fn setup_completion_for_current_shell() -> io::Result<()> {
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

pub fn is_first_run() -> bool {
    let config_dir = get_config_dir();
    let first_run_marker = config_dir.join(".first_run_complete");
    !first_run_marker.exists()
}

pub fn mark_first_run_complete() -> io::Result<()> {
    ensure_config_dir()?;
    let first_run_marker = get_config_dir().join(".first_run_complete");
    fs::write(first_run_marker, "")?;
    Ok(())
}

pub fn handle_first_run() {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
