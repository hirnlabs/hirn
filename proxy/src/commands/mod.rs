mod activate;
mod add;
mod deactivate;
mod list;
mod run;
mod skill;

use crate::caller::OutputMode;
use crate::cli::Commands;

/// Route a parsed subcommand to its handler and render the verdict.
///
/// Returns the exact string `main` prints, so dispatch stays testable
/// without capturing stdout.
pub fn dispatch(command: &Commands, mode: OutputMode) -> String {
    match command {
        Commands::List => list::render(mode),
        Commands::Run { tool, args } => run::render(tool, args, mode),
        Commands::Skill { name } => skill::render(name, mode),
        Commands::Add { source } => add::render(source, mode),
        Commands::Activate { name } => activate::render(name, mode),
        Commands::Deactivate { name } => deactivate::render(name, mode),
    }
}
