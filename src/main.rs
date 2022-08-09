use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use color_eyre::{eyre::eyre, Result};
use rayon::prelude::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Hello, world!");

    let base_path = Path::new(".");
    let mut files = Vec::new();

    visit_dirs(base_path, &mut files)?;

    // println!("Files:\n{:#?}", files);

    for (index1, module1) in files.iter().enumerate() {
        let starting_index = index1 + 1;

        files[starting_index..].par_iter().for_each(|module2| {
            if module1.path == module2.path {
                println!("Same files actually.");
            } else {
                let similarity = strsim::normalized_levenshtein(&module1.content, &module2.content);
                if similarity > 0.8 {
                    let percent = (similarity * 100.0).round();
                    println!("These two files are very similar ({}%):", percent);
                    println!("* {}\n* {}", module1.path.display(), module2.path.display());
                }
            }
        });
    }

    Ok(())
}

struct Module {
    path: PathBuf,
    content: String,
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, files: &mut Vec<Module>) -> Result<()> {
    let extension: &OsStr = OsStr::new("adoc");

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            // println!("Entry: {:?}", &path);
            if path.is_symlink() {
                // println!("Symlink: {:?}", &path);
                continue;
            } else if path.is_dir() {
                // println!("Directory: {:?}", &path);
                visit_dirs(&path, files)?;
            } else if path.is_file() && path.extension() == Some(extension) {
                // println!("Inserting file: {:?}", &path);
                let content = fs::read_to_string(&path)?;
                let module = Module {
                    path,
                    content
                };
                files.push(module);
            }
        }
    }
    Ok(())
}
