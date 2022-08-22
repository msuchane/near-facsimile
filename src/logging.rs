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

use color_eyre::Result;
use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

/// Initialize the handlers for logging and error reporting.
pub fn init_log_and_errors(verbose: u8) -> Result<()> {
    color_eyre::install()?;

    // Use the local time zone in log messages.
    let config = ConfigBuilder::new()
        // TODO: There's probably a bug in the type signature of set_time_offset_to_local,
        // which prevents us from using `?` on it. Report to upstream.
        .set_time_offset_to_local()
        .expect("Failed to determine the local time zone.")
        .set_time_level(LevelFilter::Debug)
        .build();

    let log_level = match verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };

    TermLogger::init(
        log_level,
        config,
        // Mixed mode prints errors to stderr and info to stdout. Not sure about the other levels.
        TerminalMode::default(),
        // Try to use color if possible.
        ColorChoice::Auto,
    )?;

    Ok(())
}
