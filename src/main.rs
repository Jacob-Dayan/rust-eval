use rs_eval::prelude::*;
use rs_eval::{consts::*, *};
use rust_eval as rs_eval;
use std::{fs, process::ExitCode};

/// Entry point for `rs-eval`.
///
/// Loops evaluating inputs until the user signals an exit (via `exit`, `quit`, or Ctrl+C)
/// or an unrecoverable error occurs.
pub fn main() -> ExitCode {
    let mut rl = match create_editor() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("Failed to initialize line editor: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "To enter, stream EOF (ctrl+D on Unix, ctrl+Z on Windows)\nTo exit, press Ctrl+C or type `exit`.\n"
    );

    loop {
        match run(&mut rl) {
            Ok(true) => return ExitCode::SUCCESS,
            Ok(false) => print!("\n\n"),
            Err(e) => {
                eprintln!("{e}");
                clean_temp_dir!();
                return ExitCode::FAILURE;
            }
        }
    }
}

/// Reads a single submission, validates/wraps code in `fn main()` if needed, compiles, and runs it.
///
/// Returns `Ok(true)` if the user requested exit, `Ok(false)` after completing a run.
///
/// # Errors
/// Returns an [`io::Error`] if input reading, file creation, compilation, or execution fails.
pub fn run(rl: &mut EvalEditor) -> io::Result<bool> {
    let Some(mut input) = read_input(rl)? else {
        return Ok(true);
    };

    if input.trim().is_empty() {
        return Ok(false);
    }

    if !MAIN_RE.is_match(&input) {
        if input.contains("fn ") {
            return new_io_error!("No valid main function found.");
        }
        input = format!("fn main() {{\n{input}\n}}");
    }

    clean_temp_dir!();
    fs::create_dir_all(TEMP_DIR)?;

    fs::write(CODE_FILE, format!("{HEADER}{input}{FOOTER}"))?;

    print!("\n\n");
    let result = compile_and_run!();
    clean_temp_dir!();

    result.map(|_| false)
}
