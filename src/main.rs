use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use color_eyre::{eyre::eyre, Result};
use owo_colors::OwoColorize;
use rayon::prelude::*;

const IGNORED_FILE_NAMES: [&str; 5] = [
    "master.adoc",
    "_local-attributes.adoc",
    "_title-attributes.adoc",
    "README.adoc",
    "_attributes.adoc",
];

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Hello, world!");

    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        // Mixed mode prints errors to stderr and info to stdout. Not sure about the other levels.
        simplelog::TerminalMode::default(),
        // Try to use color if possible.
        simplelog::ColorChoice::Auto,
    )?;

    let base_path = Path::new(".");

    log::info!("Loading files…");
    let files = visit_dirs(base_path)?;

    // println!("Files:\n{:#?}", files);

    log::info!("Comparing files…");

    for (index1, module1) in files.iter().enumerate() {
        let starting_index = index1 + 1;

        files[starting_index..].par_iter().for_each(|module2| {
            if module1.path == module2.path {
                println!("Same files actually.");
            } else if can_skip(module1) || can_skip(module2) {
            } else {
                let similarity = strsim::normalized_levenshtein(&module1.content, &module2.content);
                if similarity > 0.8 {
                    let percent = (similarity * 100.0).round();

                    if similarity >= 1.0 {
                        let message = format!("These two files are identical ({}%):", percent);
                        println!("{}", message.red());
                    } else {
                        let message = format!("These two files are very similar ({}%):", percent);
                        println!("{}", message.yellow());
                    };
                    println!(
                        "\t→ {}\n\t→ {}",
                        module1.path.display(),
                        module2.path.display()
                    );
                }
            }
        });
    }

    Ok(())
}

fn can_skip(module: &Module) -> bool {
    let string = module.path.file_name().and_then(OsStr::to_str);

    match string {
        Some(s) => IGNORED_FILE_NAMES.contains(&s),
        None => false,
    }
}

/// Represents a loaded AsciiDoc file, with its path and content.
struct Module {
    path: PathBuf,
    content: String,
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
            // println!("Symlink: {:?}", &path);
            continue;
        } else if path.is_dir() {
            // println!("Directory: {:?}", &path);
            files.append(&mut visit_dirs(&path)?);
        } else if path.is_file() && path.extension() == Some(extension) {
            // println!("Inserting file: {:?}", &path);
            let content = fs::read_to_string(&path)?;
            let module = Module { path, content };
            files.push(module);
        }
    }
    Ok(files)
}
