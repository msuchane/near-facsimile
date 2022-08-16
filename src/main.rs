use color_eyre::eyre::Result;

use near_facsimile::{cli, init_log_and_errors, run};

fn main() -> Result<()> {
    let options = cli::options();
    init_log_and_errors(options.verbose)?;

    run(&options)?;

    Ok(())
}
