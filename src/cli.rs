//! Command-line surface: the `lint`, `format`, and `explain` subcommands.

use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use ignore::WalkBuilder;

use crate::{Document, default_rules, format, lint, report};

const EXIT_VIOLATIONS: u8 = 1;
const EXIT_ERROR: u8 = 2;

#[derive(Parser)]
#[command(
    name = "markdown-style",
    version,
    about = "An opinionated Markdown linter and formatter"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Report style violations without changing any files.
    Lint {
        /// Files, directories, or `-` for stdin.
        #[arg(required = true)]
        paths: Vec<String>,
    },
    /// Rewrite files in place to fix violations.
    Format {
        /// Files, directories, or `-` for stdin (written to stdout).
        #[arg(required = true)]
        paths: Vec<String>,
    },
    /// Explain a rule's reasoning.
    Explain {
        /// The rule id, for example `sentence-per-line`.
        rule_id: String,
    },
}

/// Parse arguments and run. Returns the process exit code.
pub fn run() -> ExitCode {
    match Cli::parse().command {
        Command::Lint { paths } => run_lint(&paths),
        Command::Format { paths } => run_format(&paths),
        Command::Explain { rule_id } => run_explain(&rule_id),
    }
}

fn run_lint(paths: &[String]) -> ExitCode {
    let rules = default_rules();
    let color = io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();

    let mut any_violations = false;
    for target in &collect(paths) {
        let (name, source) = match read(target) {
            Ok(input) => input,
            Err(message) => return fail(&message),
        };
        let violations = lint(&Document::new(&source), &rules);
        if !violations.is_empty() {
            any_violations = true;
            print!(
                "{}",
                report::render(&name, &source, &violations, &rules, color)
            );
        }
    }

    if any_violations {
        ExitCode::from(EXIT_VIOLATIONS)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_format(paths: &[String]) -> ExitCode {
    let rules = default_rules();
    for target in &collect(paths) {
        let (_, source) = match read(target) {
            Ok(input) => input,
            Err(message) => return fail(&message),
        };
        let formatted = format(&source, &rules);
        match target {
            Target::Stdin => print!("{formatted}"),
            Target::File(path) if formatted != source => {
                if let Err(error) = fs::write(path, &formatted) {
                    return fail(&format!("{}: {error}", path.display()));
                }
            }
            Target::File(_) => {}
        }
    }
    ExitCode::SUCCESS
}

fn run_explain(rule_id: &str) -> ExitCode {
    match report::explain(rule_id, &default_rules()) {
        Some(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        None => fail(&format!("unknown rule: {rule_id}")),
    }
}

enum Target {
    Stdin,
    File(PathBuf),
}

/// Expand the path arguments into concrete targets, in the order given. A single
/// argument that fails to expand aborts the run (fail-fast).
fn collect(paths: &[String]) -> Vec<Target> {
    // Expansion errors surface at read time via a missing/unreadable path, so
    // this only widens directories; individual files are validated when read.
    let mut targets = Vec::new();
    for path in paths {
        if path == "-" {
            targets.push(Target::Stdin);
        } else if Path::new(path).is_dir() {
            targets.extend(markdown_files(Path::new(path)));
        } else {
            targets.push(Target::File(PathBuf::from(path)));
        }
    }
    targets
}

fn markdown_files(dir: &Path) -> Vec<Target> {
    let mut files: Vec<PathBuf> = WalkBuilder::new(dir)
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .map(ignore::DirEntry::into_path)
        .filter(|path| is_markdown(path))
        .collect();
    files.sort();
    files.into_iter().map(Target::File).collect()
}

fn is_markdown(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("md" | "markdown")
    )
}

fn read(target: &Target) -> Result<(String, String), String> {
    match target {
        Target::Stdin => {
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .map_err(|error| format!("<stdin>: {error}"))?;
            Ok(("<stdin>".to_string(), source))
        }
        Target::File(path) => {
            let source =
                fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
            Ok((path.display().to_string(), source))
        }
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("error: {message}");
    ExitCode::from(EXIT_ERROR)
}
