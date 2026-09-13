use rs_eval::prelude::*;
use rs_eval::{consts::*, *};
use rust_eval as rs_eval;
use std::{fs, process::ExitCode};

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

/// Reads code from stdin, auto-wraps in `fn main` if omitted, and runs it via `rustc`.
///
/// Returns `Ok(true)` on exit signals, or `Ok(false)` to continue the REPL loop.
///
/// Note: Uses a dumb hack of `input.contains("fn ")` check to reject functions without a `main()`,
/// which will false-positive on `"fn "` inside string literals or comments.
/// will be pathced in the future.
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
    fs::create_dir_all(&*TEMP_DIR)?;

    fs::write(&*CODE_FILE, format!("{HEADER}{input}{FOOTER}"))?;

    print!("\n\n");
    let result = compile_and_run!();
    clean_temp_dir!();

    result.map(|_| false)
}
