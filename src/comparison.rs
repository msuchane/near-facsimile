/*
Copyright 2022 Marek Suchánek <msuchane@redhat.com>

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::path::{Path, PathBuf};

use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use owo_colors::{OwoColorize, Stream};
use rayon::prelude::*;

use crate::{Cli, File, Percentage};

#[derive(Debug)]
pub struct Comparison<'a> {
    pub path1: &'a Path,
    pub path2: &'a PathBuf,
    pub similarity_pct: Percentage,
}

/// Groups together the various data and options used in an iteration
/// of comparing two files, for convenience.
struct ComparedPair<'a> {
    file1: &'a File,
    file2: &'a File,
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
                self.file1.path.display(),
                self.file2.path.display()
            );
            false
        } else {
            true
        }
    }
}

pub fn comparisons<'a, T>(combinations: T, options: &Cli) -> Vec<Comparison<'a>>
where
    T: Iterator<Item = (&'a File, &'a File)> + Send + ExactSizeIterator,
{
    log::info!("Comparing files…");

    // If the `progress` command-line option isn't active, hide the progress bar.
    let progress_bar = if options.progress {
        // The total number of combinations, and also of needed comparisons.
        let total = combinations.len();

        // Configure the progress bar.
        let progress_style = ProgressStyle::with_template(
            "Progress {percent:>3}%    Comparison# {human_pos:>8}/{human_len:8}    [{elapsed_precise}]",
        )
        .expect("Failed to format the progress bar.");

        ProgressBar::new(total as u64).with_style(progress_style)
    // When hidden, the progress bar doesn't render, and only satisfies the API.
    } else {
        ProgressBar::hidden()
    };

    combinations
        // Convert the current sequential iterator to a parallel one.
        .par_bridge()
        .progress_with(progress_bar)
        .map(|(file1, file2)| ComparedPair {
            file1,
            file2,
            trigram: trigram_f64(&file1.content, &file2.content),
        })
        .filter(|pair| pair.trigram_preselect(options))
        .filter_map(|pair| compare_files(&pair, options))
        .collect()
}

/// Compare the two files. Print out the report and return a struct with the information.
/// Returns None if the files were skipped or if they are more different than the threshold.
fn compare_files<'a>(pair: &ComparedPair<'a>, options: &Cli) -> Option<Comparison<'a>> {
    // The user can pick the accuracy and speed of the comparison.
    let similarity = match options.fast {
        // Levenshtein is slow and accurate. Default.
        0 => strsim::normalized_levenshtein(&pair.file1.content, &pair.file2.content),
        // Jaro is about 200% the speed of Levenshtein.
        1 => strsim::jaro(&pair.file1.content, &pair.file2.content),
        // Trigram si rudimentary, but very fast.
        // Reuse the value calculated in the iterator pipeline earlier.
        _ => pair.trigram,
    };

    if similarity > options.threshold {
        let percent = Percentage::from(similarity);
        // Prepare the listing of the file pair before printing.
        let file_display = format!(
            "  ‣ {}\n  ‣ {}",
            pair.file1.path.display(),
            pair.file2.path.display(),
        );

        if similarity >= 1.0 {
            let message = format!("These two files are identical ({:.1}%):", percent.rounded());
            log::info!(
                "{}\n{}",
                message.if_supports_color(Stream::Stdout, OwoColorize::red),
                file_display,
            );
        } else {
            let message = format!("These two files are similar ({:.1}%):", percent.rounded());
            log::info!(
                "{}\n{}",
                message.if_supports_color(Stream::Stdout, OwoColorize::yellow),
                file_display
            );
        };
        log::debug!(
            "Similarity above the threshold:\n\tDistance: {:.3}",
            similarity,
        );

        Some(Comparison {
            path1: &pair.file1.path,
            path2: &pair.file2.path,
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
