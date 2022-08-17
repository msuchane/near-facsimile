use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use ignore::Walk;
use regex::Regex;

use crate::{Cli, File};

/// Load files and filter out those that are ignored by the comparisons.
pub fn files(options: &Cli) -> Result<Vec<File>> {
    let base_path = &options.path;

    let files = visit_dirs(base_path, options)?;

    // If the "skip-lines" option is not set, return files as they are.
    if options.skip_lines.is_empty() {
        Ok(files)
    // If the "skip-lines" option is set, remove all lines that match the regular
    // expressions from the file contents, before returning them.
    } else {
        Ok(files
            .into_iter()
            .map(|file| File {
                content: strip_lines(&file.content, &options.skip_lines),
                ..file
            })
            .collect())
    }
}

/// Recursively load all files in this directory as a Vec.
fn visit_dirs(dir: &Path, options: &Cli) -> Result<Vec<File>> {
    let mut files = Vec::new();
    let walk = Walk::new(dir);

    for entry in walk {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && wanted(path, options) {
            if let Some(file) = load_file(entry.into_path())? {
                files.push(file);
            }
        }
    }

    Ok(files)
}

/// Load the content of a file as `File`, if the file is valid UTF-8 text.
/// Returns `Ok(None)` if the file is accessible but not text.
fn load_file(path: PathBuf) -> Result<Option<File>> {
    log::debug!("Loading file: {:?}", &path);
    match fs::read_to_string(&path) {
        // If the file is UTF-8 text, add it to the list of files.
        Ok(content) => Ok(Some(File { path, content })),
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

/// Remove all lines that match any specified regular expression from the text.
fn strip_lines(text: &str, regexes: &[Regex]) -> String {
    let lines: Vec<&str> = text
        .lines()
        // The filter uses the "not any" condition, or `!regexes.iter().any(...)`.
        // That is, if any regex matches the line, the filter for that line
        // evaluates to `false`, and in effect removes the line from the text.
        .filter(|line| !{
            regexes.iter().any(|regex| {
                // Add an `if` block here just so that it can produce a log message.
                if regex.is_match(line) {
                    log::debug!("Skipping line due to regex {:?}:\n{:?}", &regex, &line);
                    true
                } else {
                    false
                }
            })
        })
        .collect();

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stripped_lines() {
        let regexes = &[Regex::new("^//").unwrap()];

        let text = "Here's some documentation.\n\
            \n\
            // A comment line here.\n\
            \n\
            And further documentation.";

        let stripped = "Here's some documentation.\n\
            \n\
            \n\
            And further documentation.";

        assert_eq!(stripped, strip_lines(text, regexes));
    }
}
