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
    // TODO: Implement this on the file path instead of the loaded module.
    // Save the work of loading the whole file from disk.
    /// Determine whether to include this file in the comparison or skip it,
    /// based on the configured requires and ignores.
    fn wanted(&self, options: &Cli) -> bool {
        if !options.require_file.is_empty() {
            if !options.require_ext.is_empty() {
                self.required_file_name(options) || self.required_extension(options)
            } else {
                self.required_file_name(options)
            }
        } else if !options.ignore_file.is_empty() {
            if !options.require_ext.is_empty() {
                self.required_extension(options) && !self.ignored_file_name(options)
            } else if !options.ignore_ext.is_empty() {
                !self.ignored_file_name(options) && !self.ignored_extension(options)
            } else {
                !self.ignored_file_name(options)
            }
        } else if !options.require_ext.is_empty() {
            self.required_extension(options)
        } else if !options.ignore_ext.is_empty() {
            !self.ignored_extension(options)
        } else {
            true
        }
    }

    fn required_file_name(&self, options: &Cli) -> bool {
        let name = self.path.file_name().map(OsStr::to_os_string);

        if let Some(name) = name {
            options.require_file.contains(&name)
        } else {
            false
        }
    }

    fn required_extension(&self, options: &Cli) -> bool {
        let extension = self.path.extension().map(OsStr::to_os_string);

        if let Some(extension) = extension {
            options.require_ext.contains(&extension)
        } else {
            false
        }
    }

    fn ignored_file_name(&self, options: &Cli) -> bool {
        let name = self.path.file_name().map(OsStr::to_os_string);

        if let Some(name) = name {
            options.ignore_file.contains(&name)
        } else {
            false
        }
    }

    fn ignored_extension(&self, options: &Cli) -> bool {
        let extension = self.path.extension().map(OsStr::to_os_string);

        if let Some(extension) = extension {
            options.ignore_ext.contains(&extension)
        } else {
            false
        }
    }
}

impl From<f64> for Percentage {
    /// Store percentage simply as a multiple of the float by 100.
    fn from(item: f64) -> Self {
        let percent = item * 100.0;
        Self(percent)
    }
}

impl Percentage {
    /// Round the percentage value in a way that makes sure that values above 99.9%
    /// aren't mistaken for identical duplicates (100%).
    ///
    /// We display the percentage with the accuracy of one decimal place, rounded.
    /// If the percentage is above 99.9, it might get rounded up to 100,
    /// which would suggest to the user that the files are identical,
    /// even if they aren't fully 100.0% similar.
    ///
    /// To avoid the confusion, round everything between 99.9 and 100.0 down
    /// to 99.9. Thus, 100.0% is reserved for identical files.
    fn rounded(&self) -> f64 {
        let upscaled = self.0 * 10.0;

        if 999.0 < upscaled && upscaled < 1000.0 {
            99.9
        } else {
            upscaled.round() / 10.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_percentage() {
        assert_eq!(90.0, Percentage::from(0.9).rounded());
        assert_eq!(99.9, Percentage::from(0.999).rounded());
        assert_eq!(100.0, Percentage::from(1.0).rounded());

        // This is the interesting case:
        assert_eq!(99.9, Percentage::from(0.99999999).rounded());
    }
}
