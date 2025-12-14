// modern-cli-mcp/src/main.rs
mod groups;
mod ignore;
mod state;
mod tools;

use anyhow::Result;
use clap::Parser;
use groups::AgentProfile;
use rmcp::{transport::stdio, ServiceExt};
use tools::ModernCliTools;
use tracing_subscriber::{self, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "modern-cli-mcp")]
#[command(author, version, about = "MCP server exposing modern CLI tools", long_about = None)]
struct Args {
    /// Agent profile for tool selection (reduces cognitive load).
    /// Available: explore, architect, review, test, generator, reflector,
    /// curator, docs, lint, api, dev-deploy, full
    #[arg(short, long)]
    profile: Option<String>,

    /// List available profiles and exit.
    #[arg(long)]
    list_profiles: bool,

    /// List available tool groups and exit.
    #[arg(long)]
    list_groups: bool,
}

fn print_profiles() {
    println!("Available Agent Profiles:\n");
    println!("{:<12} {:<6} DESCRIPTION", "PROFILE", "TOOLS");
    println!("{}", "-".repeat(80));
    for profile in AgentProfile::ALL {
        println!(
            "{:<12} {:<6} {}",
            profile.id(),
            profile.pre_expanded_tool_count(),
            profile.description()
        );
    }
    println!("\nUsage: modern-cli-mcp --profile <PROFILE>");
}

fn print_groups() {
    use groups::ToolGroup;

    println!("Available Tool Groups:\n");
    println!("{:<12} {:<6} DESCRIPTION", "GROUP", "TOOLS");
    println!("{}", "-".repeat(100));
    for group in ToolGroup::ALL {
        println!(
            "{:<12} {:<6} {}",
            group.id(),
            group.tool_count(),
            group.description()
        );
    }
    println!(
        "\nTotal: {} tools across {} groups",
        ToolGroup::ALL.iter().map(|g| g.tool_count()).sum::<usize>(),
        ToolGroup::ALL.len()
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle info commands
    if args.list_profiles {
        print_profiles();
        return Ok(());
    }

    if args.list_groups {
        print_groups();
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // Parse profile if provided
    let profile = if let Some(p) = args.profile {
        match p.parse::<AgentProfile>() {
            Ok(profile) => {
                tracing::info!(
                    "Using profile: {} ({} tools pre-expanded)",
                    profile.id(),
                    profile.pre_expanded_tool_count()
                );
                Some(profile)
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                eprintln!("\nRun with --list-profiles to see available profiles.");
                std::process::exit(1);
            }
        }
    } else {
        tracing::info!("No profile specified, using virtual tool groups (expand on demand)");
        None
    };

    tracing::info!("Starting Modern CLI Tools MCP server");

    let service = ModernCliTools::new(profile)
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Server error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
