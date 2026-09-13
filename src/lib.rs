pub mod consts;
pub mod prelude;

use std::sync::{Arc, Mutex};

pub use crate as rs_eval;
use crate::prelude::*;

#[macro_export]
macro_rules! new_io_error {
    ($e:expr) => {
        Err(std::io::Error::new(std::io::ErrorKind::Other, $e))
    };
}

#[macro_export]
macro_rules! clean_temp_dir {
    () => {
        if $crate::consts::TEMP_DIR.exists() {
            let _ = std::fs::remove_dir_all(&*$crate::consts::TEMP_DIR);
        }
    };
}

#[macro_export]
macro_rules! clear_screen {
    () => {{
        #[cfg(target_family = "unix")]
        let _ = std::process::Command::new("clear").status();
        #[cfg(target_family = "windows")]
        let _ = std::process::Command::new("cmd")
            .args(["/C", "cls"])
            .status();
    }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavAction {
    #[default]
    Enter,
    PrevLine,
    NextLine,
    Submit,
}

/// Helper passed to Rustyline to bridge custom navigation key events
/// to multi-line input state.
#[derive(Default, Clone)]
pub struct RustEvalHelper {
    nav_action: Arc<Mutex<NavAction>>,
    split_pos: Arc<Mutex<Option<usize>>>,
}

pub type RsEvalHelper = RustEvalHelper;

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

    pub fn set_split_pos(&self, pos: Option<usize>) {
        if let Ok(mut lock) = self.split_pos.lock() {
            *lock = pos;
        }
    }

    #[must_use]
    pub fn get_split_pos(&self) -> Option<usize> {
        self.split_pos.lock().ok().and_then(|p| *p)
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

struct KeyNav {
    action: Arc<Mutex<NavAction>>,
    split_pos: Arc<Mutex<Option<usize>>>,
    target: NavAction,
    at_boundary: bool,
}

impl ConditionalEventHandler for KeyNav {
    fn handle(&self, _evt: &Event, _n: RepeatCount, _pos: bool, ctx: &EventContext) -> Option<Cmd> {
        let trigger = match self.target {
            NavAction::PrevLine if self.at_boundary => ctx.pos() == 0,
            NavAction::NextLine if self.at_boundary => ctx.pos() == ctx.line().len(),
            _ => true,
        };
        if trigger {
            if let Ok(mut lock) = self.action.lock() {
                *lock = self.target;
            }
            if self.target == NavAction::Enter
                && let Ok(mut lock) = self.split_pos.lock()
            {
                *lock = Some(ctx.pos());
            }
            Some(Cmd::AcceptLine)
        } else {
            None
        }
    }
}

pub type EvalEditor = Editor<RustEvalHelper, DefaultHistory>;

/// Creates an editor configured with keybindings for multi-line navigation:
/// - Left / Backspace at col 0 moves to previous line.
/// - Right at line end moves to next line.
/// - Up / Down moves vertically across lines.
/// - Enter splits line at cursor or creates next line.
/// - Ctrl+D / Ctrl+Z triggers evaluation.
pub fn create_editor() -> rustyline::Result<EvalEditor> {
    let mut rl = Editor::with_config(Config::builder().auto_add_history(false).build())?;
    let helper = RustEvalHelper::new();
    let act = helper.nav_action.clone();
    let split = helper.split_pos.clone();
    rl.set_helper(Some(helper));

    let bindings = [
        (KeyCode::Left, Modifiers::NONE, NavAction::PrevLine, true),
        (
            KeyCode::Backspace,
            Modifiers::NONE,
            NavAction::PrevLine,
            true,
        ),
        (KeyCode::Right, Modifiers::NONE, NavAction::NextLine, true),
        (KeyCode::Up, Modifiers::NONE, NavAction::PrevLine, false),
        (KeyCode::Down, Modifiers::NONE, NavAction::NextLine, false),
        (KeyCode::Enter, Modifiers::NONE, NavAction::Enter, false),
        (
            KeyCode::Char('d'),
            Modifiers::CTRL,
            NavAction::Submit,
            false,
        ),
        (
            KeyCode::Char('z'),
            Modifiers::CTRL,
            NavAction::Submit,
            false,
        ),
    ];

    for (code, mods, target, at_boundary) in bindings {
        rl.bind_sequence(
            KeyEvent(code, mods),
            EventHandler::Conditional(Box::new(KeyNav {
                action: act.clone(),
                split_pos: split.clone(),
                target,
                at_boundary,
            })),
        );
    }

    Ok(rl)
}

/// Reads multi-line input from stdin until EOF (Ctrl+D/Ctrl+Z) or exit/quit.
///
/// Rustyline is strictly single-line, so line transitions are simulated by
/// manually redrawing and moving the cursor with raw ANSI escape sequences.
pub fn read_input(rl: &mut EvalEditor) -> io::Result<Option<String>> {
    let mut lines: Vec<String> = vec![String::new()];
    let mut curr_idx = 0;
    let mut cursor_at_end = true;

    loop {
        let prompt = if curr_idx == 0 {
            consts::PROMPT_MAIN
        } else {
            consts::PROMPT_CONT
        };

        if let Some(h) = rl.helper_mut() {
            h.set_nav_action(NavAction::Enter);
            h.set_split_pos(None);
        }

        let current_text = &lines[curr_idx];
        let initial = if cursor_at_end {
            (current_text.as_str(), "")
        } else {
            ("", current_text.as_str())
        };

        let result = rl.readline_with_initial(prompt, initial);
        let action = rl
            .helper()
            .map(RustEvalHelper::get_nav_action)
            .unwrap_or(NavAction::Enter);

        match result {
            Ok(line) => {
                if curr_idx == 0 && lines.len() == 1 {
                    let trimmed = line.trim();
                    if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit")
                    {
                        return Ok(None);
                    }
                    if trimmed.eq_ignore_ascii_case("clear") {
                        clear_screen!();
                        lines = vec![String::new()];
                        cursor_at_end = true;
                        continue;
                    }
                }

                match action {
                    NavAction::Submit => {
                        lines[curr_idx] = line;
                        if curr_idx < lines.len() - 1 {
                            let down = lines.len() - 1 - curr_idx;
                            print!("\x1b[{down}B\r");
                            let _ = io::stdout().flush();
                        }
                        return Ok(Some(lines.join("\n")));
                    }
                    NavAction::PrevLine => {
                        lines[curr_idx] = line;
                        if curr_idx > 0 {
                            if lines.len() > 1
                                && curr_idx == lines.len() - 1
                                && lines[curr_idx].is_empty()
                            {
                                lines.pop();
                                print!("\x1b[1A\r\x1b[K\x1b[1A\r\x1b[K");
                            } else {
                                print!("\x1b[2A\r\x1b[K");
                            }
                            let _ = io::stdout().flush();
                            curr_idx -= 1;
                            cursor_at_end = true;
                        } else {
                            print!("\x1b[1A\r\x1b[K");
                            let _ = io::stdout().flush();
                            cursor_at_end = false;
                        }
                    }
                    NavAction::NextLine => {
                        lines[curr_idx] = line;
                        if curr_idx + 1 < lines.len() {
                            print!("\r\x1b[K");
                            let _ = io::stdout().flush();
                            curr_idx += 1;
                            cursor_at_end = false;
                        } else {
                            print!("\x1b[1A\r\x1b[K");
                            let _ = io::stdout().flush();
                            cursor_at_end = true;
                        }
                    }
                    NavAction::Enter => {
                        let split_pos = rl
                            .helper()
                            .and_then(RustEvalHelper::get_split_pos)
                            .unwrap_or(line.len())
                            .min(line.len());

                        let split_pos = if line.is_char_boundary(split_pos) {
                            split_pos
                        } else {
                            line.floor_char_boundary(split_pos)
                        };

                        let tail = line[split_pos..].to_string();
                        let line_len = line.len();
                        let mut head = line;
                        head.truncate(split_pos);

                        let prev_idx = curr_idx;
                        lines[prev_idx] = head;
                        lines.insert(prev_idx + 1, tail);
                        curr_idx += 1;

                        let prev_prompt = if prev_idx == 0 {
                            consts::PROMPT_MAIN
                        } else {
                            consts::PROMPT_CONT
                        };

                        if split_pos < line_len {
                            print!("\x1b[1A\r\x1b[K{}{}\r\n", prev_prompt, lines[prev_idx]);
                        }

                        if curr_idx < lines.len() - 1 {
                            for line in &lines[curr_idx..] {
                                print!("\r\x1b[K{}{}\r\n", consts::PROMPT_CONT, line);
                            }
                            let lines_to_go_up = lines.len() - curr_idx;
                            print!("\x1b[{lines_to_go_up}A\r\x1b[K");
                        } else {
                            print!("\r\x1b[K");
                        }
                        let _ = io::stdout().flush();

                        cursor_at_end = split_pos == line_len;
                    }
                }
            }
            Err(ReadlineError::Eof) => {
                if curr_idx < lines.len() - 1 {
                    let down = lines.len() - 1 - curr_idx;
                    print!("\x1b[{down}B\r\n");
                } else {
                    print!("\r\n");
                }
                let _ = io::stdout().flush();

                return Ok(if lines.iter().all(|l| l.trim().is_empty()) {
                    None
                } else {
                    Some(lines.join("\n"))
                });
            }
            Err(ReadlineError::Interrupted) => {
                if curr_idx < lines.len() - 1 {
                    let down = lines.len() - 1 - curr_idx;
                    print!("\x1b[{down}B\r\n");
                    let _ = io::stdout().flush();
                }
                return Ok(None);
            }
            Err(err) => return Err(std::io::Error::other(format!("Readline error: {err}"))),
        }
    }
}

#[macro_export]
macro_rules! read_all {
    ($rl:expr) => {
        $crate::read_input($rl)
    };
}

/// Compiles `tmp.rs` using raw `rustc` and runs the resulting binary.
///
/// Only the standard library is available (no external crate dependencies).
#[macro_export]
macro_rules! compile_and_run {
    () => {
        $crate::compile_and_run!(std::process::Stdio::inherit())
    };
    ($rustc_stderr:expr) => {{
        let compile_status = std::process::Command::new("rustc")
            .arg(&*$crate::consts::CODE_FILE)
            .arg("--out-dir")
            .arg(&*$crate::consts::TEMP_DIR)
            .stderr($rustc_stderr)
            .status()?;

        if compile_status.success() {
            let status = std::process::Command::new(&*$crate::consts::EXEC_FILE)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()?;

            if status.success() {
                Ok::<(), std::io::Error>(())
            } else {
                new_io_error!("Program has exited with a non-zero status.")
            }
        } else {
            new_io_error!("Compilation failed.")
        }
    }};
}
