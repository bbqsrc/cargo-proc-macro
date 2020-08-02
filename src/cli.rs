// SPDX-License-Identifier: EUPL-1.2+

use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
    process::{self, exit, Command},
};

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
    #[options(
        name = "new",
        help = "Create a new set of proc-macro crates at given path"
    )]
    New(NewArgs),
    #[options(
        name = "init",
        help = "Create a new set of proc-macro crates in an existing directory"
    )]
    Init(InitArgs),
}

#[derive(Debug, Options)]
struct NewArgs {
    #[options(
        no_short,
        help = "Set the resulting base crate name, defaults to the directory name"
    )]
    name: Option<String>,

    #[options(free, help = "Create a new proc-macro crate set at <path>")]
    path: PathBuf,

    #[options(help = "Prints help information")]
    help: bool,
}

#[derive(Debug, Options)]
struct InitArgs {
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

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Could not resolve a package name for the given directory.")]
    NameResolutionFailed,

    #[error("Running `cargo new --lib {1}` failed.")]
    CargoNewLibFailed(#[source] std::io::Error, PathBuf),

    #[error("Reading from {1} failed.")]
    ReadFailed(#[source] std::io::Error, PathBuf),

    #[error("Writing to {1} failed.")]
    WriteFailed(#[source] std::io::Error, PathBuf),
}

fn cargo_new_lib(path: &Path) -> Result<process::Output, Error> {
    Command::new("cargo")
        .arg("new")
        .arg("--lib")
        .arg(&path)
        .output()
        .map_err(|e| Error::CargoNewLibFailed(e, path.to_path_buf()))
}

fn write_proc_macro_cargo_toml(path: PathBuf, name: &str) -> Result<(), Error> {
    let toml = fs::read_to_string(&path).map_err(|e| Error::ReadFailed(e, path.clone()))?;
    let toml = toml.replace(
        "[dependencies]",
        &format!(
            "[lib]
proc-macro = true

[dependencies]
{name}_macro = {{ path = \"../{name}_macro\" }}
syn = \"1\"
proc-macro2 = \"1\"",
            name = name
        ),
    );
    fs::write(&path, toml).map_err(|e| Error::WriteFailed(e, path))?;
    Ok(())
}

// Make "real" proc-macro crate
fn create_base_crate(path: &Path, name: &str) -> Result<(), Error> {
    let base_path = path.join(&name);
    cargo_new_lib(&base_path)?;

    let lib_rs_output = BASE_TMPL.replace("@NAME@", &name.replace("-", "_"));
    let lib_rs_path = base_path.join("src").join("lib.rs");
    fs::write(&lib_rs_path, lib_rs_output)
        .map_err(|e| Error::WriteFailed(e, lib_rs_path.clone()))?;

    write_proc_macro_cargo_toml(base_path.join("Cargo.toml"), &*name)?;
    Ok(())
}

// Make macro crate with actual logic
fn create_macro_crate(path: &Path, name: &str) -> Result<(), Error> {
    let macro_path = path.join(&format!("{}_macro", name));
    cargo_new_lib(&macro_path)?;

    let lib_rs_output = CRATE_TMPL.replace("@NAME@", &name.replace("-", "_"));
    let lib_rs_path = path
        .join(&format!("{}_macro", name))
        .join("src")
        .join("lib.rs");
    fs::write(&lib_rs_path, lib_rs_output).map_err(|e| Error::WriteFailed(e, lib_rs_path))?;

    let macro_crate_cargo_toml = macro_path.join("Cargo.toml");
    let mut toml = fs::read_to_string(&macro_crate_cargo_toml)
        .map_err(|e| Error::ReadFailed(e, macro_crate_cargo_toml.clone()))?;
    toml.push_str(
        "syn = { version = \"1\", features = [\"full\", \"extra-traits\"] }
quote = \"1\"
proc-macro2 = \"1\"
",
    );
    fs::write(&macro_crate_cargo_toml, toml)
        .map_err(|e| Error::WriteFailed(e, macro_crate_cargo_toml))?;
    Ok(())
}

fn create_workspace(path: &Path, name: &str) -> Result<(), Error> {
    let workspace_cargo_toml = path.join("Cargo.toml");
    // Add workspace toml
    fs::write(
        &workspace_cargo_toml,
        format!(
            "[workspace]\nmembers = [\n  \"{name}\",\n  \"{name}_macro\",\n]",
            name = &name
        ),
    )
    .map_err(|e| Error::WriteFailed(e, workspace_cargo_toml))
}

fn create_crates(path: PathBuf, name: Option<String>) -> Result<String, Error> {
    let name = match name {
        Some(v) => v,
        None => match path.file_name() {
            Some(v) => v.to_string_lossy().to_string(),
            None => return Err(Error::NameResolutionFailed),
        },
    };

    create_base_crate(&path, &*name)?;
    create_macro_crate(&path, &*name)?;
    create_workspace(&path, &*name)?;

    Ok(name)
}

pub(crate) fn run(args: Vec<String>) -> Result<(), Error> {
    let args = match Args::parse_args(&args, gumdrop::ParsingStyle::AllOptions) {
        Ok(args) if args.help => {
            Args::print_usage();
            exit(0);
        }
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            exit(2);
        }
    };

    let name = match args.command {
        Some(Subcommand::New(a)) => {
            if a.help {
                NewArgs::print_usage();
                exit(0)
            }

            create_crates(a.path, a.name)?
        }
        Some(Subcommand::Init(a)) => {
            if a.help {
                NewArgs::print_usage();
                exit(0)
            }

            let path = a
                .path
                .unwrap_or_else(|| current_dir().expect("Could not resolve current directory"));
            create_crates(path, a.name)?
        }
        None => {
            Args::print_usage();
            exit(2);
        }
    };

    println!(
        "-- Created workspace with `{name}` and `{name}_macro` crates.",
        name = &name
    );
    println!();
    println!(
        "`{name}` is the crate you should use in Rust projects. For example:",
        name = &name
    );
    println!();
    println!("    use {name}::{name};", name = &name);
    println!("    #[{name}]", name = &name);
    println!("    fn some_compatible_element() {{ ... }}");
    println!();
    println!("The testable logic for your macro lives in `{name}_macro` and is a dependency of `{name}`.", name = &name);
    Ok(())
}
