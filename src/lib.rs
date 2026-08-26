pub use rustyline::{
    self, completion::Completer, error::ReadlineError, highlight::Highlighter, hint::Hinter,
    history::DefaultHistory, validate::Validator, Cmd, ConditionalEventHandler, Config,
    DefaultEditor, Editor, Event, EventContext, EventHandler, Helper, KeyCode, KeyEvent, Modifiers,
    RepeatCount,
};
pub use std::io::{self, BufWriter, Read, Write};
use std::sync::{Arc, Mutex};
pub mod consts;

#[macro_export]
macro_rules! new_io_error {
    ($e:expr) => {
        Err(std::io::Error::new(std::io::ErrorKind::Other, $e))
    };
}

/// Navigation action triggered by navigation keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavAction {
    #[default]
    Enter,
    PrevLine,
    NextLine,
    Submit,
}

/// Custom helper for `rust-eval`.
#[derive(Default, Clone)]
pub struct RustEvalHelper {
    nav_action: Arc<Mutex<NavAction>>,
}

impl RustEvalHelper {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_nav_action(&self, action: NavAction) {
        if let Ok(mut lock) = self.nav_action.lock() {
            *lock = action;
        }
    }

    #[must_use]
    pub fn get_nav_action(&self) -> NavAction {
        self.nav_action
            .lock()
            .map(|a| *a)
            .unwrap_or(NavAction::Enter)
    }
}

impl Completer for RustEvalHelper {
    type Candidate = String;
}
impl Hinter for RustEvalHelper {
    type Hint = String;
}
impl Highlighter for RustEvalHelper {}
impl Validator for RustEvalHelper {}
impl Helper for RustEvalHelper {}

struct KeyNav(Arc<Mutex<NavAction>>, NavAction, bool);

impl ConditionalEventHandler for KeyNav {
    fn handle(&self, _evt: &Event, _n: RepeatCount, _pos: bool, ctx: &EventContext) -> Option<Cmd> {
        let trigger = match self.1 {
            NavAction::PrevLine if self.2 => ctx.pos() == 0,
            NavAction::NextLine if self.2 => ctx.pos() == ctx.line().len(),
            _ => true,
        };
        if trigger {
            if let Ok(mut lock) = self.0.lock() {
                *lock = self.1;
            }
            Some(Cmd::AcceptLine)
        } else {
            None
        }
    }
}

/// Type alias for the configured `rust-eval` editor.
pub type EvalEditor = Editor<RustEvalHelper, DefaultHistory>;

/// Creates and configures a new [`EvalEditor`] with multi-line editing and keybindings.
///
/// # Errors
/// Returns [`rustyline::error::ReadlineError`] if editor initialization fails.
pub fn create_editor() -> rustyline::Result<EvalEditor> {
    let mut rl = Editor::with_config(Config::builder().auto_add_history(false).build())?;
    let helper = RustEvalHelper::new();
    let act = helper.nav_action.clone();
    rl.set_helper(Some(helper));

    let mut bind = |code, mods, nav, check| {
        rl.bind_sequence(
            KeyEvent(code, mods),
            EventHandler::Conditional(Box::new(KeyNav(act.clone(), nav, check))),
        );
    };

    bind(KeyCode::Left, Modifiers::NONE, NavAction::PrevLine, true);
    bind(KeyCode::Backspace, Modifiers::NONE, NavAction::PrevLine, true);
    bind(KeyCode::Right, Modifiers::NONE, NavAction::NextLine, true);
    bind(KeyCode::Up, Modifiers::NONE, NavAction::PrevLine, false);
    bind(KeyCode::Down, Modifiers::NONE, NavAction::NextLine, false);
    bind(KeyCode::Char('d'), Modifiers::CTRL, NavAction::Submit, false);
    bind(KeyCode::Char('z'), Modifiers::CTRL, NavAction::Submit, false);

    Ok(rl)
}

/// Reads multi-line input from [`EvalEditor`].
///
/// Displays purple `>>> ` on the first line and purple `... ` on all subsequent lines.
///
/// # Errors
/// Returns an [`std::io::Error`] if reading from the editor fails with an unexpected error.
pub fn read_input(rl: &mut EvalEditor) -> io::Result<Option<String>> {
    let mut lines: Vec<String> = vec![String::new()];
    let mut curr_idx = 0;
    let mut cursor_at_end = true;

    loop {
        let prompt = if curr_idx == 0 {
            format!("{PURPLE}>>> {RESET}", PURPLE = consts::PURPLE, RESET = consts::RESET)
        } else {
            format!("{PURPLE}... {RESET}", PURPLE = consts::PURPLE, RESET = consts::RESET)
        };

        if let Some(h) = rl.helper_mut() {
            h.set_nav_action(NavAction::Enter);
        }

        let current_text = &lines[curr_idx];
        let initial = if cursor_at_end {
            (current_text.as_str(), "")
        } else {
            ("", current_text.as_str())
        };

        let result = rl.readline_with_initial(&prompt, initial);
        let action = rl.helper().map(RustEvalHelper::get_nav_action).unwrap_or(NavAction::Enter);

        match result {
            Ok(line) => {
                if curr_idx == 0 && lines.len() == 1 {
                    let trimmed = line.trim();
                    if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
                        return Ok(None);
                    }
                    if trimmed.eq_ignore_ascii_case("clear") {
                        #[cfg(target_family = "unix")]
                        let _ = std::process::Command::new("clear").status();
                        #[cfg(target_family = "windows")]
                        let _ = std::process::Command::new("cmd").args(["/C", "cls"]).status();

                        lines = vec![String::new()];
                        cursor_at_end = true;
                        continue;
                    }
                }

                lines[curr_idx] = line;

                match action {
                    NavAction::Submit => return Ok(Some(lines.join("\n"))),
                    NavAction::PrevLine if curr_idx > 0 => {
                        print!("\x1b[1A\r\x1b[K");
                        let _ = io::stdout().flush();
                        curr_idx -= 1;
                        cursor_at_end = true;
                    }
                    NavAction::NextLine if curr_idx + 1 < lines.len() => {
                        curr_idx += 1;
                        cursor_at_end = false;
                    }
                    NavAction::Enter => {
                        if curr_idx + 1 < lines.len() {
                            curr_idx += 1;
                            cursor_at_end = false;
                        } else {
                            lines.push(String::new());
                            curr_idx += 1;
                            cursor_at_end = true;
                        }
                    }
                    _ => {}
                }
            }
            Err(ReadlineError::Eof) => {
                return Ok(if lines.len() == 1 && lines[0].trim().is_empty() {
                    None
                } else {
                    Some(lines.join("\n"))
                });
            }
            Err(ReadlineError::Interrupted) => return Ok(None),
            Err(err) => return Err(std::io::Error::other(format!("Readline error: {err}"))),
        }
    }
}

/// Reads multi-line input using [`read_input`].
#[macro_export]
macro_rules! read_all {
    ($rl:expr) => {
        $crate::read_input($rl)
    };
}

/// Calls `rustc` to compile the code, then runs the compiled binary with `stdin`, `stdout`, and `stderr` redirected.
///
/// # Errors
/// If successful returns [`Result::Ok`], otherwise returns a new [`std::io::Error`].
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
