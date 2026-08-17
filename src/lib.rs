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
macro_rules! print_results {
    ($output:expr) => {
        println!("===STDOUT===");
        io::stdout().write_all(&$output.stdout)?;
        println!("===STDERR===");
        io::stderr().write_all(&$output.stderr)?;
        println!("===EXIT CODE===");
        eprintln!("{}", $output.status);
    };
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
            let output = std::process::Command::new($crate::consts::EXEC_FILE).output()?;
            $crate::print_results!(output);
        }
    }};
}
