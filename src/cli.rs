use std::{path::PathBuf, process::Command};
use gumdrop::Options;

#[derive(Debug, Options)]
struct Args {
    #[options(help = "show help information")]
    help: bool,

    #[options(command)]
    command: Option<Subcommand>,
}

#[derive(Debug, Options)]
enum Subcommand {
    #[options(name = "new", help = "create a new set of proc-macro crates")]
    New(NewArgs),
}

#[derive(Debug, Options)]
struct NewArgs {
    #[options(
        no_short,
        help = "Set the resulting base crate name, defaults to the directory name"
    )]
    name: Option<String>,

    #[options(free, help = "Create a new proc-macro crate set at <path>")]
    path: Option<PathBuf>,

    #[options(help = "Prints help information")]
    help: bool,
}

impl NewArgs {
    fn print_usage() {
        println!("cargo-proc-macro -- Manage proc-macro crates with Cargo\n<https://github.com/bbqsrc/cargo-proc-macro>\n\nUsage: cargo proc-macro new [OPTIONS] <path>\n");
        println!("{}\n", NewArgs::usage());
    }
}

impl Args {
    fn print_usage() {
        println!("cargo-proc-macro -- Manage proc-macro crates with Cargo\n<https://github.com/bbqsrc/cargo-proc-macro>\n\nUsage: cargo proc-macro [OPTIONS] [SUBCOMMAND]\n");
        println!("{}\n", Args::usage());
        println!("Available commands:\n{}", Args::command_list().unwrap());
    }
}

const BASE_TMPL: &str = "extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn @NAME@(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as proc_macro2::TokenStream);
    let item = parse_macro_input!(item as proc_macro2::TokenStream);
    
    match @NAME@_macro::@NAME@(attr, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}
";

const CRATE_TMPL: &str = "use proc_macro2::TokenStream;
use quote::quote;

pub fn @NAME@(attr: TokenStream, item: TokenStream) -> Result<TokenStream, syn::Error> {
    // Implement your proc-macro logic here. :)
    Ok(item)
}
";

pub(crate) fn run(args: Vec<String>) {
    log::trace!("Args: {:?}", args);

    let args = match Args::parse_args(&args, gumdrop::ParsingStyle::AllOptions) {
        Ok(args) if args.help => {
            Args::print_usage();
            std::process::exit(0);
        }
        Ok(args) => args,
        Err(e) => {
            log::error!("{}", e);
            std::process::exit(2);
        }
    };

    println!("{:?}", &args);

    match args.command {
        Some(Subcommand::New(a)) => {
            if a.help {
                NewArgs::print_usage();
                std::process::exit(0)
            }

            let path = a.path.unwrap_or_else(|| std::env::current_dir().unwrap());

            let name = a
                .name
                .unwrap_or_else(|| path.file_name().unwrap().to_string_lossy().to_string());

            // Make "real" proc-macro crate
            let _base_crate_output = Command::new("cargo")
                .arg("new")
                .arg("--lib")
                .arg(&path.join(&name))
                .output()
                .unwrap();
            let base_crate_lib_rs = path.join(&name).join("src").join("lib.rs");
            std::fs::write(
                base_crate_lib_rs,
                BASE_TMPL.replace("@NAME@", &name.replace("-", "_")),
            )
            .unwrap();
            let base_crate_cargo_toml = path.join(&name).join("Cargo.toml");

            let toml = std::fs::read_to_string(&base_crate_cargo_toml).unwrap();
            let toml = toml.replace(
                "[dependencies]",
                &format!(
                    "[lib]
proc-macro = true

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
{name}_macro = {{ path = \"../{name}_macro\" }}
syn = \"1\"
proc-macro2 = \"1\"",
                    name = &name
                ),
            );
            std::fs::write(&base_crate_cargo_toml, toml).unwrap();

            // Make macro crate with actual logic
            let _macro_crate_cmd = Command::new("cargo")
                .arg("new")
                .arg("--lib")
                .arg(&path.join(&format!("{}_macro", name)))
                .output()
                .unwrap();
            let base_crate_lib_rs = path
                .join(&format!("{}_macro", name))
                .join("src")
                .join("lib.rs");
            std::fs::write(
                base_crate_lib_rs,
                CRATE_TMPL.replace("@NAME@", &name.replace("-", "_")),
            )
            .unwrap();
            let macro_crate_cargo_toml = path.join(&format!("{}_macro", name)).join("Cargo.toml");
            let mut toml = std::fs::read_to_string(&macro_crate_cargo_toml).unwrap();
            toml.push_str(
                "syn = { version = \"1\", features = [\"full\", \"extra-traits\"] }
quote = \"1\"
proc-macro2 = \"1\"
",
            );
            std::fs::write(&macro_crate_cargo_toml, toml).unwrap();

            // Add workspace toml
            std::fs::write(
                path.join("Cargo.toml"),
                format!(
                    "[workspace]\nmembers = [\n  \"{name}\",\n  \"{name}_macro\",\n]",
                    name = &name
                ),
            )
            .unwrap();
        }
        None => {
            Args::print_usage();
            std::process::exit(2);
        }
    }
}
