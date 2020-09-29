// SPDX-License-Identifier: EUPL-1.2+

use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
    process::{self, exit, Command},
};

use gumdrop::Options;

mod templates;

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

    #[options(default = "attribute",
              help = "Specify the proc-macro kind:
                     \tattribute: --kind={a, attr, attribute}
                     \tderive: --kind={d, derive}
                     \tfunction-like: --kind={f, function}")]
    kind: ProcMacroKind,

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

    #[options(default = "attribute",
              help = "Specify the proc-macro kind:
                     \tattribute: --kind={a, attr, attribute}
                     \tderive: --kind={d, derive}
                     \tfunction-like: --kind={f, function}")]
    kind: ProcMacroKind,

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

#[derive(Debug, Copy, Clone)]
enum ProcMacroKind {
    Attr,
    Derive,
    Function,
}

impl std::str::FromStr for ProcMacroKind {
    type Err = String;
    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        match kind {
            "a" | "attr" | "attribute" => Ok(ProcMacroKind::Attr),
            "d" | "derive" => Ok(ProcMacroKind::Derive),
            "f" | "function" => Ok(ProcMacroKind::Function),
            _ => Err(format!("`{}`", kind)),
        }
    }
}

impl ProcMacroKind {
    pub fn base_impl(&self, name: &str) -> String {
        let snake_name = name.replace("-", "_");
        match self {
            ProcMacroKind::Attr =>
                templates::ATTR_BASE_TMPL
                 .replace("@SNAKE_NAME@", &snake_name),
            ProcMacroKind::Derive =>
                templates::DERIVE_BASE_TMPL
                 .replace("@NAME@", name)
                 .replace("@SNAKE_NAME@", &snake_name),
            ProcMacroKind::Function =>
                templates::FUNCTION_BASE_TMPL
                .replace("@SNAKE_NAME@", &snake_name),
        }
    }
    pub fn crate_impl(&self, name: &str) -> String {
        let snake_name = name.replace("-", "_");
        match self {
            ProcMacroKind::Attr =>
                templates::ATTR_CRATE_TMPL
                 .replace("@SNAKE_NAME@", &snake_name),
            ProcMacroKind::Derive =>
                templates::DERIVE_CRATE_TMPL
                 .replace("@SNAKE_NAME@", &snake_name),
            ProcMacroKind::Function =>
                templates::FUNCTION_CRATE_TMPL
                 .replace("@NAME@", &snake_name),
        }
    }
    pub fn workspace_msg(&self, name: &str) -> String {
        let snake_name = name.replace("-", "_");
        match self {
            ProcMacroKind::Attr =>
                templates::ATTR_WKSP_MSG
                 .replace("@NAME@", &name)
                 .replace("@SNAKE_NAME@", &snake_name),
            ProcMacroKind::Derive =>
                templates::DERIVE_WKSP_MSG
                 .replace("@NAME@", &name)
                 .replace("@SNAKE_NAME@", &snake_name),
            ProcMacroKind::Function =>
                templates::FUNCTION_WKSP_MSG
                 .replace("@NAME@", &name)
                 .replace("@SNAKE_NAME@", &snake_name),
        }
    }
}

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
fn create_base_crate(path: &Path, name: &str, macro_kind: ProcMacroKind) -> Result<(), Error> {
    let base_path = path.join(&name);
    cargo_new_lib(&base_path)?;

    let lib_rs_output = macro_kind.base_impl(name);
    let lib_rs_path = base_path.join("src").join("lib.rs");
    fs::write(&lib_rs_path, lib_rs_output)
        .map_err(|e| Error::WriteFailed(e, lib_rs_path.clone()))?;

    write_proc_macro_cargo_toml(base_path.join("Cargo.toml"), &*name)?;
    Ok(())
}

// Make macro crate with actual logic
fn create_macro_crate(path: &Path, name: &str, macro_kind: ProcMacroKind) -> Result<(), Error> {
    let macro_path = path.join(&format!("{}_macro", name));
    cargo_new_lib(&macro_path)?;

    let lib_rs_output = macro_kind.crate_impl(name);
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

fn create_crates(path: PathBuf, name: Option<String>, kind: ProcMacroKind) -> Result<String, Error> {
    let name = match name {
        Some(v) => v,
        None => match path.file_name() {
            Some(v) => v.to_string_lossy().to_string(),
            None => return Err(Error::NameResolutionFailed),
        },
    };

    create_base_crate(&path, &*name, kind)?;
    create_macro_crate(&path, &*name, kind)?;
    create_workspace(&path, &*name)?;

    println!("{}", kind.workspace_msg(&name));
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
            eprintln!("error: {}\n", e);
            Args::print_usage();
            exit(2);
        }
    };

    match args.command {
        Some(Subcommand::New(a)) => {
            if a.help {
                NewArgs::print_usage();
                exit(0)
            }

            create_crates(a.path, a.name, a.kind)?;
            Ok(())
        }
        Some(Subcommand::Init(a)) => {
            if a.help {
                NewArgs::print_usage();
                exit(0)
            }

            let path = a
                .path
                .unwrap_or_else(|| current_dir().expect("Could not resolve current directory"));
            create_crates(path, a.name, a.kind)?;
            Ok(())
        }
        None => {
            Args::print_usage();
            exit(2);
        }
    }
}
