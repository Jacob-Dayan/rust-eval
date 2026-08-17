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
                // The `let _ =` is to suppress the error if the directory doesn't exist,
                // which only would happen in a very weird situation
                // where both the directory was deleted, but there was an error
                // that should have been cutting the program off before it was able to clean up
                let _ = fs::remove_dir_all(TEMP_DIR);
            }
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("To enter, stream EOF (ctrl+D on Unix, ctrl+Z on Windows)\n");
    let mut input = read_all!(">>> ")?;

    if !MAIN_RE.is_match(&input) {
        // If no main function is found, but there are function definitions,
        // then we can't wrap the code in a main function
        // then we have to return an error.
        if input.contains("fn ") {
            return Err("No valid main function found".into());
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
    // So if trying to access stdin, it doesn't look like the EOF is not recognized
    print!("\n\n");
    compile_and_run!()?;

    fs::remove_dir_all(TEMP_DIR)?;
    Ok(())
}
