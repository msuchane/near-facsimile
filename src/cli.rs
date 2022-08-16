use std::ffi::OsString;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the root documentation directory
    #[clap(short, long, value_parser, value_name = "DIR", default_value = ".")]
    pub path: PathBuf,

    /// The similarity fraction above which to report files
    #[clap(short, long, value_name = "DECIMAL", default_value = "0.8")]
    pub threshold: f64,

    /// Use a faster but less precise comparison method
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub fast: u8,

    /// Display progress and debugging information
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Path to the output CSV file
    #[clap(short, long, value_name = "FILE", default_value = "comparisons.csv")]
    pub csv: PathBuf,

    /// Path to the output JSON file
    #[clap(short, long, value_name = "FILE", default_value = "comparisons.json")]
    pub json: PathBuf,

    /// Ignore these file names in the search and comparison
    #[clap(long, value_name = "NAMES")]
    pub ignore_files: Vec<OsString>,

    /// Ignore these file extensions in the search and comparison
    #[clap(long, value_name = "EXTENSIONS")]
    pub ignore_ext: Vec<OsString>,

    /// Look for these file names in the search and comparison
    #[clap(long, value_name = "NAMES")]
    pub require_files: Vec<OsString>,

    /// Look for these file extensions in the search and comparison
    #[clap(long, value_name = "EXTENSIONS")]
    pub require_ext: Vec<OsString>,
}

/// Parse the current command-line options.
#[must_use]
pub fn options() -> Cli {
    Cli::parse()
}
