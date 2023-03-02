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

use bpaf::Bpaf;
use regex::Regex;

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
pub struct Cli {
    /// Path to the root documentation directory
    #[bpaf(short, long, argument("DIR"), fallback(".".into()))]
    pub path: PathBuf,

    /// The similarity percentage above which to report files
    #[bpaf(short, long, argument("DECIMAL"), fallback(85.0))]
    pub threshold: f64,

    /// Use a faster but less precise comparison method
    #[bpaf(short, long, switch, many, map(vec_len))]
    pub fast: usize,

    /// Display status and debugging information
    #[bpaf(short, long, switch, many, map(vec_len))]
    pub verbose: usize,

    /// Save the results as a CSV file
    #[bpaf(short, long, argument("FILE"))]
    pub csv: Option<PathBuf>,

    /// Save the results as a JSON file
    #[bpaf(short, long, argument("FILE"))]
    pub json: Option<PathBuf>,

    /// Skip all lines that match this regular expression when comparing files
    #[bpaf(long, argument("REGEX"))]
    pub skip_lines: Vec<Regex>,

    /// Display detailed progress information
    #[bpaf(short('P'), long)]
    pub progress: bool,

    #[bpaf(external)]
    pub file_settings: FileSettings,
}

#[derive(Clone, Debug, Bpaf)]
pub enum FileSettings {
    Ignore {
        /// Ignore this file name in the search and comparison
        //#[arg(long, value_name = "NAME", conflicts_with = "require_file")]
        #[bpaf(long, argument("NAME"))]
        ignore_file: Vec<OsString>,
        /// Ignore this file extension in the search and comparison
        //#[arg(long, value_name = "EXTENSION", conflicts_with = "require_ext")]
        #[bpaf(long, argument("EXTENSION"))]
        ignore_ext: Vec<OsString>,
    },
    Require {
        /// Look for this file name in the search and comparison
        //#[arg(long, value_name = "NAME", conflicts_with = "ignore_ext")]
        #[bpaf(long, argument("NAME"))]
        require_file: Vec<OsString>,
        /// Look for this file extension in the search and comparison
        #[bpaf(long, argument("EXTENSION"))]
        require_ext: Vec<OsString>,
    },
    Mixed {
        /// Ignore this file name in the search and comparison
        //#[arg(long, value_name = "NAME", conflicts_with = "require_file")]
        #[bpaf(long, argument("NAME"))]
        ignore_file: Vec<OsString>,
        /// Look for this file extension in the search and comparison
        #[bpaf(long, argument("EXTENSION"))]
        require_ext: Vec<OsString>,
    },
}

/// Calculate the length of a vector for repeating flags, such as verbosity.
///
/// This function has to take the argument by value because that's how
/// the `bpaf` parser passes it in the map application.
#[allow(clippy::needless_pass_by_value)]
fn vec_len<T>(vec: Vec<T>) -> usize {
    vec.len()
}

/// Parse the current command-line options.
#[must_use]
pub fn options() -> Cli {
    let mut options = cli().run();

    // Provide the similarity threshold as a decimal value between 0.0 and 1.0,
    // rather than the human-readable percentage between 0.0 and 100.0 that the user
    // specifies on the command line.
    //
    // This saves some work later, where we would otherwise divide the threshold
    // for each file comparison.
    options.threshold /= 100.0;

    options
}
