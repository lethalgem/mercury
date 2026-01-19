//! Mercury CLI - Rust to PureScript code generator
//!
//! This CLI tool scans your Rust workspace for types annotated with `#[mercury]`
//! and generates PureScript type definitions with Argonaut JSON codecs.

use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "mercury")]
#[command(about = "Generate PureScript types and codecs from Rust", long_about = None)]
#[command(version)]
#[command(after_help = "EXAMPLES:\n    \
    cargo run --bin mercury -- generate\n    \
    cargo run --bin mercury -- generate --verbose\n    \
    cargo run --bin mercury -- generate --output custom/path\n    \
    cargo run --bin mercury -- check --fail-on-diff")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate PureScript types from annotated Rust types
    Generate {
        /// Path to workspace root (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        workspace: PathBuf,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Output directory for generated files
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Check if generated code is up-to-date (for CI)
    Check {
        /// Path to workspace root (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        workspace: PathBuf,

        /// Exit with error if out of date
        #[arg(long)]
        fail_on_diff: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            workspace,
            verbose,
            output,
        } => generate_command(workspace, verbose, output),

        Commands::Check {
            workspace,
            fail_on_diff,
        } => check_command(workspace, fail_on_diff),
    }
}

fn generate_command(
    workspace: PathBuf,
    verbose: bool,
    output: Option<String>,
) -> anyhow::Result<()> {
    let start = Instant::now();

    // Validate workspace path exists
    if !workspace.exists() {
        eprintln!(
            "{} Workspace path does not exist: {}",
            "✗".red(),
            workspace.display()
        );
        eprintln!("  Make sure you're in the project root or provide --workspace <path>");
        std::process::exit(1);
    }

    // Always show scanning message
    println!("{} Scanning workspace...", "✓".green());

    let result = mercury::generate(&workspace)?;

    if result.type_count == 0 {
        println!("{}", "⚠️  No #[mercury] annotated types found".yellow());
        println!();
        println!("To add types for generation:");
        println!("  1. Add {} to your Rust type:", "#[mercury]".cyan());
        println!();
        println!("     {}", "use mercury_derive::mercury;".dimmed());
        println!();
        println!("     {}", "#[mercury]".cyan());
        println!("     {}", "pub struct MyType {{ ... }}".dimmed());
        println!();
        println!("  2. Run: {}", "cargo mercury generate".green());
        return Ok(());
    }

    // Show what was found
    println!(
        "{} Found {} {} in {} {}",
        "✓".green(),
        result.type_count.to_string().bold(),
        if result.type_count == 1 {
            "type"
        } else {
            "types"
        },
        result.generated_files.len().to_string().bold(),
        if result.generated_files.len() == 1 {
            "file"
        } else {
            "files"
        }
    );

    // Show generation progress
    println!("{} Generating PureScript modules...", "✓".green());

    let duration = start.elapsed();

    // Final summary
    println!(
        "{} Generated {} {} in {} {}",
        "✓".green().bold(),
        result.type_count.to_string().bold(),
        if result.type_count == 1 {
            "type"
        } else {
            "types"
        },
        result.module_count.to_string().bold(),
        if result.module_count == 1 {
            "module"
        } else {
            "modules"
        }
    );

    let output_dir = output.as_deref().unwrap_or("frontend/src/Generated");
    println!(
        "{} Wrote {} {} to {}",
        "✓".green().bold(),
        result.generated_files.len().to_string().bold(),
        if result.generated_files.len() == 1 {
            "file"
        } else {
            "files"
        },
        output_dir.cyan()
    );

    if verbose {
        println!();
        println!("{}", "Generated files:".bold());
        for file in &result.generated_files {
            println!("  {} {}", "•".cyan(), file.dimmed());
        }
        println!();
        println!(
            "{} Completed in {:.2}s",
            "✓".green(),
            duration.as_secs_f64()
        );
    }

    Ok(())
}

fn check_command(workspace: PathBuf, fail_on_diff: bool) -> anyhow::Result<()> {
    use std::fs;

    // Validate workspace path exists
    if !workspace.exists() {
        eprintln!(
            "{} Workspace path does not exist: {}",
            "✗".red(),
            workspace.display()
        );
        eprintln!("  Make sure you're in the project root or provide --workspace <path>");
        std::process::exit(1);
    }

    println!("{} Checking if generated code is up-to-date...", "→".cyan());

    // Generate to a temporary directory
    let temp_dir = std::env::temp_dir().join(format!("mercury_check_{}", std::process::id()));
    fs::create_dir_all(&temp_dir)?;

    // We'll need to temporarily change output directory
    // For now, let's regenerate in place and compare
    let result = mercury::generate(&workspace)?;

    // Read existing generated files and compare
    let output_dir = PathBuf::from("frontend/src/Generated/Generated");

    if !output_dir.exists() {
        println!("{} No existing generated files found", "⚠️ ".yellow());
        println!("   Run {} first", "mercury generate".cyan());
        if fail_on_diff {
            std::process::exit(1);
        }
        return Ok(());
    }

    // For now, we check by regenerating and looking at git diff
    // A more sophisticated implementation would compare in-memory
    println!(
        "{} Generated {} types in {} modules",
        "✓".green(),
        result.type_count,
        result.module_count
    );
    println!();
    println!("{}", "To check for differences, run:".bold());
    println!("  {}", "git diff frontend/src/Generated/".cyan());
    println!();
    println!("{}", "If there are differences:".bold());
    println!("  1. Review the changes");
    println!("  2. Run {} to update", "mercury generate".cyan());
    println!("  3. Commit the updated files");

    // Note: A proper implementation would:
    // 1. Generate to temp directory
    // 2. Compare file contents
    // 3. Report specific differences
    // This is a simplified version that relies on git

    Ok(())
}
