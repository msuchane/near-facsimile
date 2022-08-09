use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Optional: Path to the root documentation directory
    #[clap(value_parser, value_name = "DIR", default_value = ".")]
    pub path: PathBuf,

    // Sets a custom config file
    //#[clap(short, long, value_parser, value_name = "FILE")]
    //config: Option<PathBuf>,
    /// Display debugging information
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    //#[clap(subcommand)]
    //command: Option<Commands>,
    /// The similarity fraction above which to report files
    #[clap(short, long, value_name = "DECIMAL", default_value = "0.8")]
    pub threshold: f64,
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
