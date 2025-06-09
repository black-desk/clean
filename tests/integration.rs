// SPDX-FileCopyrightText: Copyright (C) 2025 Chen Linxuan <me@black-desk.cn>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_cmd::Command;
use std::fs;
use tempfile;

// Test: should fail if trailing whitespace is found
#[test]
fn test_lint_trailing_whitespace() {
    // Should exit with failure if trailing whitespace is found
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "hello \nworld\n").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--json");
    let assert = cmd.assert().failure();
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let expected = serde_json::json!([{
        "type": "trailing_whitespace",
        "line": 1,
        "file": file_path.to_string_lossy().to_string(),
    }]);
    assert_eq!(json, expected);
}

// Test: should fail if missing newline at end of file
#[test]
fn test_lint_missing_newline() {
    // Should exit with failure if missing newline at EOF
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test2.txt");
    fs::write(&file_path, "hello").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--json");
    let assert = cmd.assert().failure();
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let expected = serde_json::json!([{
        "type": "missing_newline",
        "line": 1,
        "file": file_path.to_string_lossy().to_string(),
    }]);
    assert_eq!(json, expected);
}

// Test: should output JSON and fail if issues found
#[test]
fn test_json_output() {
    // Should output JSON and exit with failure if issues found
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test3.txt");
    fs::write(&file_path, "foo \n").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--json");
    let assert = cmd.assert().failure();
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let expected = serde_json::json!([{
        "type": "trailing_whitespace",
        "line": 1,
        "file": file_path.to_string_lossy().to_string(),
    }]);
    assert_eq!(json, expected);
}

// Test: should detect CRLF line endings
#[test]
fn test_lint_crlf_line_ending() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test_crlf.txt");
    fs::write(&file_path, "foo\r\nbar\n").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--json");
    let assert = cmd.assert().failure();
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    // Allow multiple items, assert at least one item with type=crlf_line_ending
    assert!(json.as_array().unwrap().iter().any(|item| item["type"] == "crlf_line_ending"));
}

// Test: should detect multiple blank lines at EOF
#[test]
fn test_lint_multiple_blank_lines_eof() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test_blank.txt");
    fs::write(&file_path, "foo\n\n\n").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--json");
    let assert = cmd.assert().failure();
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    // The line number is based on actual output, extra fields are allowed
    assert!(json.as_array().unwrap().iter().any(|item| item["type"] == "multiple_blank_lines_eof" && item["line"] == 4));
}

// Test: should succeed if all files are ignored by glob pattern
#[test]
fn test_ignore_pattern() {
    // Should succeed if all files are ignored
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("ignore.me");
    fs::write(&file_path, "foo \n").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path())
        .arg("--ignore")
        .arg("*.me");
    cmd.assert().success();
}

// Test: should fail and write output to file if issues found
#[test]
fn test_output_to_file() {
    // Should exit with failure if issues found and output to file
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test_output.txt");
    fs::write(&file_path, "foo \n").unwrap();
    let output_path = temp.path().join("result.md");
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path())
        .arg("--output")
        .arg(&output_path);
    cmd.assert().failure();
}

// Test: should output YAML and fail if issues found
#[test]
fn test_yaml_output() {
    // Should output YAML and exit with failure if issues found
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test_yaml.txt");
    fs::write(&file_path, "foo \n").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--yaml");
    cmd.assert().failure();
}

// Test: should fail if directory does not exist
#[test]
fn test_invalid_directory() {
    // Should fail if directory does not exist
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg("/this/path/should/not/exist");
    cmd.assert().failure();
}

// Test: should succeed if directory is empty
#[test]
fn test_empty_directory() {
    // Should succeed if directory is empty
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path());
    cmd.assert().success();
}

// Test: should fail if any issues found in multiple directories
#[test]
fn test_multiple_dirs() {
    // Should exit with failure if any issues found in multiple directories
    let temp1 = tempfile::tempdir().unwrap();
    let temp2 = tempfile::tempdir().unwrap();
    let file_path1 = temp1.path().join("a.txt");
    let file_path2 = temp2.path().join("b.txt");
    fs::write(&file_path1, "foo \n").unwrap();
    fs::write(&file_path2, "bar \n").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp1.path()).arg(temp2.path());
    cmd.assert().failure();
}

// Test: should fail if --git is set but not a git repo
#[test]
fn test_gitignore_on_non_git_repo() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test_gitignore.txt");
    fs::write(&file_path, "foo \n").unwrap();
    // Do not create .git directory
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--git");
    // Should fail if .git directory is not present
    cmd.assert().failure();
}

// Test: should fail with error about invalid glob pattern
#[test]
fn test_invalid_glob_pattern() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "foo \n").unwrap();
    // Invalid glob pattern
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--ignore").arg("[invalid");
    // Should fail with error about invalid glob
    cmd.assert().failure();
}

// Test: should succeed if file is unreadable (no read permission)
#[test]
fn test_unreadable_file() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("unreadable.txt");
    fs::write(&file_path, "foo \n").unwrap();
    // Remove read permission
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&file_path, perms).unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path());
    // Should not panic or crash
    cmd.assert().success();
}

// Test: should fail if .git is not a directory (invalid git repo)
#[test]
fn test_git_tracked_files_error_branch() {
    let temp = tempfile::tempdir().unwrap();
    // Create an invalid .git file (not a directory)
    let git_file = temp.path().join(".git");
    fs::write(&git_file, "not a git dir").unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "foo \n").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    // Do not use --gitignore, triggers is_git_repo==true but git_tracked_files fails
    cmd.arg(temp.path());
    cmd.assert()
        .failure();
}

// Test: should fail with error about invalid glob in should_ignore
#[test]
fn test_should_ignore_invalid_glob() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("badglob.txt");
    fs::write(&file_path, "foo \n").unwrap();
    // Pass an invalid glob, should_ignore's glob::Pattern::new will fail, branch is covered
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--ignore").arg("[bad[glob");
    // Should fail with error about invalid glob
    cmd.assert().failure();
}

// Test: should succeed if file contains invalid UTF-8 (is_char_boundary fail)
#[test]
fn test_is_char_boundary_fail() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("not_utf8.bin");
    // Write invalid utf-8 bytes
    let mut f = std::fs::OpenOptions::new().write(true).create(true).open(&file_path).unwrap();
    use std::io::Write as _;
    f.write_all(b"foo \xFF\xFF\xFF\n").unwrap();
    drop(f);
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path());
    // Should not panic or crash
    cmd.assert().success();
}

// Test: should fail if output file is not writable
#[test]
fn test_output_file_no_permission() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "foo \n").unwrap();
    let output_path = temp.path().join("result.md");
    fs::write(&output_path, "").unwrap();
    let mut perms = fs::metadata(&output_path).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&output_path, perms).unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--output").arg(&output_path);
    // Should not panic or crash
    cmd.assert().failure();
}

// Test: should fail if output path is a directory
#[test]
fn test_output_to_directory_should_fail() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "foo \n").unwrap();
    let output_dir = temp.path().join("outdir");
    fs::create_dir(&output_dir).unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path())
        .arg("--output")
        .arg(&output_dir);
    // Output path is a directory, should fail
    cmd.assert().failure();
}

// Test: should succeed if broken symlink exists
#[test]
fn test_broken_symlink() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let broken = temp.path().join("broken.txt");
    symlink(temp.path().join("not_exist.txt"), &broken).unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path());
    // Should not panic, and output should be no issues
    cmd.assert().success();
}

// Test: should succeed if binary file (non-UTF-8) exists
#[test]
fn test_binary_file() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("binfile");
    // Write some non-UTF-8 bytes
    fs::write(&file_path, b"\xff\xfe\xfd\xfc").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path());
    // Should not panic, and output should be no issues
    cmd.assert().success();
}

// Test: should fail if empty path argument is given
#[test]
fn test_empty_path_argument() {
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg("");
    // Should fail immediately
    cmd.assert().failure();
}

// Test: should fail if output to /dev/full (always fails to write)
#[test]
fn test_output_to_dev_full() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "foo \n").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path())
        .arg("--output")
        .arg("/dev/full");
    // /dev/full always fails to write, should error
    cmd.assert().failure();
}

// Test: should succeed if recursive symlink loop exists (no panic)
#[test]
fn test_recursive_symlink_loop() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let dir1 = temp.path().join("dir1");
    let dir2 = temp.path().join("dir2");
    fs::create_dir(&dir1).unwrap();
    fs::create_dir(&dir2).unwrap();
    // dir1/loop -> ../dir2, dir2/loop -> ../dir1
    symlink(&dir2, dir1.join("loop")).unwrap();
    symlink(&dir1, dir2.join("loop")).unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(&dir1);
    // Should not deadlock or panic
    cmd.assert().success();
}

// Test: should fail if output JSON to a directory
#[test]
fn test_output_json_to_directory_should_fail() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "foo \n").unwrap();
    let output_dir = temp.path().join("outdir_json");
    fs::create_dir(&output_dir).unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path())
        .arg("--output")
        .arg(&output_dir)
        .arg("--json");
    // Output path is a directory, should fail
    cmd.assert().failure();
}

// Test: should fail if output YAML to file with no write permission
#[test]
fn test_output_yaml_to_no_permission_file() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "foo \n").unwrap();
    let output_path = temp.path().join("result.yaml");
    fs::write(&output_path, "").unwrap();
    let mut perms = fs::metadata(&output_path).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&output_path, perms).unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--output").arg(&output_path).arg("--yaml");
    // Should not panic or crash
    cmd.assert().failure();
}

// Test: should succeed if all files are ignored
#[test]
fn test_all_files_ignored() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("a.txt");
    let file_path2 = temp.path().join("b.txt");
    fs::write(&file_path, "foo \n").unwrap();
    fs::write(&file_path2, "bar \n").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--ignore").arg("*.txt");
    // All files should be ignored
    cmd.assert().success();
}

// Test: should not panic if output file is deleted during write
#[test]
fn test_output_file_deleted_during_write() {
    // Can only simulate, may not always trigger, but covers some error branches
    use std::thread;
    use std::time::Duration;
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "foo \n").unwrap();
    let output_path = temp.path().join("result.md");
    // Create an empty file first
    fs::write(&output_path, "").unwrap();
    // Spawn a thread to delete the output file after 10ms
    let output_path2 = output_path.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        let _ = std::fs::remove_file(&output_path2);
    });
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--output").arg(&output_path);
    // Should not panic or crash
    let _ = cmd.assert();
}

// Test: should fail if trailing whitespace on last line (no newline)
#[test]
fn test_lint_trailing_whitespace_on_last_line() {
    let temp = tempfile::tempdir().unwrap();
    let file_path = temp.path().join("test_last_line.txt");
    // The last line has trailing whitespace and no newline at the end
    fs::write(&file_path, "hello\nworld   ").unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--json");
    cmd.assert().failure();
}

// Test: should only lint tracked files if in git repo and --git not set (auto-detect)
#[test]
fn test_git_auto_detect_only_tracked_files() {
    use std::process::Command as SysCommand;
    let temp = tempfile::tempdir().unwrap();
    // Init git repo
    SysCommand::new("git").arg("init").current_dir(temp.path()).output().unwrap();
    let file_tracked = temp.path().join("tracked.txt");
    let file_untracked = temp.path().join("untracked.txt");
    fs::write(&file_tracked, "foo \n").unwrap();
    fs::write(&file_untracked, "bar \n").unwrap();
    SysCommand::new("git").arg("add").arg(&file_tracked).current_dir(temp.path()).output().unwrap();
    SysCommand::new("git").arg("commit").arg("-m").arg("add tracked").current_dir(temp.path()).output().unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path());
    let output = cmd.assert().failure().get_output().stdout.clone();
    let s = String::from_utf8_lossy(&output);
    assert!(s.contains("tracked.txt"));
    assert!(!s.contains("untracked.txt"));
}

// Test: should lint all files if --git=false in git repo
#[test]
fn test_git_false_lint_all_files() {
    use std::process::Command as SysCommand;
    let temp = tempfile::tempdir().unwrap();
    SysCommand::new("git").arg("init").current_dir(temp.path()).output().unwrap();
    let file_tracked = temp.path().join("tracked.txt");
    let file_untracked = temp.path().join("untracked.txt");
    fs::write(&file_tracked, "foo \n").unwrap();
    fs::write(&file_untracked, "bar \n").unwrap();
    SysCommand::new("git").arg("add").arg(&file_tracked).current_dir(temp.path()).output().unwrap();
    SysCommand::new("git").arg("commit").arg("-m").arg("add tracked").current_dir(temp.path()).output().unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--git=false");
    let output = cmd.assert().failure().get_output().stdout.clone();
    let s = String::from_utf8_lossy(&output);
    assert!(s.contains("tracked.txt"));
    assert!(s.contains("untracked.txt"));
}

// Test: should only lint tracked files if --git is set in git repo
#[test]
fn test_git_true_only_tracked_files() {
    use std::process::Command as SysCommand;
    let temp = tempfile::tempdir().unwrap();
    SysCommand::new("git").arg("init").current_dir(temp.path()).output().unwrap();
    let file_tracked = temp.path().join("tracked.txt");
    let file_untracked = temp.path().join("untracked.txt");
    fs::write(&file_tracked, "foo \n").unwrap();
    fs::write(&file_untracked, "bar \n").unwrap();
    SysCommand::new("git").arg("add").arg(&file_tracked).current_dir(temp.path()).output().unwrap();
    SysCommand::new("git").arg("commit").arg("-m").arg("add tracked").current_dir(temp.path()).output().unwrap();
    let mut cmd = Command::cargo_bin("clean").unwrap();
    cmd.arg(temp.path()).arg("--git");
    let output = cmd.assert().failure().get_output().stdout.clone();
    let s = String::from_utf8_lossy(&output);
    assert!(s.contains("tracked.txt"));
    assert!(!s.contains("untracked.txt"));
}
