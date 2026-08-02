use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_markdown-style");

/// A unique temp directory for one test, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("mdstyle-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).unwrap();
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn lint_reports_violations_and_exits_one() {
    let dir = TempDir::new("lint");
    let file = dir.write("doc.md", "# Title\n\ntext   \n");

    let output = Command::new(BIN).arg("lint").arg(&file).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("trailing-whitespace: trailing whitespace"));
    assert!(stdout.contains("= why:"));
}

#[test]
fn lint_exits_zero_on_a_clean_file() {
    let dir = TempDir::new("clean");
    let file = dir.write("doc.md", "# Title\n\nHello world\n");

    let status = Command::new(BIN).arg("lint").arg(&file).status().unwrap();

    assert_eq!(status.code(), Some(0));
}

#[test]
fn format_rewrites_the_file_in_place() {
    let dir = TempDir::new("format");
    let file = dir.write("doc.md", "Title\n=====\n\ntext   \n\n\n");

    let status = Command::new(BIN).arg("format").arg(&file).status().unwrap();

    assert_eq!(status.code(), Some(0));
    assert_eq!(fs::read_to_string(&file).unwrap(), "# Title\n\ntext\n");
}

#[test]
fn format_reads_stdin_and_writes_stdout() {
    let mut child = Command::new(BIN)
        .arg("format")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"Title\n=====\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "# Title\n");
}

#[test]
fn explain_prints_a_rule_rationale() {
    let output = Command::new(BIN)
        .arg("explain")
        .arg("final-newline")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("final-newline\n"));
    assert!(stdout.contains("POSIX"));
}

#[test]
fn rules_lists_every_rule_with_its_kind() {
    let output = Command::new(BIN).arg("rules").output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("sentence-per-line"));
    assert!(stdout.contains("heading-increment"));
    // Kinds are labelled: sentence-per-line fixes, heading-increment flags.
    assert!(stdout.contains("flag"));
    assert!(stdout.contains("fix+flag"));
}

#[test]
fn a_missing_path_is_an_operational_error() {
    let output = Command::new(BIN)
        .arg("lint")
        .arg("does-not-exist.md")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error:"));
}

#[test]
fn a_directory_is_walked_for_markdown_files() {
    let dir = TempDir::new("walk");
    dir.write("a.md", "text   \n");
    dir.write("b.txt", "ignored   \n");

    let output = Command::new(BIN).arg("lint").arg(&dir.0).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("a.md"));
    assert!(!stdout.contains("b.txt"));
}
