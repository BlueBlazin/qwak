use std::io::{self, Write};

pub fn parse_agent_command(agent_str: &str) -> (String, Vec<String>) {
    match shlex::split(agent_str) {
        Some(parts) if !parts.is_empty() => {
            let command = parts[0].clone();
            let args = parts[1..].to_vec();
            (command, args)
        }
        _ => (agent_str.to_string(), vec![]),
    }
}

pub fn truncate_prompt(prompt: &str, max_length: usize) -> String {
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

pub fn get_current_datetime() -> String {
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

pub fn confirm_reset() -> bool {
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

pub fn read_prompt_from_stdin() -> io::Result<String> {
    use std::io::Read;
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

#[cfg(test)]
pub fn parse_agent_args(args: &[String]) -> Result<Vec<String>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
