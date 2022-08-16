use std::fs;
use std::io;
use std::path::Path;

use color_eyre::Result;
use rayon::prelude::*;

use crate::{Cli, Module};

/// Load files and filter out those that are ignored by the comparisons.
pub fn files(options: &Cli) -> Result<Vec<Module>> {
    let base_path = &options.path;

    let files = visit_dirs(base_path, options)?;
    Ok(files
        .into_par_iter()
        .filter(|file| file.wanted(options))
        .collect())
}

/// Recursively load all files in this directory as a Vec.
fn visit_dirs(dir: &Path, options: &Cli) -> Result<Vec<Module>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        // println!("Entry: {:?}", &path);
        if path.is_symlink() {
            log::debug!("Skipping the symbolic link: {:?}", &path);
        } else if path.is_dir() {
            log::debug!("Descending into directory: {:?}", &path);
            files.append(&mut visit_dirs(&path, options)?);
        } else if path.is_file() {
            log::debug!("Loading file: {:?}", &path);
            match fs::read_to_string(&path) {
                // If the file is UTF-8 text, add it to the list of files.
                Ok(content) => {
                    let module = Module { path, content };
                    files.push(module);
                }
                // If we can't read the file:
                Err(e) => {
                    // If we can't read it because it's not UTF-8, just skip the file.
                    if e.kind() == io::ErrorKind::InvalidData {
                        log::debug!(
                            "Skipping file that is not valid UTF-8 text: {}",
                            path.display()
                        );
                    // If we can't read it for any other reason, exit with the contained error.
                    } else {
                        Err(e)?;
                    }
                }
            };
        }
    }
    Ok(files)
}
