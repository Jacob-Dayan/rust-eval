pub use std::io::{self, BufWriter, Read, Write};
pub mod consts;

#[macro_export]
macro_rules! new_io_error {
    ($e:expr) => {
        Err(std::io::Error::new(std::io::ErrorKind::Other, $e))
    };
}

/// Reads all input from `stdin` until `EOF`, displaying a prompt first.
///
/// # Errors
/// Returns an [`std::io::Error`] if reading from `stdin` or flushing `stdout` fails.
#[macro_export]
macro_rules! read_all {
    ($prompt:expr) => {{
        print!("{}", $prompt);
        $crate::io::stdout().flush()?;
        let mut input = String::new();
        $crate::io::stdin().read_to_string(&mut input)?;
        Ok::<String, $crate::io::Error>(input.trim().to_string())
    }};
}

/// Calls `rustc` to compile the code, then runs the compiled binary with `stdin`, `stdout`, and `stderr` redirected.
///
/// # Errors
/// Because `rustc` prints to stderr by itself, we do not have to catch any errors from it and print them.
/// That lets us focus on whether the compilation or execution was successful.
/// if successful returns [`Result::Ok`], otherwise returns a new [`std::io::Error`] with a descriptive message.
#[macro_export]
macro_rules! compile_and_run {
    () => {
        $crate::compile_and_run!(std::process::Stdio::inherit())
    };
    ($rustc_stderr:expr) => {{
        let compile_status = std::process::Command::new("rustc")
            .arg($crate::consts::CODE_FILE)
            .arg("--out-dir")
            .arg($crate::consts::TEMP_DIR)
            .stderr($rustc_stderr)
            .status()?;

        if compile_status.success() {
            let status = std::process::Command::new($crate::consts::EXEC_FILE)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()?;

            if status.success() {
                Ok::<(), $crate::io::Error>(())
            } else {
                new_io_error!("Program has exited with a non-zero status.")
            }
        } else {
            new_io_error!("Compilation failed.")
        }
    }};
}
