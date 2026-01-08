//! TUI command implementation.

use color_eyre::eyre::Result;

/// Run the TUI command.
pub async fn run() -> Result<()> {
    netdiag_tui::run().await.map_err(|e| color_eyre::eyre::eyre!("{}", e))
}
