# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Qwk is a command line tool for creating aliases for AI agents to perform predefined tasks. It allows users to store prompts and execute them through agent interfaces like Claude Code.

## Common Commands

```bash
# Build the project
cargo build

# Run the project
cargo run

# Build for release
cargo build --release

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Architecture

This is a Rust CLI application built with Cargo. The main functionality includes:

- **Command aliases**: Store and retrieve predefined AI agent prompts
- **Agent integration**: Execute prompts through different AI agent interfaces
- **Prompt management**: Set prompts via command line, stdin, or piped input

Key command patterns:
- `qwk <alias>` - Execute a stored prompt
- `qwk --set <alias> "<prompt>"` - Store a new prompt alias
- `qwk --agent <agent-name>` - Configure the agent to use

The current implementation is minimal (Hello World), indicating this is an early-stage project that needs the core CLI functionality implemented.