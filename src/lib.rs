use std::convert::From;
use std::ffi::OsStr;
use std::path::PathBuf;

use color_eyre::{eyre::bail, Result};
use permutator::Combination;

pub mod cli;
mod comparison;
mod load_files;
mod serialize;

use cli::Cli;
use comparison::{comparisons, Comparison};
use load_files::files;
use serialize::serialize;

const IGNORED_FILE_NAMES: [&str; 6] = [
    "master.adoc",
    "main.adoc",
    "_attributes.adoc",
    "_local-attributes.adoc",
    "_title-attributes.adoc",
    "README.adoc",
];

/// Represents a loaded AsciiDoc file, with its path and content.
#[derive(Debug)]
pub struct Module {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, PartialEq)]
pub struct Percentage(f64);

pub fn run(options: &Cli) -> Result<()> {
    log::info!("Loading files…");
    let files = files(options)?;

    if files.len() < 2 {
        bail!("Too few files that match the settings to compare in this directory.");
    }

    // Combinations by 2 pair each file with each file, so that no comparison
    // occurs more than once.
    let combinations = files.combination(2).map(|v| (v[0], v[1]));

    log::info!("Comparing files…");
    let comparisons = comparisons(combinations, options);

    log::info!("Producing a CSV table…");
    serialize(comparisons, options)?;

    Ok(())
}

/// Initialize the handlers for logging and error reporting.
pub fn init_log_and_errors(verbose: u8) -> Result<()> {
    color_eyre::install()?;

    let log_level = match verbose {
        0 => simplelog::LevelFilter::Warn,
        1 => simplelog::LevelFilter::Info,
        _ => simplelog::LevelFilter::Debug,
    };

    simplelog::TermLogger::init(
        log_level,
        simplelog::Config::default(),
        // Mixed mode prints errors to stderr and info to stdout. Not sure about the other levels.
        simplelog::TerminalMode::default(),
        // Try to use color if possible.
        simplelog::ColorChoice::Auto,
    )?;

    Ok(())
}

impl Module {
    /// Determine if we can skip comparing this module, because it's common content.
    fn can_skip(&self) -> bool {
        let string = self.path.file_name().and_then(OsStr::to_str);

        let skip = match string {
            Some(s) => IGNORED_FILE_NAMES.contains(&s),
            None => false,
        };

        if skip {
            log::debug!("Skipping file {:?}", &self.path);
        }

        skip
    }
}

impl From<f64> for Percentage {
    /// We display the percentage with the accuracy of one decimal place, rounded.
    /// If the percentage is above 99.9, it might get rounded up to 100,
    /// which would suggest to the user that the files are identical,
    /// even if they aren't fully 100.0% similar.
    ///
    /// To avoid the confusion, round everything between 99.9 and 100.0 down
    /// to 99.9. Thus, 100.0% is reserved for identical files.
    fn from(item: f64) -> Self {
        let percent = item * 100.0;

        if 99.9 < percent && percent < 100.0 {
            Self(99.9)
        } else {
            Self(percent)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_percentage() {
        assert_eq!(90.0, Percentage::from(0.9).0);
        assert_eq!(99.9, Percentage::from(0.999).0);
        assert_eq!(100.0, Percentage::from(1.0).0);

        // This is the interesting case:
        assert_eq!(99.9, Percentage::from(0.99999999).0);
    }
}
