#[macro_use]
extern crate rust_eval as rs_eval;

use rs_eval::consts;
use std::fs;
use std::io::{self, Cursor, Read};
use std::process::Stdio;
use std::sync::Mutex;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

macro_rules! simulate_stdin {
    ($input:expr) => {{
        let mut stdin_mock = Cursor::new($input.as_bytes());
        let mut buffer = String::new();
        stdin_mock.read_to_string(&mut buffer).unwrap();
        buffer.trim().to_string()
    }};
}

#[test]
fn test_simulate_stdin_variations() {
    assert_eq!(simulate_stdin!("hello world"), "hello world");
    assert_eq!(simulate_stdin!(""), "");
    assert_eq!(simulate_stdin!("   trimmed content   "), "trimmed content");
    assert_eq!(
        simulate_stdin!("first line\nsecond line\n"),
        "first line\nsecond line"
    );
}

#[test]
fn test_compile_and_run_success() -> io::Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    clean_temp_dir!();

    fs::create_dir_all(&*consts::TEMP_DIR)?;
    let code = format!(
        "{}\nfn main() {{\n    println!(\"Hello from integration test!\");\n}}\n{}",
        consts::HEADER,
        consts::FOOTER
    );
    fs::write(&*consts::CODE_FILE, code)?;

    let result = compile_and_run!();
    assert!(
        result.is_ok(),
        "Expected code compilation and execution to succeed"
    );

    clean_temp_dir!();
    Ok(())
}

#[test]
fn test_compile_and_run_compilation_failure() -> io::Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    clean_temp_dir!();

    fs::create_dir_all(&*consts::TEMP_DIR)?;
    let invalid_code = format!(
        "{}\nfn main() {{\n    println!(\"Missing closing parenthesis\";\n}}\n{}",
        consts::HEADER,
        consts::FOOTER
    );
    fs::write(&*consts::CODE_FILE, invalid_code)?;

    let result = compile_and_run!(Stdio::null());
    assert!(
        result.is_err(),
        "Expected compilation to fail for invalid syntax"
    );
    assert_eq!(result.unwrap_err().to_string(), "Compilation failed.");

    clean_temp_dir!();
    Ok(())
}

#[test]
fn test_compile_and_run_runtime_panic() -> io::Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    clean_temp_dir!();

    fs::create_dir_all(&*consts::TEMP_DIR)?;
    let panic_code = format!(
        "{}\nfn main() {{\n    panic!(\"Explicit runtime test panic!\");\n}}\n{}",
        consts::HEADER,
        consts::FOOTER
    );
    fs::write(&*consts::CODE_FILE, panic_code)?;

    let result = compile_and_run!();
    assert!(
        result.is_err(),
        "Expected runtime execution to fail due to panic"
    );
    assert_eq!(
        result.unwrap_err().to_string(),
        "Program has exited with a non-zero status."
    );

    clean_temp_dir!();
    Ok(())
}

#[test]
fn test_compile_and_run_arithmetic_evaluation() -> io::Result<()> {
    let _guard = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    clean_temp_dir!();

    fs::create_dir_all(&*consts::TEMP_DIR)?;
    let arithmetic_code = format!(
        "{}\nfn main() {{\n    let a = 10;\n    let b = 20;\n    assert_eq!(a + b, 30);\n}}\n{}",
        consts::HEADER,
        consts::FOOTER
    );
    fs::write(&*consts::CODE_FILE, arithmetic_code)?;

    let result = compile_and_run!();
    assert!(
        result.is_ok(),
        "Expected arithmetic expressions to evaluate successfully"
    );

    clean_temp_dir!();
    Ok(())
}

#[test]
fn test_default_editor_and_history() {
    use rs_eval::{create_editor, prelude::*};
    let mut rl = DefaultEditor::new().expect("Failed to create DefaultEditor");
    let test_line = "let x = 42;";
    let added = rl.add_history_entry(test_line);
    assert!(added.is_ok(), "Failed to add history entry");

    let mut eval_rl = create_editor().expect("Failed to create EvalEditor");
    let added_eval = eval_rl.add_history_entry(test_line);
    assert!(
        added_eval.is_ok(),
        "Failed to add history entry to EvalEditor"
    );
}

#[test]
fn test_eval_helper_nav_action() {
    use rs_eval::{NavAction, RustEvalHelper};
    let helper = RustEvalHelper::new();
    assert_eq!(helper.get_nav_action(), NavAction::Enter);

    helper.set_nav_action(NavAction::PrevLine);
    assert_eq!(helper.get_nav_action(), NavAction::PrevLine);

    helper.set_nav_action(NavAction::NextLine);
    assert_eq!(helper.get_nav_action(), NavAction::NextLine);

    helper.set_nav_action(NavAction::Submit);
    assert_eq!(helper.get_nav_action(), NavAction::Submit);
}
