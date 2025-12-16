// modern-cli-mcp/src/main.rs
mod cli;
mod format;
mod groups;
mod ignore;
mod state;
mod tools;

use anyhow::Result;
use clap::Parser;
use groups::{AgentProfile, ToolGroup};
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

    /// Enable dynamic toolsets (beta). Starts with only meta-tools exposed.
    /// Use enable_toolset to activate tool groups on demand.
    #[arg(long, env = "MCP_DYNAMIC_TOOLSETS")]
    dynamic_toolsets: bool,

    /// Tool groups to pre-enable with --dynamic-toolsets (comma-separated).
    /// Example: --toolsets filesystem,git,search
    #[arg(long, env = "MCP_TOOLSETS", value_delimiter = ',')]
    toolsets: Option<Vec<String>>,

    /// List available profiles and exit.
    #[arg(long)]
    list_profiles: bool,

    /// List available tool groups and exit.
    #[arg(long)]
    list_groups: bool,

    /// List tools available for direct execution (busybox-style).
    #[arg(long)]
    list_tools: bool,

    /// Enable dual-response mode. Tools return both formatted summary (for humans)
    /// and raw structured data (for LLM processing) in a single response.
    #[arg(long, env = "MCP_DUAL_RESPONSE")]
    dual_response: bool,
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
    // Check for direct tool execution before parsing clap args
    // This allows `modern-cli-mcp eza -la` to work without clap interference
    let raw_args: Vec<String> = std::env::args().collect();
    if raw_args.len() > 1 {
        let first_arg = &raw_args[1];
        // Skip if it looks like a flag
        if !first_arg.starts_with('-') && cli::is_known_tool(first_arg) {
            cli::run_tool_directly(first_arg, &raw_args[2..]);
        }
    }

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

    if args.list_tools {
        cli::print_tools();
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // Parse profile if provided (mutually exclusive with dynamic_toolsets)
    let profile = if args.dynamic_toolsets {
        if args.profile.is_some() {
            eprintln!("Warning: --profile is ignored when --dynamic-toolsets is enabled");
        }
        None
    } else if let Some(p) = args.profile {
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

    // Parse pre-enabled toolsets for dynamic mode
    let pre_enabled_toolsets: Vec<ToolGroup> = if args.dynamic_toolsets {
        args.toolsets
            .unwrap_or_default()
            .iter()
            .filter_map(|s| match s.parse::<ToolGroup>() {
                Ok(g) => Some(g),
                Err(e) => {
                    eprintln!("Warning: {}", e);
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    if args.dynamic_toolsets {
        if pre_enabled_toolsets.is_empty() {
            tracing::info!(
                "Dynamic toolsets enabled. Starting with meta-tools only. Use enable_toolset to activate groups."
            );
        } else {
            tracing::info!(
                "Dynamic toolsets enabled with {} pre-enabled groups: {}",
                pre_enabled_toolsets.len(),
                pre_enabled_toolsets
                    .iter()
                    .map(|g| g.id())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    if args.dual_response {
        tracing::info!("Dual-response mode enabled (formatted + raw data)");
    }

    tracing::info!("Starting Modern CLI Tools MCP server");

    let service = ModernCliTools::new_with_config(
        profile,
        args.dynamic_toolsets,
        pre_enabled_toolsets,
        args.dual_response,
    )
    .serve(stdio())
    .await
    .inspect_err(|e| {
        tracing::error!("Server error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
