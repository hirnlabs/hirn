use clap::{Parser, Subcommand};

/// hirn-proxy: resolve tool calls into summary + policy verdict.
///
/// Never executes tools on behalf of the agent. Stub (HIR-55 scaffold).
#[derive(Debug, Parser)]
#[command(name = "hirn-proxy", version)]
pub struct Cli {
    /// Force JSON output (override caller detection).
    #[arg(long, global = true)]
    pub json: bool,

    /// Force pretty output with prompts (override caller detection).
    #[arg(long, global = true)]
    pub interactive: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List available tools.
    List,
    /// Resolve a tool call (no execution).
    Run {
        /// Tool name.
        tool: String,
        /// JSON args string.
        args: String,
    },
    /// Show a skill.
    Skill {
        /// Skill name.
        name: String,
    },
    /// Add a tool from a source.
    Add {
        /// Source (path or URL).
        source: String,
    },
    /// Activate a tool.
    Activate {
        /// Tool name.
        name: String,
    },
    /// Deactivate a tool.
    Deactivate {
        /// Tool name.
        name: String,
    },
}
