# 🦀 `rust_eval`

**`rust_eval`** is a lightweight Rust CLI tool that reads Rust source code interactively from standard input (`stdin`), validates it, dynamically compiles it using `rustc`, and executes it on the fly in an `Eval`-style workflow.

---

## Features

* **Interactive Input:** Stream Rust scripts directly from `stdin` until an `EOF` signal is received.
* **Regex Validation:** Uses pre-compiled regex matching to verify the presence of a valid `fn main` entry point before attempting compilation.
* **Cross-Platform Support:** Automatically handles execution targets and temp paths for Unix (`Linux`/`macOS`) and `Windows` (`.exe`).
* **Isolated Build Environment:** Automatically manages a temporary directory (`tmp/`) for source generation and compilation, ensuring thorough cleanup even on error.
* **Detailed Execution Output:** Displays structured reports for `STDOUT`, `STDERR`, and the process `Exit Code`.

---

### How It Works
1. **Input Reading:** Displays a `>>> ` prompt and streams user code until `EOF` (`Ctrl+D` on Unix/macOS, `Ctrl+Z` on Windows) is signaled.
2. **Validation:** Evaluates input against a `LazyLock<Regex>` pattern to ensure a valid `fn main(...)` function signature exists.
3. **Source Preparation:** Creates the `tmp/` workspace and outputs `tmp/tmp.rs`, wrapped with auto-generated headers and footers.
4. **Compilation & Execution:** Invokes `rustc tmp/tmp.rs --out-dir tmp`. On successful compilation, executes the generated binary (`tmp/tmp` or `tmp/tmp.exe`) and captures standard output and error streams.
5. **Cleanup:** Cleans up the temporary workspace upon successful execution or runtime failures.

---

##  Getting Started

### Prerequisites

* **Rust** toolchain installed with `rustc` available in your system `PATH`.

### Running locally

1. Clone the repository:
   ```bash
   git clone https://github.com/Jacob-Dayan/rust_eval.git
   cd rust_eval
   ```

2. Run using `cargo`:
   ```bash
   cargo run
   ```

---

##  Example Usage

Upon starting the binary:

```text
To enter, stream EOF (ctrl+D on Unix, ctrl+Z on Windows)

>>> fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();
    println!("The sum is: {}", sum);
}
```

Press `Ctrl+D` (or `Ctrl+Z` on Windows) to trigger evaluation:

```text
===STDOUT===
The sum is: 15
===STDERR===
===EXIT CODE===
exit status: 0
```

And if you just wanna run a simple code snippet, you don't even have to mess with `main` function.

the code from below, works exactly the same as the one above:

```text
To enter, stream EOF (ctrl+D on Unix, ctrl+Z on Windows)

>>> let numbers = vec![1, 2, 3, 4, 5];
    let sum: i32 = numbers.iter().sum();
    println!("The sum is: {}", sum);
```
---

##  Project Structure

```text
.
├── src/
│   ├── lib.rs      # Exported macros and modules
│   ├── consts.rs   # Path constants, Regex matchers, Header & Footer wrappers
│   └── main.rs     # CLI entry point, input handling, process control, and cleanup
└── Cargo.toml
```

---

## License

This project is licensed under the MIT License.
