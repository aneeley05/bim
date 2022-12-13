// editor.rs
// Handles editor instance and utils -- input, cursor movement, rendering

use std::io;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use crate::{terminal, Document};

pub struct Editor {
    running: bool,                    // Is the editor running?
    options_mode: bool,               // Is the editor in options mode?
    scroll_position: usize,           // How many lines down the document is scrolled
    status_bar: String,               // The status bar text
    pub terminal: terminal::Terminal, // The terminal instance
    pub open_document: Document,      // The open document
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            running: true,
            options_mode: false,
            scroll_position: 0,
            status_bar: format!("ESC to quit."),
            terminal: terminal::Terminal::default(),
            open_document: Document::default(),
        }
    }
}

impl Editor {
    // Starts raw mode then main loop
    pub fn run(&mut self) {
        let _stdout = std::io::stdout().into_raw_mode().unwrap(); // Start raw mode
        loop { // Main loop
            // Set the status bar
            if !self.options_mode { // Options mode
                self.status_bar = format!(
                    "({}/{}) ESC for Options ",
                    self.terminal.get_cursor_position().y + 1 + self.scroll_position,
                    self.open_document.lines.len()
                );
            } else { // Editor mode
                self.status_bar = format!(
                    "[Options] ESC: Back to Editor / a: Save and Exit / s: Save / q: Quit "
                );
            }

            // Draw the editor
            if let Err(error) = self.draw() {
                panic!("{}", error);
            }
            // Check for exit
            if !self.running {
                self.terminal.clear();
                self.terminal.set_cursor_position(terminal::Position::default());
                // Show goodbye in production
                if !cfg!(debug_assertions) {
                    println!("Goodbye.\r");
                }
                break;
            }
            // Process inputs
            if self.options_mode { // Options mode
                if let Err(error) = self.process_options() {
                    panic!("{}", error);
                }
            } else { // Editor mode
                if let Err(error) = self.process_input() {
                    panic!("{}", error);
                }
            }
        }
    }

    // Essentially renders the editor
    pub fn draw(&mut self) -> Result<(), std::io::Error> {
        self.terminal.clear(); // Clear the screen
        self.terminal.set_cursor_visibility(false); // Hide cursor before drawing
        self.terminal.zero_cursor_position(); // Zero out the cursor position

        // Draw the editor
        for mut row_index in 0..self.terminal.height - 1 {
            row_index = row_index + self.scroll_position; // Adjust for scroll position
            // Write line if it exists at row index otherwise draw a tilde
            if self.open_document.lines.len() > row_index {
                println!("{}\r", self.open_document.lines[row_index].replace("\n", ""));
            } else {
                println!("{}{}", "~", "\r");
            }
            // Draw welcome message if editor is empty
            if self.open_document.lines.len() <= 1 && self.open_document.lines[0].len() == 0 {
                if row_index == (self.terminal.height / 2) - 2 { // The adjustment up 2 is arbitrary but it looks good
                    let message = format!("BIM (Bad vIM) - version {}", env!("CARGO_PKG_VERSION")); // Welcome message
                    let mut padding = self.terminal.width - message.len(); // Calculate padding
                    if padding > 0 {
                        padding /= 2; // Divide by 2 to center
                        for _ in 0..padding {
                            print!(" "); // Print padding
                        }
                    }
                    print!("{}\r", message); // Print welcome message
                }
            }
        }
        // Print bottom status bar
        print!("{}{}{}{}",
            termion::color::Bg(termion::color::White),
            self.status_bar, "\r",
            termion::color::Bg(termion::color::Reset));
        
        self.terminal.set_cursor_position(self.terminal.get_cursor_position()); // Undo cursor zeroing
        self.terminal.set_cursor_visibility(true); // Show cursor after drawing
        self.terminal.flush() // Flush the terminal
    }

    // Handles all keystrokes in editor mode
    pub fn process_input(&mut self) -> Result<(), std::io::Error> {
        let key = read_key()?; // Read keystroke
        match key {
            Key::Char('\n') => { // Enter key
                let mut position = self.terminal.get_cursor_position();                                // Current cursor position
                let mut line = self.open_document.lines[position.y + self.scroll_position].clone();    // Current line
                let after_cursor = line.split_off(position.x);                                         // All characters after cursor
                line.truncate(position.x);                                                             // Remove all characters after cursor from current line
                self.open_document.lines[position.y + self.scroll_position] = line;                    // Update current line
                self.open_document.lines.insert(position.y + self.scroll_position + 1, after_cursor);  // Insert new line after current line
                if position.y + 1 > self.terminal.height - 2 { // Attempting to enter past end of screen
                    self.scroll_position += 1; // Scroll down 1
                } else {
                    position.y = position.y.saturating_add(1); // Move cursor down 1
                }
                position.x = 0; // Move cursor to beginning of line
                self.terminal.set_cursor_position(position); // Update cursor position
            }
            Key::Backspace => { // Backspace key
                let mut position = self.terminal.get_cursor_position();                             // Current cursor position
                let mut line = self.open_document.lines[position.y + self.scroll_position].clone(); // Current line
                if position.x > 0 {                                                                 // If cursor is not at beginning of line
                    line.remove(position.x - 1);                                                    // Remove character before cursor
                    position.x = position.x.saturating_sub(1);                                      // Move cursor back 1
                    self.terminal.set_cursor_position(position);                                    // Update cursor position
                    self.open_document.lines[position.y + self.scroll_position] = line;             // Update current line
                } else if position.y > 0 || self.scroll_position > 0 {                              // If cursor is at beginning of line and not at beginning of document
                    let mut prev_line =                                                             // Previous line
                        self.open_document.lines[(position.y + self.scroll_position) - 1].clone();
                    if self.scroll_position > 0 && position.y == 0 {                                // If cursor is at beginning of screen and not at beginning of document
                        self.scroll_position = self.scroll_position.saturating_sub(1);              // Scroll up 1
                    }
                    let prev_line_len = prev_line.len().clone();                                    // Cloned length of previous line (used to set position later)
                    prev_line += &line;                                                             // Append contents of current line to previous line
                    self.open_document.lines.remove(position.y + self.scroll_position);             // Remove current line
                    position.y = position.y.saturating_sub(1);                                      // Move cursor up 1
                    position.x = prev_line_len;                                                     // Move cursor to the cloned length of the line before
                    self.terminal.set_cursor_position(position);                                    // Update cursor position
                    self.open_document.lines[position.y + self.scroll_position] = prev_line;        // Update line
                }
            }
            Key::Char(c) => { // Any "normal" character
                let mut position = self.terminal.get_cursor_position();                              // Current cursor position
                let mut line = self.open_document.lines[position.y + self.scroll_position].clone();  // Clone current line
                line.insert(position.x, c);                                                          // Insert character at cursor position
                self.open_document.lines[position.y + self.scroll_position] = line;                  // Update current line
                position.x = position.x.saturating_add(1);                                           // Move cursor forward 1
                self.terminal.set_cursor_position(position);                                         // Update cursor position
            }
            // Cursor movement keys
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageDown
            | Key::PageUp
            | Key::Home
            | Key::End => self.arrow_move(key),
            Key::Esc => self.options_mode = true, // Enter options mode on ESC
            _ => (), // Ignore all other keys
        }
        Ok(())
    }

    // Handles all keystrokes in options mode
    pub fn process_options(&mut self) -> Result<(), std::io::Error> {
        let key = read_key()?; // Read keystroke
        match key {
            Key::Char('q') => self.running = false, // Exit program on q
            Key::Char('s') => { // Save on s
                self.open_document.save();
                self.options_mode = false;
            }
            Key::Char('a') => { // Save and exit on a
                self.open_document.save();
                self.options_mode = false;
                self.running = false;
            }
            Key::Esc => self.options_mode = false, // Exit options mode on ESC
            _ => (), // Ignore all other keys
        }
        Ok(())
    }

    // Takes a termion key and moves cursor accordingly
    pub fn arrow_move(&mut self, key: Key) {
        let mut position = self.terminal.get_cursor_position();
        match key {
            Key::Up => { // Up arrow
                if position.y > 0 { // If cursor is not at top of screen
                    position.y = position.y.saturating_sub(1); // Move cursor up 1
                } else if position.y == 0 && self.scroll_position > 0 { // If cursor is at top of screen and not at top of document
                    self.scroll_position = self.scroll_position.saturating_sub(1); // Scroll up 1
                }
                if position.x > self.open_document.lines[position.y + self.scroll_position].len() { // If cursor is past end of line after moving
                    position.x = self.open_document.lines[position.y + self.scroll_position].len(); // Move cursor to end of line
                }
            }
            Key::Down => { // Down arrow
                let is_at_end_of_document = (position.y + self.scroll_position + 1) == self.open_document.lines.len(); // If cursor is at end of document
                if !is_at_end_of_document && position.y < self.terminal.height.saturating_sub(2) { // If cursor is not at bottom of screen and not at end of document
                    position.y = position.y.saturating_add(1); // Move cursor down 1
                    if position.x > self.open_document.lines[position.y + self.scroll_position].len() { // If cursor is past end of line after moving
                        position.x = self.open_document.lines[position.y + self.scroll_position].len(); // Move cursor to end of line
                    }
                } else if !is_at_end_of_document && position.y == self.terminal.height.saturating_sub(2) { // If cursor is at bottom of screen and not at end of document
                    self.scroll_position = self.scroll_position.saturating_add(1); // Scroll down 1
                }
            }
            Key::Left => { // Left arrow
                if position.x > 0 { // If cursor is not at beginning of line
                    position.x = position.x.saturating_sub(1); // Move cursor left 1
                }
            }
            Key::Right => { // Right arrow
                if position.x < self.open_document.lines[position.y + self.scroll_position].len() { // If cursor is not at end of line
                    position.x = position.x.saturating_add(1); // Move cursor right 1
                }
            }
            Key::Home => position.x = 0, // Home key moves cursor to beginning of line
            Key::End => position.x = self.open_document.lines[position.y + self.scroll_position].len(), // End key moves cursor to end of line
            _ => (), // Ignore all other keys
        }
        self.terminal.set_cursor_position(position); // Update cursor position
    }
}

// Reads a termion key from stdin
fn read_key() -> Result<Key, std::io::Error> {
    loop {
        if let Some(key) = io::stdin().lock().keys().next() {
            return key;
        }
    }
}
