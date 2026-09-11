use clap::Parser;
use hirn_proxy::{caller, cli::Cli, commands};

fn main() {
    let cli = Cli::parse();
    let mode = caller::detect_output_mode(cli.json, cli.interactive);
    println!("{}", commands::dispatch(&cli.command, mode));
}
