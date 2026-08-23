//! Integration tests for the `rust-eval` suite.
//!
//! This module validates core features of the `rust-eval` workspace, including:
//! - Simulated standard input handling via [`read_all!`].
//! - Code generation, compilation, and execution via [`compile_and_run!`].
//!
//! # Thread Safety Notice
//! Tests interacting with the file system share the constant directory path `./tmp`
//! defined in [`rust_eval_core::consts`]. To prevent race conditions during parallel
//! test execution (`cargo test`), file system operations are serialized using [`ENV_MUTEX`].

#[macro_use]
extern crate rust_eval_core;

use rust_eval_core::consts;
use std::fs;
use std::io::{self, Cursor, Read};
use std::process::Stdio;
use std::sync::Mutex;

/// Global mutex to enforce sequential execution for tests that manipulate
/// the shared `./tmp` directory structure and `rustc` compilation outputs.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Helper function to simulate `stdin` stream reading for macro validation.
///
/// Wraps an in-memory string slice into a [`Cursor`], reads it to completion,
/// and returns the trimmed result string.
///
/// # Arguments
/// * `input` - The string slice simulating incoming `stdin` content.
fn simulate_stdin(input: &str) -> String {
    let mut stdin_mock = Cursor::new(input.as_bytes());
    let mut buffer = String::new();
    stdin_mock.read_to_string(&mut buffer).unwrap();
    buffer.trim().to_string()
}

/// Helper function to guarantee a clean workspace in `./tmp` before and after test execution.
///
/// Ensures the test environment is reset regardless of previous panics or aborted runs.
fn clean_temp_dir() {
    if std::path::Path::new(consts::TEMP_DIR).exists() {
        let _ = fs::remove_dir_all(consts::TEMP_DIR);
    }
}

/// Tests that simulating input streams correctly trims whitespace and preserves input text.
#[test]
fn test_simulate_stdin_variations() {
    // Basic standard input
    assert_eq!(simulate_stdin("hello world"), "hello world");

    // Empty input stream
    assert_eq!(simulate_stdin(""), "");

    // Input surrounded by leading and trailing whitespace
    assert_eq!(simulate_stdin("   trimmed content   "), "trimmed content");

    // Input containing multi-line strings
    assert_eq!(
        simulate_stdin("first line\nsecond line\n"),
        "first line\nsecond line"
    );
}

/// Tests successful compilation and execution of valid Rust source code via [`compile_and_run!`].
///
/// # Errors
/// Returns an [`io::Result`] error if file system setup fails unexpectedly.
#[test]
fn test_compile_and_run_success() -> io::Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();
    clean_temp_dir();

    fs::create_dir_all(consts::TEMP_DIR)?;
    let code = format!(
        "{}\nfn main() {{\n    println!(\"Hello from integration test!\");\n}}\n{}",
        consts::HEADER,
        consts::FOOTER
    );
    fs::write(consts::CODE_FILE, code)?;

    let result = compile_and_run!();
    assert!(
        result.is_ok(),
        "Expected code compilation and execution to succeed"
    );

    clean_temp_dir();
    Ok(())
}

/// Tests that syntax errors in submitted Rust code are caught during `rustc` compilation.
///
/// # Errors
/// Returns an [`io::Result`] error if file system setup fails unexpectedly.
#[test]
fn test_compile_and_run_compilation_failure() -> io::Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();
    clean_temp_dir();

    fs::create_dir_all(consts::TEMP_DIR)?;
    let invalid_code = format!(
        "{}\nfn main() {{\n    println!(\"Missing closing parenthesis\";\n}}\n{}",
        consts::HEADER,
        consts::FOOTER
    );
    fs::write(consts::CODE_FILE, invalid_code)?;

    let result = compile_and_run!(Stdio::null());
    assert!(
        result.is_err(),
        "Expected compilation to fail for invalid syntax"
    );
    assert_eq!(result.unwrap_err().to_string(), "Compilation failed.");

    clean_temp_dir();
    Ok(())
}

/// Tests that programs compiling successfully but panicking at runtime return a non-zero exit status error.
///
/// # Errors
/// Returns an [`io::Result`] error if file system setup fails unexpectedly.
#[test]
fn test_compile_and_run_runtime_panic() -> io::Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();
    clean_temp_dir();

    fs::create_dir_all(consts::TEMP_DIR)?;
    let panic_code = format!(
        "{}\nfn main() {{\n    panic!(\"Explicit runtime test panic!\");\n}}\n{}",
        consts::HEADER,
        consts::FOOTER
    );
    fs::write(consts::CODE_FILE, panic_code)?;

    let result = compile_and_run!();
    assert!(
        result.is_err(),
        "Expected runtime execution to fail due to panic"
    );
    assert_eq!(
        result.unwrap_err().to_string(),
        "Program has exited with a non-zero status."
    );

    clean_temp_dir();
    Ok(())
}

/// Tests basic variable allocations and standard arithmetic operations within compiled Rust code.
///
/// # Errors
/// Returns an [`io::Result`] error if file system setup fails unexpectedly.
#[test]
fn test_compile_and_run_arithmetic_evaluation() -> io::Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap();
    clean_temp_dir();

    fs::create_dir_all(consts::TEMP_DIR)?;
    let arithmetic_code = format!(
        "{}\nfn main() {{\n    let a = 10;\n    let b = 20;\n    assert_eq!(a + b, 30);\n}}\n{}",
        consts::HEADER,
        consts::FOOTER
    );
    fs::write(consts::CODE_FILE, arithmetic_code)?;

    let result = compile_and_run!();
    assert!(
        result.is_ok(),
        "Expected arithmetic expressions to evaluate successfully"
    );

    clean_temp_dir();
    Ok(())
}
