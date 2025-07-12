use clap::Parser;
use ipak::prelude::ipak::args::CommandExecution;
use ipak::utils::args::Args;
use ipak::utils::error::Error;
use log::LevelFilter;

/// The main function of the `ipak` CLI application.
///
/// This function parses command-line arguments, dispatches to the appropriate
/// subcommand handler, and returns a `Result` indicating success or an `Error`.
fn main() -> Result<(), Error> {
    let args = Args::parse();

    let mut log_builder = env_logger::builder();

    if args.quiet {
        log_builder.filter_level(LevelFilter::Off);
    } else if args.debug {
        log_builder.filter_level(LevelFilter::Debug);
    } else if args.verbose {
        log_builder.filter_level(LevelFilter::Info);
    } else {
        log_builder.filter_level(LevelFilter::Warn);
    }

    log_builder.init();
    args.command.exec()
}
