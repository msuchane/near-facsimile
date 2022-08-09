use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the root documentation directory
    #[clap(value_parser, value_name = "DIR", default_value = ".")]
    pub path: PathBuf,

    /// The similarity fraction above which to report files
    #[clap(short, long, value_name = "DECIMAL", default_value = "0.8")]
    pub threshold: f64,

    /// Use a faster but less precise comparison method
    #[clap(short, long, action)]
    pub fast: bool,

    /// Display progress and debugging information
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Path to the output CSV file
    #[clap(short, long, value_name = "FILE", default_value = "comparisons.csv")]
    pub csv_path: PathBuf,
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

/// Parse the current command-line options.
#[must_use]
pub fn options() -> Cli {
    Cli::parse()
}
