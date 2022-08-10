use std::convert::From;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use owo_colors::{OwoColorize, Stream};
use permutator::Combination;
use rayon::prelude::*;

pub mod cli;
mod serialize;

use cli::Cli;
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
#[derive(Clone, Debug)]
struct Module {
    path: PathBuf,
    content: String,
}

#[derive(Debug)]
pub struct Comparison<'a> {
    path1: &'a Path,
    path2: &'a PathBuf,
    similarity_pct: Percentage,
}

#[derive(Debug, PartialEq)]
struct Percentage(f64);

pub fn run(options: &Cli) -> Result<()> {
    let base_path = &options.path;

    log::info!("Loading files…");
    let files = visit_dirs(base_path)?;
    let files: Vec<Module> = files
        .into_par_iter()
        .filter(|file| !file.can_skip())
        .collect();

    // Combinations by 2 pair each file with each file, so that no comparison
    // occurs more than once.
    let combinations: Vec<(&Module, &Module)> =
        files.combination(2).map(|v| (v[0], v[1])).collect();

    // The total number of combinations, and also of needed comparisons.
    let total = combinations.len();

    log::info!("Comparing files…");

    let comparisons: Vec<Comparison> = combinations
        .par_iter()
        .enumerate()
        .filter_map(|(index, (module1, module2))| {
            compare_modules(module1, module2, index, total, options)
        })
        .collect();

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

/// Compare the two modules. Print out the report and return a struct with the information.
/// Returns None if the files were skipped or if they are more different than the threshold.
fn compare_modules<'a>(
    module1: &'a Module,
    module2: &'a Module,
    index: usize,
    total: usize,
    options: &Cli,
) -> Option<Comparison<'a>> {
    log::debug!("Comparison #{}/{}", index, total);

    // Calculate the trigram similarity of the two files. This only takes
    // about 10% of the time needed for Jaro, or about 5% of Levenshtein.
    // Use teh value to pre-select files for comparison.
    let trig_sim: f64 = trigram::similarity(&module1.content, &module2.content).into();

    // Require that the trigram similarity is at least half of the set similarity threshold.
    // If it's lower than half of the threshold, skip the actual, expensive comparison.
    if trig_sim < options.threshold / 2.0 {
        log::debug!("Trigram similarity below the threshold: {:.3}", trig_sim);
        return None;
    }

    // Jaro is about 200% the speed of Levenshtein. The user can pick.
    let compare_fn = if options.fast {
        strsim::jaro
    } else {
        strsim::normalized_levenshtein
    };
    let similarity = compare_fn(&module1.content, &module2.content);

    if similarity > options.threshold {
        let percent = Percentage::from(similarity);

        if similarity >= 1.0 {
            let message = format!(
                "[{}/{}] These two files are identical ({:.1}%):",
                index, total, percent.0
            );
            println!(
                "{}",
                message.if_supports_color(Stream::Stdout, OwoColorize::red)
            );
        } else {
            let message = format!(
                "[{}/{}] These two files are similar ({:.1}%):",
                index, total, percent.0
            );
            println!(
                "{}",
                message.if_supports_color(Stream::Stdout, OwoColorize::yellow)
            );
        };
        println!(
            "\t→ {}\n\t→ {}",
            module1.path.display(),
            module2.path.display(),
        );
        log::debug!(
            "Similarity above the threshold:\n\tDistance: {:.3}, Trigram: {:.3}",
            similarity,
            trig_sim
        );

        Some(Comparison {
            path1: &module1.path,
            path2: &module2.path,
            similarity_pct: percent,
        })
    } else {
        // The files are too different.
        log::debug!(
            "Similarity below the threshold:\n\tDistance: {:.3}, Trigram: {:.3}",
            similarity,
            trig_sim
        );
        None
    }
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

/// Recursively load all files in this directory as a Vec.
fn visit_dirs(dir: &Path) -> Result<Vec<Module>> {
    let mut files = Vec::new();

    // Look for files with this extension. Ignore the rest.
    let extension: &OsStr = OsStr::new("adoc");

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        // println!("Entry: {:?}", &path);
        if path.is_symlink() {
            log::debug!("Skipping the symbolic link: {:?}", &path);
        } else if path.is_dir() {
            log::debug!("Descending into directory: {:?}", &path);
            files.append(&mut visit_dirs(&path)?);
        } else if path.is_file() && path.extension() == Some(extension) {
            log::debug!("Loading file: {:?}", &path);
            let content = fs::read_to_string(&path)?;
            let module = Module { path, content };
            files.push(module);
        }
    }
    Ok(files)
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
