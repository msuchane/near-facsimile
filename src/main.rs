use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::ffi::OsStr;

use color_eyre::{eyre::eyre, Result};

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Hello, world!");

    let base_path = Path::new(".");
    let mut files = HashMap::new();

    visit_dirs(base_path, &mut files)?;

    println!("Files:\n{:#?}", files);

    Ok(())
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, files: &mut HashMap<PathBuf, String>) -> Result<()> {
    let extension: &OsStr = OsStr::new("adoc");

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            println!("Entry: {:?}", &path);
            if path.is_symlink() {
                println!("Symlink: {:?}", &path);
                continue;
            } else if path.is_dir() {
                println!("Directory: {:?}", &path);
                visit_dirs(&path, files)?;
            } else if path.is_file() && path.extension() == Some(extension) {
                println!("Inserting file: {:?}", &path);
                let content = fs::read_to_string(&path)?;
                files.insert(path, content);
            }
        }
    }
    Ok(())
}
