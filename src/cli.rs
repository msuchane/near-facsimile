use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Optional: Path to the root documentation directory
    #[clap(value_parser, value_name = "DIR")]
    pub path: Option<PathBuf>,

    // Sets a custom config file
    //#[clap(short, long, value_parser, value_name = "FILE")]
    //config: Option<PathBuf>,
    /// Display debugging information
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    //#[clap(subcommand)]
    //command: Option<Commands>,
}

/*
#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[clap(short, long, action)]
        list: bool,
    },
}
*/

pub fn options() -> Cli {
    Cli::parse()
}
