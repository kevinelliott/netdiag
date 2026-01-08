//! Completions command implementation.

use crate::app::{Cli, CompletionsArgs};
use clap::CommandFactory;
use clap_complete::generate;
use color_eyre::eyre::Result;

/// Run the completions command.
pub fn run(args: CompletionsArgs) -> Result<()> {
    let mut cmd = Cli::command();
    let shell: clap_complete::Shell = args.shell.into();
    generate(shell, &mut cmd, "netdiag", &mut std::io::stdout());
    Ok(())
}
