/*
Copyright 2022 Marek Suchánek <msuchane@redhat.com>

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::ffi::OsString;
use std::path::PathBuf;

use clap::Parser;
use regex::Regex;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the root documentation directory
    #[arg(short, long, value_parser, value_name = "DIR", default_value = ".")]
    pub path: PathBuf,

    /// The similarity percentage above which to report files
    #[arg(short, long, value_name = "DECIMAL", default_value = "85.0")]
    pub threshold: f64,

    /// Use a faster but less precise comparison method
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub fast: u8,

    /// Display status and debugging information
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Save the results as a CSV file
    #[arg(short, long, value_name = "FILE")]
    pub csv: Option<PathBuf>,

    /// Save the results as a JSON file
    #[arg(short, long, value_name = "FILE")]
    pub json: Option<PathBuf>,

    /// Ignore this file name in the search and comparison
    #[arg(long, value_name = "NAME", conflicts_with = "require_file")]
    pub ignore_file: Vec<OsString>,

    /// Ignore this file extension in the search and comparison
    #[arg(long, value_name = "EXTENSION", conflicts_with = "require_ext")]
    pub ignore_ext: Vec<OsString>,

    /// Look for this file name in the search and comparison
    #[arg(long, value_name = "NAME", conflicts_with = "ignore_ext")]
    pub require_file: Vec<OsString>,

    /// Look for this file extension in the search and comparison
    #[arg(long, value_name = "EXTENSION")]
    pub require_ext: Vec<OsString>,

    /// Skip all lines that match this regular expression when comparing files
    #[arg(long, value_name = "REGEX")]
    pub skip_lines: Vec<Regex>,

    /// Display detailed progress information
    #[arg(short = 'P', long, action)]
    pub progress: bool,
}

/// Parse the current command-line options.
#[must_use]
pub fn options() -> Cli {
    let mut options = Cli::parse();

    // Provide the similarity threshold as a decimal value between 0.0 and 1.0,
    // rather than the human-readable percentage between 0.0 and 100.0 that the user
    // specifies on the command line.
    //
    // This saves some work later, where we would otherwise divide the threshold
    // for each file comparison.
    options.threshold /= 100.0;

    options
}
