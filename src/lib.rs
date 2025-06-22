pub mod cli;
pub mod completion;
pub mod config;
pub mod utils;

pub use cli::{Cli, Commands, run};
pub use completion::{
    Shell, generate_completions, handle_first_run, setup_completion_for_current_shell,
};
pub use config::{
    create_aliases_backup, ensure_config_dir, get_agent, get_aliases_file, get_config_dir,
    load_aliases, save_aliases, set_agent,
};
pub use utils::{confirm_reset, get_current_datetime, parse_agent_command, truncate_prompt};
