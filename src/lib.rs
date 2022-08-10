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
    similarity_pct: f64,
}

pub fn run(options: &Cli) -> Result<()> {
    let base_path = &options.path;

    log::info!("Loading files…");
    let files = visit_dirs(base_path)?;

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
    log::debug!("File #{}/{}", index, total);
    if module1.path == module2.path {
        log::warn!("Comparing the same files.");
        None
    } else if can_skip(module1) || can_skip(module2) {
        log::debug!("Skipping files {:?} and {:?}", &module1.path, &module2.path);
        None
    } else {
        let similarity = if options.fast {
            strsim::jaro(&module1.content, &module2.content)
        } else {
            strsim::normalized_levenshtein(&module1.content, &module2.content)
        };
        if similarity > options.threshold {
            let percent = similarity * 100.0;

            if similarity >= 1.0 {
                let message = format!(
                    "[{}/{}] These two files are identical ({:.1}%):",
                    index, total, percent
                );
                println!(
                    "{}",
                    message.if_supports_color(Stream::Stdout, OwoColorize::red)
                );
            } else {
                let message = format!(
                    "[{}/{}] These two files are similar ({:.1}%):",
                    index, total, percent
                );
                println!(
                    "{}",
                    message.if_supports_color(Stream::Stdout, OwoColorize::yellow)
                );
            };
            println!(
                "\t→ {}\n\t→ {}",
                module1.path.display(),
                module2.path.display()
            );

            Some(Comparison {
                path1: &module1.path,
                path2: &module2.path,
                similarity_pct: percent,
            })
        } else {
            // The files are too different.
            None
        }
    }
}

/// Determine if we can skip comparing this module, because it's common content.
fn can_skip(module: &Module) -> bool {
    let string = module.path.file_name().and_then(OsStr::to_str);

    match string {
        Some(s) => IGNORED_FILE_NAMES.contains(&s),
        None => false,
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
