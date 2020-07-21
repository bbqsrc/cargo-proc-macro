use std::env;
use std::process::exit;

mod cli;

fn main() {
    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    if env::var("CARGO").is_err() {
        eprintln!("This binary may only be called via `cargo proc-macro`.");
        exit(1);
    }

    let args = std::env::args().skip(2).collect::<Vec<_>>();

    cli::run(args);
}
