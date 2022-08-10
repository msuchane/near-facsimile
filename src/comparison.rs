use std::path::{Path, PathBuf};

use owo_colors::{OwoColorize, Stream};
use rayon::prelude::*;

use crate::{Cli, Module, Percentage};

#[derive(Debug)]
pub struct Comparison<'a> {
    pub path1: &'a Path,
    pub path2: &'a PathBuf,
    pub similarity_pct: Percentage,
}

pub fn comparisons<'a, T>(
    combinations: T,
    options: &Cli,
) -> Vec<Comparison<'a>>
    where T: Iterator<Item = (&'a Module, &'a Module)> + Send + ExactSizeIterator
{
    // The total number of combinations, and also of needed comparisons.
    let total = combinations.len();

    combinations
        .enumerate()
        // Convert the current sequential iterator to a parallel one.
        .par_bridge()
        .filter(|(_index, (mod1, mod2))| similar_trigrams(mod1, mod2, options))
        .filter_map(|(index, (module1, module2))| {
            compare_modules(module1, module2, index, total, options)
        })
        .collect()
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
            "Similarity above the threshold:\n\tDistance: {:.3}",
            similarity,
        );

        Some(Comparison {
            path1: &module1.path,
            path2: &module2.path,
            similarity_pct: percent,
        })
    } else {
        // The files are too different.
        log::debug!("Similarity below the threshold:{:.3}", similarity,);
        None
    }
}

/// Calculate the trigram similarity of the two files. This only takes
/// about 10% of the time needed for Jaro, or about 5% of Levenshtein.
/// Use the value to pre-select files for comparison.
fn similar_trigrams(module1: &Module, module2: &Module, options: &Cli) -> bool {
    let trig_sim: f64 = trigram::similarity(&module1.content, &module2.content).into();

    // Require that the trigram similarity is at least half of the set similarity threshold.
    // If it's lower than half of the threshold, skip the actual, expensive comparison.
    if trig_sim < options.threshold / 2.0 {
        log::debug!(
            "Trigram similarity below the threshold: {:.3}\n\t→{}\n\t→{}",
            trig_sim,
            module1.path.display(),
            module2.path.display()
        );
        false
    } else {
        true
    }
}
