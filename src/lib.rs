use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use owo_colors::{OwoColorize, Stream};
use rayon::prelude::*;

pub mod cli;

const IGNORED_FILE_NAMES: [&str; 6] = [
    "master.adoc",
    "main.adoc",
    "_attributes.adoc",
    "_local-attributes.adoc",
    "_title-attributes.adoc",
    "README.adoc",
];
const OUT_FILE_NAME: &str = "comparisons.csv";

/// Represents a loaded AsciiDoc file, with its path and content.
struct Module {
    path: PathBuf,
    content: String,
}

#[derive(Debug)]
struct Comparison<'a> {
    path1: &'a Path,
    path2: &'a PathBuf,
    similarity_pct: f64,
}

pub fn run(options: &cli::Cli) -> Result<()> {
    let base_path = &options.path;
    let threshold = options.threshold;

    log::info!("Loading files…");
    let files = visit_dirs(base_path)?;

    log::info!("Comparing files…");

    let mut comparisons: Vec<Comparison> =
        files
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut acc, (index1, module1)| {
                let starting_index = index1 + 1;

                let mut comparisons: Vec<Comparison> = files[starting_index..]
                    .par_iter()
                    .filter_map(|module2| compare_modules(module1, module2, threshold))
                    .collect();

                acc.append(&mut comparisons);

                acc
            });

    log::info!("Producing a CSV table…");
    serialize(&mut comparisons)?;

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
    threshold: f64,
) -> Option<Comparison<'a>> {
    if module1.path == module2.path {
        log::warn!("Comparing the same files.");
        None
    } else if can_skip(module1) || can_skip(module2) {
        log::debug!("Skipping files {:?} and {:?}", &module1.path, &module2.path);
        None
    } else {
        let similarity = strsim::normalized_levenshtein(&module1.content, &module2.content);
        if similarity > threshold {
            let percent = similarity * 100.0;

            if similarity >= 1.0 {
                let message = format!("These two files are identical ({:.1}%):", percent);
                println!(
                    "{}",
                    message.if_supports_color(Stream::Stdout, OwoColorize::red)
                );
            } else {
                let message = format!("These two files are similar ({:.1}%):", percent);
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

/// Serialize the resulting comparisons as a CSV table.
fn serialize(comparisons: &mut [Comparison]) -> Result<()> {
    // Prepare to write to the CSV file.
    let mut wtr = csv::Writer::from_path(OUT_FILE_NAME)?;

    // The CSV header:
    wtr.write_record(&["% similar", "File 1", "File 2"])?;

    // Sort from highest to lowest. You can't sort f64 values, so convert them to u32
    // with a precision of percentage with a single decimal place, then subtract from 1000.
    comparisons
        .par_sort_by_key(|comparison| 1000 - (comparison.similarity_pct * 10.0).round() as i32);

    // Each comparison entry writes a row in the CSV table.
    for comparison in comparisons {
        wtr.write_record(&[
            format!("{:.1}", comparison.similarity_pct),
            comparison.path1.display().to_string(),
            comparison.path2.display().to_string(),
        ])?;
    }

    // Flush the CSV writer buffer.
    wtr.flush()?;

    Ok(())
}
