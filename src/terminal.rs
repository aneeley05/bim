// terminal.rs
// Handles terminal instance and utils -- updating cursor position, clearing terminal, getting terminal size

use std::io::stdout;
use std::io::Write;

use termion::raw::{IntoRawMode, RawTerminal};

#[derive(Clone, Copy)]
pub struct Position {
    pub x: usize, // X position
    pub y: usize, // Y position
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

pub struct Terminal {
    pub stdout: RawTerminal<std::io::Stdout>, // Raw terminal output
    pub cursor_position: Position, // Cursor position
    pub height: usize, // Terminal height in rows
    pub width: usize, // Terminal width in columns
}

impl Default for Terminal {
    fn default() -> Self {
        Self {
            stdout: stdout().into_raw_mode().unwrap(),
            cursor_position: Position::default(),
            height: termion::terminal_size().unwrap().1 as usize,
            width: termion::terminal_size().unwrap().0 as usize,
        }
    }
}

#[allow(unused_must_use)]
impl Terminal {
    // Sets the cursor visibility
    pub fn set_cursor_visibility(&mut self, visible: bool) {
        if visible == true {
            print!("{}", termion::cursor::Show);
        } else {
            print!("{}", termion::cursor::Hide);
        }
    }

    // Sets the cursor position
    pub fn set_cursor_position(&mut self, position: Position) {
        let x = position.x.saturating_add(1);
        let y = position.y.saturating_add(1);
        write!(self.stdout, "{}", termion::cursor::Goto(x as u16, y as u16));
        self.cursor_position = position;
    }

    // Sets the cursor position to 0, 0 without updaing position field
    pub fn zero_cursor_position(&mut self) {
        write!(self.stdout, "{}", termion::cursor::Goto(1, 1));
    }

    // Clears the terminal
    pub fn clear(&mut self) {
        write!(self.stdout, "{}", termion::clear::All);
    }

    // Returns a copy of the cursor position
    pub fn get_cursor_position(&self) -> Position {
        self.cursor_position.clone()
    }

    // Flushes the terminal
    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.stdout.flush()
    }
}
