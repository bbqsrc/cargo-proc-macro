// SPDX-License-Identifier: EUPL-1.2+

use std::{
    env::current_dir,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{self, exit, Command},
};

use gumdrop::Options;
use heck::{CamelCase, KebabCase};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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

    #[options(
        default = "attribute",
        help = "Specify the proc-macro kind:
                     \tattribute: --kind={a, attr, attribute}
                     \tderive: --kind={d, derive}
                     \tfunction-like: --kind={f, function}"
    )]
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

    #[options(
        default = "attribute",
        help = "Specify the proc-macro kind:
                     \tattribute: --kind={a, attr, attribute}
                     \tderive: --kind={d, derive}
                     \tfunction-like: --kind={f, function}"
    )]
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
    pub fn lib_impl(&self, name: &str) -> String {
        let snake_name = name.replace("-", "_");
        match self {
            ProcMacroKind::Attr => templates::ATTR_LIB_TMPL.replace("@SNAKE_NAME@", &snake_name),
            ProcMacroKind::Derive => templates::DERIVE_LIB_TMPL
                .replace("@NAME@", name)
                .replace("@SNAKE_NAME@", &snake_name)
                .replace("@STRUCT_NAME@", &name.to_camel_case()),
            ProcMacroKind::Function => {
                templates::FUNCTION_LIB_TMPL.replace("@SNAKE_NAME@", &snake_name)
            }
        }
    }

    pub fn macro_shim(&self, name: &str) -> String {
        let snake_name = name.replace("-", "_");
        match self {
            ProcMacroKind::Attr => templates::ATTR_MACRO_TMPL.replace("@SNAKE_NAME@", &snake_name),
            ProcMacroKind::Derive => templates::DERIVE_MACRO_TMPL
                .replace("@NAME@", name)
                .replace("@SNAKE_NAME@", &snake_name)
                .replace("@STRUCT_NAME@", &name.to_camel_case()),
            ProcMacroKind::Function => {
                templates::FUNCTION_MACRO_TMPL.replace("@SNAKE_NAME@", &snake_name)
            }
        }
    }

    pub fn macro_impl(&self, name: &str) -> String {
        let snake_name = name.replace("-", "_");
        match self {
            ProcMacroKind::Attr => templates::ATTR_IMPL_TMPL.replace("@SNAKE_NAME@", &snake_name),
            ProcMacroKind::Derive => {
                templates::DERIVE_IMPL_TMPL.replace("@SNAKE_NAME@", &snake_name)
            }
            ProcMacroKind::Function => {
                templates::FUNCTION_IMPL_TMPL.replace("@SNAKE_NAME@", &snake_name)
            }
        }
    }

    pub fn print_workspace_msg(&self, path: &Path, name: &str) {
        let choice = if atty::is(atty::Stream::Stdout) {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };
        let mut stdout = StandardStream::stdout(choice);

        let snake_name = name.replace("-", "_");

        let header = templates::WKSP_MSG_HEADER.replace("@NAME@", &name);
        let footer = templates::WKSP_MSG_FOOTER.replace("@NAME@", &name);
        let example = match self {
            ProcMacroKind::Attr => templates::WKSP_ATTR_EXAMPLE
                .replace("@NAME@", &name)
                .replace("@SNAKE_NAME@", &snake_name),
            ProcMacroKind::Derive => templates::WKSP_DERIVE_EXAMPLE
                .replace("@NAME@", &name)
                .replace("@SNAKE_NAME@", &snake_name)
                .replace("@STRUCT_NAME@", &name.to_camel_case()),
            ProcMacroKind::Function => templates::WKSP_FUNCTION_EXAMPLE
                .replace("@NAME@", &name)
                .replace("@SNAKE_NAME@", &snake_name),
        };

        stdout
            .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)))
            .unwrap();
        stdout.write_fmt(format_args!("    Created")).unwrap();
        stdout.reset().unwrap();
        stdout
            .write_fmt(format_args!(
                " workspace `{}` with members `impl` and `macro`\n",
                path.display()
            ))
            .unwrap();

        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))
            .unwrap();
        write!(&mut stdout, "help: ").unwrap();
        stdout.reset().unwrap();

        writeln!(&mut stdout, "{}", header).unwrap();

        for line in example.split("\n") {
            stdout
                .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Blue)))
                .unwrap();
            write!(&mut stdout, "    ").unwrap();
            stdout.reset().unwrap();
            writeln!(&mut stdout, "{}", line).unwrap();
        }

        stdout.reset().unwrap();

        writeln!(&mut stdout, "{}", footer).unwrap();
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

fn cargo_new_lib(path: &Path, name: &str) -> Result<process::Output, Error> {
    if path.join("Cargo.toml").exists() {
        return Err(Error::CargoNewLibFailed(
            std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Cargo.toml already exists at this location",
            ),
            path.join("Cargo.toml"),
        ));
    }

    Command::new("cargo")
        .arg("new")
        .arg("--lib")
        .arg(&path)
        .arg("--name")
        .arg(name)
        .args(&["--vcs", "none"])
        .output()
        .map_err(|e| Error::CargoNewLibFailed(e, path.to_path_buf()))
}

fn cargo_new_workspace(path: &Path, name: &str) -> Result<(), Error> {
    if path.join("Cargo.toml").exists() {
        return Err(Error::CargoNewLibFailed(
            std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Cargo.toml already exists at this location",
            ),
            path.join("Cargo.toml"),
        ));
    }

    Command::new("cargo")
        .arg("new")
        .arg("--lib")
        .arg(&path)
        .arg("--name")
        .arg(name)
        .output()
        .map_err(|e| Error::CargoNewLibFailed(e, path.to_path_buf()))?;

    let workspace_cargo_toml = path.join("Cargo.toml");
    let toml = fs::read_to_string(&workspace_cargo_toml)
        .map_err(|e| Error::ReadFailed(e, workspace_cargo_toml.clone()))?;
    let toml = toml.replace(
        "[dependencies]",
        &format!(
            "[dependencies]
{name}-macro = {{ version = \"=0.1.0\", path = \"macro\" }}

[workspace]
members = [\".\", \"impl\", \"macro\"]
default-members = [\".\", \"impl\"]
",
            name = name
        ),
    );
    fs::write(&workspace_cargo_toml, toml)
        .map_err(|e| Error::WriteFailed(e, workspace_cargo_toml))?;
    Ok(())
}

fn create_impl_crate(path: &Path, name: &str, macro_kind: ProcMacroKind) -> Result<(), Error> {
    let impl_path = path.join("impl");
    cargo_new_lib(&impl_path, &format!("{}-impl", name))?;

    let lib_rs_output = macro_kind.macro_impl(name);
    let lib_rs_path = impl_path.join("src").join("lib.rs");
    fs::write(&lib_rs_path, lib_rs_output).map_err(|e| Error::WriteFailed(e, lib_rs_path))?;

    let impl_cargo_toml = impl_path.join("Cargo.toml");
    let mut toml = fs::read_to_string(&impl_cargo_toml)
        .map_err(|e| Error::ReadFailed(e, impl_cargo_toml.clone()))?;
    toml.push_str(
        "syn = { version = \"1\", features = [\"full\", \"extra-traits\"] }
quote = \"1\"
proc-macro2 = \"1\"
",
    );
    fs::write(&impl_cargo_toml, toml).map_err(|e| Error::WriteFailed(e, impl_cargo_toml))?;
    Ok(())
}

fn create_macro_crate(path: &Path, name: &str, macro_kind: ProcMacroKind) -> Result<(), Error> {
    let macro_path = path.join("macro");
    cargo_new_lib(&macro_path, &format!("{}-macro", name))?;

    let lib_rs_output = macro_kind.macro_shim(name);
    let lib_rs_path = macro_path.join("src").join("lib.rs");
    fs::write(&lib_rs_path, lib_rs_output).map_err(|e| Error::WriteFailed(e, lib_rs_path))?;

    let macro_cargo_toml = macro_path.join("Cargo.toml");
    let mut toml = fs::read_to_string(&macro_cargo_toml)
        .map_err(|e| Error::ReadFailed(e, macro_cargo_toml.clone()))?;
    toml = toml.replace(
        "[dependencies]",
        &format!(
            "[lib]
proc-macro = true

[dependencies]
{}-impl = {{ version = \"=0.1.0\", path = \"../impl\" }}
syn = {{ version = \"1\", features = [\"full\", \"extra-traits\"] }}
quote = \"1\"
proc-macro2 = \"1\"",
            name
        ),
    );

    fs::write(&macro_cargo_toml, toml).map_err(|e| Error::WriteFailed(e, macro_cargo_toml))?;
    Ok(())
}

fn create_workspace(path: &Path, name: &str, macro_kind: ProcMacroKind) -> Result<(), Error> {
    cargo_new_workspace(&path, name)?;

    let lib_rs_output = macro_kind.lib_impl(name);
    let lib_rs_path = path.join("src").join("lib.rs");
    fs::write(&lib_rs_path, lib_rs_output)
        .map_err(|e| Error::WriteFailed(e, lib_rs_path.clone()))?;

    Ok(())
}

fn create_crates(
    path: PathBuf,
    name: Option<String>,
    kind: ProcMacroKind,
) -> Result<String, Error> {
    let name = match name {
        // don't modify user-specified names,
        Some(v) => v,
        None => match path.file_name() {
            Some(v) => {
                let v = v.to_string_lossy().to_string();
                // modify CamelCase names, leave snake_case alone
                if !v.contains('_') {
                    v.to_kebab_case()
                } else {
                    v
                }
            }
            None => return Err(Error::NameResolutionFailed),
        },
    };

    // create_lib_crate(&path, &*name, kind)?;
    create_workspace(&path, &*name, kind)?;
    create_impl_crate(&path, &*name, kind)?;
    create_macro_crate(&path, &*name, kind)?;

    kind.print_workspace_msg(&path, &name);
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
