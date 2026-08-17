use rust_eval::{consts::*, *};
use std::{
    fs::{self, File},
    io::{BufWriter, Read, Write},
    path::Path,
    process::ExitCode,
};

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            if Path::new(TEMP_DIR).exists() {
                let _ = fs::remove_dir_all(TEMP_DIR);
            }
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("To enter, stream EOF (ctrl+D on Unix, ctrl+Z on Windows)\n");
    let input = read_all!(">>> ")?;

    if !MAIN_RE.is_match(&input) {
        return Err("No valid main function found".into());
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

    compile_and_run!();

    fs::remove_dir_all(TEMP_DIR)?;
    Ok(())
}
