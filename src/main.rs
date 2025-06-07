// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: Copyright (C) 2025 Chen Linxuan <me@black-desk.cn>

use anyhow::Result;
use clap::{ArgAction, Parser};
use log::{error, warn};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    author = "Chen Linxuan <me@black-desk.cn>",
    version,
    about = "A simple CLI linter for whitespace and line ending issues in text files.",
    long_about = "A simple command-line tool to lint text files for common whitespace and
line ending issues. Checks for trailing whitespace, missing newline at
end of file, CRLF line endings, and multiple blank lines at EOF.

See the full documentation and usage examples at:
https://github.com/black-desk/clean#readme"
)]
struct Cli {
    /// Output results in JSON format
    #[arg(long, action = ArgAction::SetTrue)]
    json: bool,
    /// Output results in YAML format
    #[arg(long, action = ArgAction::SetTrue)]
    yaml: bool,
    /// Ignore file or path (supports glob, can be set multiple times)
    #[arg(long, value_name = "PATTERN", num_args = 0.., action = ArgAction::Append)]
    ignore: Vec<String>,
    /// Write output to file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Directories to lint (default: current directory)
    #[arg(value_name = "DIR", default_value = ".", num_args = 0..)]
    dirs: Vec<PathBuf>,
    /// Only lint files tracked by git (auto-enabled in git repo)
    ///
    /// If not set, tracked files are linted only if the directory is a git repository.
    /// If set to true, only git tracked files are linted.
    /// If set to false, all files (not just tracked) are linted, even in a git repository.
    #[arg(long, value_parser = clap::value_parser!(bool), num_args = 0..=1, default_missing_value = "true", action = ArgAction::Set)]
    git: Option<bool>,
}

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "snake_case")]
enum IssueType {
    TrailingWhitespace,
    MissingNewline,
    CrlfLineEnding,
    MultipleBlankLinesEof,
}

#[derive(Debug, serde::Serialize, Clone)]
struct Issue {
    #[serde(rename = "type")]
    issue_type: IssueType,
    line: Option<usize>,
    file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

fn is_git_repo(dir: &std::path::Path) -> bool {
    dir.join(".git").exists()
}

fn git_tracked_files(dir: &std::path::Path) -> anyhow::Result<HashSet<String>> {
    let output = Command::new("git")
        .arg("ls-files")
        .current_dir(dir)
        .output()?;
    if !output.status.success() {
        match output.status.code() {
            Some(code) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("`git ls-files` exit with code={}: {}", code, stderr.trim());
            }
            _ => {
                anyhow::bail!(
                    "`git ls-files` killed by signal: {}",
                    output.status.signal().unwrap()
                );
            }
        }
    }
    let files = String::from_utf8_lossy(&output.stdout);
    Ok(files
        .lines()
        .map(|l| dir.join(l).to_string_lossy().to_string())
        .collect())
}

fn should_ignore(path: &str, ignores: &[String]) -> Result<bool, glob::PatternError> {
    for pat in ignores {
        let pat_obj = glob::Pattern::new(pat)?;
        if pat_obj.matches(path) {
            return Ok(true);
        }
        if pat_obj.matches(
            PathBuf::from(path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .as_ref(),
        ) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn lint_file(path: &str, content: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = content.split('\n').collect();
    for (i, line) in lines.iter().enumerate().take(lines.len().saturating_sub(1)) {
        if line.trim_end().len() != line.len() {
            issues.push(Issue {
                issue_type: IssueType::TrailingWhitespace,
                line: Some(i + 1),
                file: path.to_string(),
                message: Some("Trailing whitespace".into()),
            });
        }
    }
    if let Some(last) = lines.last() {
        if last.trim_end().len() != last.len() {
            issues.push(Issue {
                issue_type: IssueType::TrailingWhitespace,
                line: Some(lines.len()),
                file: path.to_string(),
                message: Some("Trailing whitespace".into()),
            });
        }
    }
    if !content.ends_with('\n') {
        issues.push(Issue {
            issue_type: IssueType::MissingNewline,
            line: Some(lines.len()),
            file: path.to_string(),
            message: Some("Missing newline at end of file".into()),
        });
    }
    if content.contains("\r\n") {
        for (i, line) in lines.iter().enumerate() {
            if line.contains('\r') {
                issues.push(Issue {
                    issue_type: IssueType::CrlfLineEnding,
                    line: Some(i + 1),
                    file: path.to_string(),
                    message: Some("Contains CRLF line endings".into()),
                });
            }
        }
    }
    if !content.is_empty() {
        let mut n = 0;
        for c in content.chars().rev() {
            if c == '\n' || c == '\r' {
                n += 1;
            } else {
                break;
            }
        }
        if n > 1 {
            issues.push(Issue {
                issue_type: IssueType::MultipleBlankLinesEof,
                line: Some(lines.len()),
                file: path.to_string(),
                message: Some("Multiple blank lines at end of file".into()),
            });
        }
    }
    issues
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    let mut all_issues = Vec::new();
    for dir in &cli.dirs {
        if !dir.exists() {
            anyhow::bail!("Directory not found: {}", dir.display());
        }
        let in_git_repo = is_git_repo(dir);
        let use_git = match cli.git {
            None => in_git_repo,
            Some(true) => true,
            Some(false) => false,
        };
        let mut tracked_files = None;
        if use_git {
            tracked_files = Some(git_tracked_files(dir)?);
        }
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let path_str = path.to_string_lossy();
            if let Some(ref files) = tracked_files {
                if !files.contains(&path.to_string_lossy().to_string()) {
                    continue;
                }
            }
            match should_ignore(&path_str, &cli.ignore) {
                Ok(true) => continue,
                Ok(false) => {}
                Err(e) => {
                    error!("Invalid glob pattern: {}", e);
                    std::process::exit(1);
                }
            }
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    warn!("failed to read file '{}': {}", path_str, e);
                    continue;
                }
            };
            if !content.is_char_boundary(content.len()) {
                warn!(
                    "file '{}' is not a valid UTF-8 text file, skipped",
                    path_str
                );
                continue;
            }
            let issues = lint_file(&path_str, &content);
            all_issues.extend(issues);
        }
    }
    // Output
    let mut out: Box<dyn Write> = if let Some(ref p) = cli.output {
        match fs::File::create(p) {
            Ok(f) => Box::new(f),
            Err(e) => {
                if let Ok(md) = fs::metadata(p) {
                    if md.is_dir() {
                        anyhow::bail!("output path is a directory: {}", p.display());
                    } else {
                        anyhow::bail!("failed to write output file {}: {}", p.display(), e);
                    }
                } else {
                    anyhow::bail!("failed to write output file {}: {}", p.display(), e);
                }
            }
        }
    } else {
        Box::new(io::stdout())
    };
    if cli.json {
        for i in &mut all_issues {
            i.message = None;
        }
        serde_json::to_writer_pretty(&mut out, &all_issues)?;
        if all_issues.is_empty() {
            return Ok(());
        }
        anyhow::bail!("issues found");
    }
    if cli.yaml {
        for i in &mut all_issues {
            i.message = None;
        }
        serde_yaml::to_writer(&mut out, &all_issues)?;
        if all_issues.is_empty() {
            return Ok(());
        }
        anyhow::bail!("issues found");
    }
    writeln!(out, "# Clean report\n")?;
    for dir in &cli.dirs {
        let mut cur_file = "";
        for issue in all_issues
            .iter()
            .filter(|i| i.file.starts_with(&*dir.to_string_lossy()))
        {
            if issue.file != cur_file {
                if !cur_file.is_empty() {
                    writeln!(out)?;
                }
                writeln!(out, "## {}\n", issue.file)?;
                cur_file = &issue.file;
            }
            writeln!(
                out,
                "- **Line:** `{}` {}",
                issue.line.unwrap_or(0),
                issue.message.as_deref().unwrap_or("")
            )?;
        }
        writeln!(out)?;
    }
    if all_issues.is_empty() {
        writeln!(out, "No lint issues found.\n")?;
        return Ok(());
    }
    anyhow::bail!("issues found");
}
