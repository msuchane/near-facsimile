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

use std::convert::From;

#[derive(Debug, PartialEq)]
pub struct Percentage(pub f64);

impl From<f64> for Percentage {
    /// Store percentage simply as a multiple of the float by 100.
    fn from(item: f64) -> Self {
        let percent = item * 100.0;
        Self(percent)
    }
}

impl Percentage {
    /// Round the percentage value in a way that makes sure that values above 99.9%
    /// aren't mistaken for identical duplicates (100%).
    ///
    /// We display the percentage with the accuracy of one decimal place, rounded.
    /// If the percentage is above 99.9, it might get rounded up to 100,
    /// which would suggest to the user that the files are identical,
    /// even if they aren't fully 100.0% similar.
    ///
    /// To avoid the confusion, round everything between 99.9 and 100.0 down
    /// to 99.9. Thus, 100.0% is reserved for identical files.
    pub fn rounded(&self) -> f64 {
        let upscaled = self.0 * 10.0;

        if 999.0 < upscaled && upscaled < 1000.0 {
            99.9
        } else {
            upscaled.round() / 10.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_percentage() {
        assert_eq!(90.0, Percentage::from(0.9).rounded());
        assert_eq!(99.9, Percentage::from(0.999).rounded());
        assert_eq!(100.0, Percentage::from(1.0).rounded());

        // This is the interesting case:
        assert_eq!(99.9, Percentage::from(0.999_999_99).rounded());
    }
}
