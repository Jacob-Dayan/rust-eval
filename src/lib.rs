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
        if std::path::Path::new($crate::consts::TEMP_DIR).exists() {
            let _ = std::fs::remove_dir_all($crate::consts::TEMP_DIR);
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

#[derive(Default, Clone)]
pub struct RustEvalHelper {
    nav_action: Arc<Mutex<NavAction>>,
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
            Some(Cmd::AcceptLine)
        } else {
            None
        }
    }
}

pub type EvalEditor = Editor<RustEvalHelper, DefaultHistory>;

pub fn create_editor() -> rustyline::Result<EvalEditor> {
    let mut rl = Editor::with_config(Config::builder().auto_add_history(false).build())?;
    let helper = RustEvalHelper::new();
    let act = helper.nav_action.clone();
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
                target,
                at_boundary,
            })),
        );
    }

    Ok(rl)
}

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

                lines[curr_idx] = line;

                match action {
                    NavAction::Submit => {
                        if curr_idx < lines.len() - 1 {
                            let down = lines.len() - 1 - curr_idx;
                            print!("\x1b[{down}B\r");
                            let _ = io::stdout().flush();
                        }
                        return Ok(Some(lines.join("\n")));
                    }
                    NavAction::PrevLine => {
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
                        curr_idx += 1;
                        if curr_idx == lines.len() {
                            lines.push(String::new());
                            cursor_at_end = true;
                        } else {
                            print!("\r\x1b[K");
                            let _ = io::stdout().flush();
                            cursor_at_end = false;
                        }
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
                Ok::<(), std::io::Error>(())
            } else {
                new_io_error!("Program has exited with a non-zero status.")
            }
        } else {
            new_io_error!("Compilation failed.")
        }
    }};
}
