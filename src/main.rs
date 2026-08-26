use rust_eval::{consts::*, *};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
    process::ExitCode,
};

/// Loop calling [run] until it returns an error or a signal to exit.
///
/// Returns [ExitCode::SUCCESS] if [run] returns `Ok(...)`, otherwise returns [ExitCode::FAILURE].
pub fn main() -> ExitCode {
    let mut rl = match create_editor() {
        Ok(rl) => rl,
        Err(e) => {
            eprintln!("Failed to initialize line editor: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("To enter, stream EOF (ctrl+D on Unix, ctrl+Z on Windows)\nTo exit, press Ctrl+C or type `exit`.\n");

    loop {
        match run(&mut rl) {
            Ok(should_exit) => {
                if should_exit {
                    return ExitCode::SUCCESS;
                } else {
                    print!("\n\n");
                    continue;
                }
            }
            Err(e) => {
                eprintln!("{e}");
                if Path::new(TEMP_DIR).exists() {
                    // The `let _ =` is to suppress the error if the directory cannot be removed,
                    // which would be weird.
                    let _ = fs::remove_dir_all(TEMP_DIR);
                }
                return ExitCode::FAILURE;
            }
        }
    }
}

/// The function that manages the REPL-like loop and evaluates user input.
///
/// Reads user code using [`read_input`] from the [`EvalEditor`],
/// checks if the code contains a main function and wraps it in one if necessary.
/// If the code does not contain a main function, but does contain function definitions,
/// then an error is returned.
///
/// After the code validates, checks if `tmp` folder exists and creates it if it doesn't,
/// then writes the input code to the [`code file`] in `tmp` folder after writing a header comment, and calls
/// [`compile_and_run macro`] to compile and run the code using `rustc`.
/// After running, the `tmp` folder is deleted.
///
/// # Errors
///
/// The function's return type is [`io::Result`], so any errors are returned as [`io::Error`].
///
/// The writing to the [`code file`] is done using a [`io::BufWriter`] for better performance and less IO and system calls.
///
/// [`compile_and_run macro`]: macro@rust_eval::compile_and_run
/// [`code file`]: rust_eval::consts::CODE_FILE
pub fn run(rl: &mut EvalEditor) -> io::Result<bool> {
    let raw_input = read_input(rl)?;

    let mut input = match raw_input {
        Some(line) => line,
        None => return Ok(true), // Ctrl+C or exit received, signal exit
    };

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(false);
    }

    if !MAIN_RE.is_match(&input) {
        // If no main function is found, but there are function definitions,
        // then we can't wrap the code in a main function
        // then we have to return an error.
        if input.contains("fn ") {
            return new_io_error!("No valid main function found.");
        }
        // Here, we can wrap the code in a main function, because there are no function definitions
        // this lets us run code like this:
        // let nums: Vec<i32> = vec![1, 2, 3, 4, 5];
        // let sum: i32 = nums.iter().sum();
        // println!("Sum: {sum}");
        // without having to define a main function
        else {
            input = format!("fn main() {{ {input} }}");
        }
    }

    if Path::new(TEMP_DIR).exists() {
        fs::remove_dir_all(TEMP_DIR)?;
    }
    fs::create_dir_all(TEMP_DIR)?;

    let mut buffer = BufWriter::new(File::create(CODE_FILE)?);
    buffer.write_all(HEADER.as_bytes())?;
    buffer.write_all(input.as_bytes())?;
    buffer.write_all(FOOTER.as_bytes())?;
    buffer.flush()?;

    print!("\n\n");
    let result = compile_and_run!();

    if Path::new(TEMP_DIR).exists() {
        let _ = fs::remove_dir_all(TEMP_DIR);
    }

    result.map(|_| false)
}
