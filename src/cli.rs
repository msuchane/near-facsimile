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

    /// Display status and debugging information
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Save the results as a CSV file
    #[clap(short, long, value_name = "FILE")]
    pub csv: Option<PathBuf>,

    /// Save the results as a JSON file
    #[clap(short, long, value_name = "FILE")]
    pub json: Option<PathBuf>,

    /// Ignore this file name in the search and comparison
    #[clap(long, value_name = "NAME", conflicts_with = "require-file")]
    pub ignore_file: Vec<OsString>,

    /// Ignore this file extension in the search and comparison
    #[clap(long, value_name = "EXTENSION", conflicts_with = "require-ext")]
    pub ignore_ext: Vec<OsString>,

    /// Look for this file name in the search and comparison
    #[clap(long, value_name = "NAME", conflicts_with = "ignore-ext")]
    pub require_file: Vec<OsString>,

    /// Look for this file extension in the search and comparison
    #[clap(long, value_name = "EXTENSION")]
    pub require_ext: Vec<OsString>,

    /// Skip all lines that match this regular expression when comparing files
    #[clap(long, value_name = "REGEX")]
    pub skip_lines: Vec<Regex>,

    /// Display detailed progress information
    #[clap(short = 'P', long, action)]
    pub progress: bool,
}

/// Parse the current command-line options.
#[must_use]
pub fn options() -> Cli {
    Cli::parse()
}
