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

/// Groups together the various data and options used in an iteration
/// of comparing two files, for convenience.
struct ComparedPair<'a> {
    index: usize,
    total: usize,
    module1: &'a Module,
    module2: &'a Module,
    trigram: f64,
}

impl ComparedPair<'_> {
    /// Calculating the trigram only takes
    /// about 10% of the time needed for Jaro, or about 5% of Levenshtein.
    /// Use the value to pre-select files for comparison.
    fn trigram_preselect(&self, options: &Cli) -> bool {
        // Require that the trigram similarity is at least half of the set similarity threshold.
        // If it's lower than half of the threshold, skip the actual, expensive comparison.
        if self.trigram < options.threshold / 2.0 {
            log::debug!(
                "Trigram similarity below the threshold: {:.3}\n\t→{}\n\t→{}",
                self.trigram,
                self.module1.path.display(),
                self.module2.path.display()
            );
            false
        } else {
            true
        }
    }
}

pub fn comparisons<'a, T>(combinations: T, options: &Cli) -> Vec<Comparison<'a>>
where
    T: Iterator<Item = (&'a Module, &'a Module)> + Send + ExactSizeIterator,
{
    // The total number of combinations, and also of needed comparisons.
    let total = combinations.len();

    combinations
        .enumerate()
        // Convert the current sequential iterator to a parallel one.
        .par_bridge()
        .map(|(index, (module1, module2))| ComparedPair {
            index,
            total,
            module1,
            module2,
            trigram: trigram_f64(&module1.content, &module2.content),
        })
        .filter(|pair| pair.trigram_preselect(options))
        .filter_map(|pair| compare_modules(&pair, options))
        .collect()
}

/// Compare the two modules. Print out the report and return a struct with the information.
/// Returns None if the files were skipped or if they are more different than the threshold.
fn compare_modules<'a>(pair: &ComparedPair<'a>, options: &Cli) -> Option<Comparison<'a>> {
    log::debug!("Comparison #{}/{}", pair.index, pair.total);

    // The user can pick the accuracy and speed of the comaprison.
    let similarity = match options.fast {
        // Levenshtein is slow and accurate. Default.
        0 => strsim::normalized_levenshtein(&pair.module1.content, &pair.module2.content),
        // Jaro is about 200% the speed of Levenshtein.
        1 => strsim::jaro(&pair.module1.content, &pair.module2.content),
        // Trigram si rudimentary, but very fast.
        // Reuse the value calculated in the iterator pipeline earlier.
        _ => pair.trigram,
    };

    if similarity > options.threshold {
        let percent = Percentage::from(similarity);

        if similarity >= 1.0 {
            let message = format!(
                "[{}/{}] These two files are identical ({:.1}%):",
                pair.index,
                pair.total,
                percent.rounded()
            );
            println!(
                "{}",
                message.if_supports_color(Stream::Stdout, OwoColorize::red)
            );
        } else {
            let message = format!(
                "[{}/{}] These two files are similar ({:.1}%):",
                pair.index,
                pair.total,
                percent.rounded()
            );
            println!(
                "{}",
                message.if_supports_color(Stream::Stdout, OwoColorize::yellow)
            );
        };
        println!(
            "\t→ {}\n\t→ {}",
            pair.module1.path.display(),
            pair.module2.path.display(),
        );
        log::debug!(
            "Similarity above the threshold:\n\tDistance: {:.3}",
            similarity,
        );

        Some(Comparison {
            path1: &pair.module1.path,
            path2: &pair.module2.path,
            similarity_pct: percent,
        })
    } else {
        // The files are too different.
        log::debug!("Similarity below the threshold:{:.3}", similarity,);
        None
    }
}

/// Calculate the trigram metric and convert to f64,
/// so that we can easily compare it with the other metrics.
fn trigram_f64(content1: &str, content2: &str) -> f64 {
    f64::from(trigram::similarity(content1, content2))
}
