//! Build automation tasks for netdiag.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use xshell::{cmd, Shell};

#[derive(Parser)]
#[command(name = "xtask", about = "Build automation for netdiag")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all checks (fmt, clippy, test)
    Check,
    /// Format code
    Fmt,
    /// Run clippy
    Clippy,
    /// Run tests
    Test,
    /// Build release binaries
    Build {
        /// Target platform
        #[arg(short, long)]
        target: Option<String>,
    },
    /// Build for all platforms
    BuildAll,
    /// Generate shell completions
    Completions {
        /// Output directory
        #[arg(short, long, default_value = "completions")]
        output: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    // Change to workspace root
    let workspace_root = project_root();
    sh.change_dir(&workspace_root);

    match cli.command {
        Commands::Check => {
            check(&sh)?;
        }
        Commands::Fmt => {
            fmt(&sh)?;
        }
        Commands::Clippy => {
            clippy(&sh)?;
        }
        Commands::Test => {
            test(&sh)?;
        }
        Commands::Build { target } => {
            build(&sh, target.as_deref())?;
        }
        Commands::BuildAll => {
            build_all(&sh)?;
        }
        Commands::Completions { output } => {
            generate_completions(&sh, &output)?;
        }
    }

    Ok(())
}

fn project_root() -> std::path::PathBuf {
    let dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap());

    // If we're in xtask, go up one level
    if dir.ends_with("xtask") {
        dir.parent().unwrap().to_path_buf()
    } else {
        dir
    }
}

fn check(sh: &Shell) -> Result<()> {
    println!("Running all checks...");
    fmt(sh)?;
    clippy(sh)?;
    test(sh)?;
    println!("All checks passed!");
    Ok(())
}

fn fmt(sh: &Shell) -> Result<()> {
    println!("Checking formatting...");
    cmd!(sh, "cargo fmt --all -- --check")
        .run()
        .context("Format check failed")?;
    Ok(())
}

fn clippy(sh: &Shell) -> Result<()> {
    println!("Running clippy...");
    cmd!(sh, "cargo clippy --workspace --all-targets --all-features -- -D warnings")
        .run()
        .context("Clippy failed")?;
    Ok(())
}

fn test(sh: &Shell) -> Result<()> {
    println!("Running tests...");
    cmd!(sh, "cargo test --workspace --all-features")
        .run()
        .context("Tests failed")?;
    Ok(())
}

fn build(sh: &Shell, target: Option<&str>) -> Result<()> {
    match target {
        Some(t) => {
            println!("Building for target: {}", t);
            cmd!(sh, "cargo build --release --target {t} -p netdiag-cli")
                .run()
                .context("Build failed")?;
        }
        None => {
            println!("Building for current platform...");
            cmd!(sh, "cargo build --release -p netdiag-cli")
                .run()
                .context("Build failed")?;
        }
    }
    Ok(())
}

fn build_all(sh: &Shell) -> Result<()> {
    let targets = [
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc",
    ];

    for target in targets {
        println!("Building for {}...", target);
        if let Err(e) = cmd!(sh, "cargo build --release --target {target} -p netdiag-cli").run() {
            eprintln!("Warning: Failed to build for {}: {}", target, e);
        }
    }

    Ok(())
}

fn generate_completions(sh: &Shell, output: &str) -> Result<()> {
    use std::fs;

    println!("Generating shell completions...");

    fs::create_dir_all(output).context("Failed to create output directory")?;

    let shells = ["bash", "zsh", "fish", "powershell", "elvish"];

    for shell in shells {
        println!("  Generating {} completions...", shell);
        let output_file = format!("{}/netdiag.{}", output, shell);
        cmd!(sh, "cargo run -p netdiag-cli -- completions {shell}")
            .read()
            .map(|content| fs::write(&output_file, content))
            .context(format!("Failed to generate {} completions", shell))??;
    }

    println!("Completions generated in {}/", output);
    Ok(())
}
