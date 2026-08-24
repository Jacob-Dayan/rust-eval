use rust_eval_core::{consts::*, *};
use std::{
    fs::{self, File},
    io::{BufWriter, Read, Write},
    path::Path,
    process::ExitCode,
};

/// Loop calling [run] until it returns an error or a signal to exit.
///
/// Returns [ExitCode::SUCCESS] if [run] returns `Ok(...)`, otherwise returns [ExitCode::FAILURE].
pub fn main() -> ExitCode {
    loop {
        match run() {
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
/// Gets user's code using [`read_all macro`] macro defined in [`rust_eval_core`],
/// checks if the code contains a main function and wraps it in one if necessary.
/// If the code does not contain a main function, but does contain function definitions,
/// then an error is returned.
///
/// After the code validates, checks if `tmp` folder exists and creates it if it doesn't.
/// then writes the input code to the [`code file`] in `tmp` folder after writing a header comment, and calls
/// [`compile_and_run macro`] to compile and run the code using `rustc`.
/// After running, the `tmp` folder is deleted.
///
/// # Errors
///
/// The function's return type is [`io::Result`], so any errors are returned as [`io::Error`].
/// You might wonder why we don't use [`Result<bool, Box<dyn Error>>`] instead in a function when we might
/// return a custom error (which Box<dyn Error> makes it easy to do with `into()` function); well that's a term of performance:
/// Box<dyn Error> is a trait object, so its size is not known at compile time,
/// then it has to be allocated on the heap.
/// while [`io::Error`] is a struct with a fixed size allowing it to be stored on the stack.
/// We grant performance over convenience, but it is not very noticeable when using the [`new_io_error`] macro.
///
/// The writing to the [`code file`] is done using a [`io::BufWriter`] for better performance and less IO and system calls.
///
/// [`read_all macro`]: macro@rust_eval_core::read_all
/// [`compile_and_run macro`]: macro@rust_eval_core::compile_and_run
/// [`code file`]: rust_eval_core::consts::CODE_FILE
pub fn run() -> io::Result<bool> {
    let mut should_exit = false;
    println!("To enter, stream EOF (ctrl+D on Unix, ctrl+Z on Windows)\nTo exit, type `exit`.\n");
    let mut input = read_all!(format_args!("{PURPLE}>>> {RESET}"))?; // More performant than format!
    if input.trim().to_ascii_lowercase().as_str() == "exit" {
        should_exit = true;
        return Ok(should_exit);
    } else if input.trim().to_ascii_lowercase().as_str() == "clear" {
        std::process::Command::new("clear")
            .spawn()
            .expect("Failed to clear terminal.")
            .wait()
            .unwrap();

        println!(
            "To enter, stream EOF (ctrl+D on Unix, ctrl+Z on Windows)\nTo exit, type `exit`.\n"
        );
        input = read_all!(format_args!("{PURPLE}>>> {RESET}"))?;
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

        /*  ```
            let nums: Vec<i32> = vec![1, 2, 3, 4, 5];
            let sum: i32 = nums.iter().sum();
            println!("Sum: {sum}");
        ```
        */
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
    // printing `\n` twice, so if trying to access stdin,
    // or there are some errors,
    // it doesn't look like the EOF is not recognized or the errors are
    // being injected to the input AND the EOF is not being recognized

    // (You can see for yourself, if you remove this it would look unappealing)
    print!("\n\n");
    compile_and_run!()?;

    fs::remove_dir_all(TEMP_DIR)?;
    Ok(should_exit)
}
