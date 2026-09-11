pub mod activate;
pub mod add;
pub mod deactivate;
pub mod list;
pub mod run;
pub mod skill;

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
        Commands::Add { source, local, only } => {
            add::run(source, *local, only.as_deref(), mode, &add::RealAddEnv)
        }
        Commands::Activate { name } => activate::render(name, mode),
        Commands::Deactivate { name } => deactivate::render(name, mode),
    }
}
