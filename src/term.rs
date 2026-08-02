//! RAII guards so the terminal is always restored, even on an early return or
//! a panic partway through an interactive capture loop.

use std::io::{self, Write};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{ExecutableCommand, cursor};

pub struct RawMode;

impl RawMode {
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(RawMode)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

pub struct MouseCapture;

impl MouseCapture {
    pub fn enter() -> io::Result<Self> {
        io::stdout().execute(EnableMouseCapture)?;
        Ok(MouseCapture)
    }
}

impl Drop for MouseCapture {
    fn drop(&mut self) {
        let _ = io::stdout().execute(DisableMouseCapture);
    }
}

/// Read a line of input from the user with a prompt (cooked mode).
///
/// Returns `None` on end of input (Ctrl-D). Callers must treat that as "stop",
/// never as an empty line — otherwise a closed stdin spins forever.
pub fn prompt_line(prompt: &str) -> io::Result<Option<String>> {
    let mut out = io::stdout();
    write!(out, "{prompt}")?;
    out.flush()?;
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        println!();
        return Ok(None);
    }
    Ok(Some(line.trim().to_string()))
}

/// Clear the screen and the scrollback buffer, so a displayed seed phrase
/// cannot be recovered by scrolling up.
pub fn clear_all() -> io::Result<()> {
    let mut out = io::stdout();
    out.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::Purge,
    ))?;
    out.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;
    out.execute(cursor::MoveTo(0, 0))?;
    out.flush()
}
