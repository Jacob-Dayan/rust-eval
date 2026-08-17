pub use std::io::{self, BufWriter, Read, Write};
pub mod consts;

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

#[macro_export]
macro_rules! compile_and_run {
    () => {{
        let compile_status = std::process::Command::new("rustc")
            .arg($crate::consts::CODE_FILE)
            .arg("--out-dir")
            .arg($crate::consts::TEMP_DIR)
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
                Err($crate::io::Error::new(
                    $crate::io::ErrorKind::Other,
                    "Process failed",
                ))
            }
        } else {
            Err($crate::io::Error::new(
                $crate::io::ErrorKind::Other,
                "Compilation failed",
            ))
        }
    }};
}
