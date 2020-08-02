// SPDX-License-Identifier: EUPL-1.2+

use std::env;
use std::process::exit;

mod cli;

fn main() -> anyhow::Result<()> {
    if env::var("CARGO").is_err() {
        eprintln!("This binary may only be called via `cargo proc-macro`.");
        exit(1);
    }

    let args = std::env::args().skip(2).collect::<Vec<_>>();

    Ok(cli::run(args)?)
}
