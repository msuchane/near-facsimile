use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use ignore::Walk;

use crate::{Cli, Module};

/// Load files and filter out those that are ignored by the comparisons.
pub fn files(options: &Cli) -> Result<Vec<Module>> {
    let base_path = &options.path;

    visit_dirs(base_path, options)
}

/// Recursively load all files in this directory as a Vec.
fn visit_dirs(dir: &Path, options: &Cli) -> Result<Vec<Module>> {
    let mut files = Vec::new();
    let walk = Walk::new(dir);

    for entry in walk {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && wanted(path, options) {
            if let Some(module) = load_file(entry.into_path())? {
                files.push(module);
            }
        }
    }

    Ok(files)
}

/// Load the content of a file as `Module`, if the file is valid UTF-8 text.
/// Returns `Ok(None)` if the file is accessible but not text.
fn load_file(path: PathBuf) -> Result<Option<Module>> {
    log::debug!("Loading file: {:?}", &path);
    match fs::read_to_string(&path) {
        // If the file is UTF-8 text, add it to the list of files.
        Ok(content) => Ok(Some(Module { path, content })),
        // If we can't read the file:
        Err(e) => {
            // If we can't read it because it's not UTF-8, just skip the file.
            if e.kind() == io::ErrorKind::InvalidData {
                log::debug!(
                    "Skipping file that is not valid UTF-8 text: {}",
                    path.display()
                );
                Ok(None)
            // If we can't read it for any other reason, exit with the contained error.
            } else {
                Err(e)?
            }
        }
    }
}

/// Determine whether to include this file in the comparison or skip it,
/// based on the configured requires and ignores.
fn wanted(path: &Path, options: &Cli) -> bool {
    if !options.require_file.is_empty() {
        if !options.require_ext.is_empty() {
            required_file_name(path, options) || required_extension(path, options)
        } else {
            required_file_name(path, options)
        }
    } else if !options.ignore_file.is_empty() {
        if !options.require_ext.is_empty() {
            required_extension(path, options) && !ignored_file_name(path, options)
        } else if !options.ignore_ext.is_empty() {
            !ignored_file_name(path, options) && !ignored_extension(path, options)
        } else {
            !ignored_file_name(path, options)
        }
    } else if !options.require_ext.is_empty() {
        required_extension(path, options)
    } else if !options.ignore_ext.is_empty() {
        !ignored_extension(path, options)
    } else {
        true
    }
}

fn required_file_name(path: &Path, options: &Cli) -> bool {
    let name = path.file_name().map(OsStr::to_os_string);

    if let Some(name) = name {
        options.require_file.contains(&name)
    } else {
        false
    }
}

fn required_extension(path: &Path, options: &Cli) -> bool {
    let extension = path.extension().map(OsStr::to_os_string);

    if let Some(extension) = extension {
        options.require_ext.contains(&extension)
    } else {
        false
    }
}

fn ignored_file_name(path: &Path, options: &Cli) -> bool {
    let name = path.file_name().map(OsStr::to_os_string);

    if let Some(name) = name {
        options.ignore_file.contains(&name)
    } else {
        false
    }
}

fn ignored_extension(path: &Path, options: &Cli) -> bool {
    let extension = path.extension().map(OsStr::to_os_string);

    if let Some(extension) = extension {
        options.ignore_ext.contains(&extension)
    } else {
        false
    }
}
