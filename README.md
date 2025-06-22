# Qwk

A command-line tool for creating shortcuts to AI agent prompts. Store complex prompts as simple aliases and execute them with your preferred AI agent.

## Overview

Qwk allows you to save frequently used AI prompts as shortcuts and execute them quickly. Instead of retyping long prompts or searching through your history, create an alias once and run it with a simple command.

## Installation

```bash
cargo install qwk
```

Or build from source:

```bash
git clone https://github.com/your-username/qwk.git
cd qwk
cargo build --release
```

## Quick Start

1. **Set an alias for a prompt:**
   ```bash
   qwk --set react-setup "Create a new React project with Vite, install styled-components, and set up a basic component structure"
   ```

2. **Configure your AI agent (defaults to 'claude'):**
   ```bash
   qwk --agent claude
   ```

3. **Run your shortcut:**
   ```bash
   qwk react-setup
   ```

## Usage

### Creating Shortcuts

Set a shortcut with an inline prompt:
```bash
qwk --set my-alias "Your prompt here"
```

Set a shortcut by piping from a file:
```bash
cat my-prompt.txt | qwk --set my-alias
```

Set a shortcut interactively (paste your prompt):
```bash
qwk --set my-alias
# Paste your prompt and press Ctrl+D (Unix) or Ctrl+Z (Windows)
```

### Running Shortcuts

Execute a shortcut:
```bash
qwk my-alias
```

Execute with additional agent arguments:
```bash
qwk my-alias -- --temperature=0.7 --max-tokens=1000
```

### Configuration

Set the AI agent command (default: `claude`):
```bash
qwk --agent your-agent-command
```

Reset all shortcuts (creates backup):
```bash
qwk --reset
```

## Configuration Files

Qwk stores its configuration in `~/.config/qwk/`:
- `aliases.json` - Your shortcuts and prompts
- `agent` - Your configured AI agent command
- `aliases_backup_YYYYMMDD_HHMMSS.json` - Automatic backups when resetting

## Examples

**Web development:**
```bash
qwk --set debug-css "Help me debug this CSS layout issue. Look at the HTML and CSS and suggest fixes for alignment problems."
qwk debug-css
```

**Code review:**
```bash
qwk --set review "Review this code for potential bugs, performance issues, and suggest improvements."
qwk review -- --model=claude-3-sonnet
```

**Documentation:**
```bash
qwk --set docs "Generate comprehensive documentation for this code including usage examples."
qwk docs
```

## Commands

| Command | Description |
|---------|-------------|
| `qwk <alias>` | Execute a saved shortcut |
| `qwk <alias> -- <args>` | Execute shortcut with agent arguments |
| `qwk --set <alias> [prompt]` | Create or update a shortcut |
| `qwk --agent <command>` | Set the AI agent command |
| `qwk --reset` | Reset all shortcuts (with backup) |
| `qwk --help` | Show help information |

## Requirements

- Rust 1.70+ (for building from source)
- An AI agent command-line tool (like `claude`, `gpt`, etc.)

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.