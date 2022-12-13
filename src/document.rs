// document.rs
// Handles document instance and utils -- importing a file to a Document, saving a Document to a file

use std::io::Write;

pub struct Document {
    pub lines: Vec<String>, // Lines of text
    pub path: String,       // Path to file
}

impl Document {
    pub fn default() -> Self {
        Self {
            lines: vec!["".to_string()], // There must be at least one line
            path: "".to_string(),
        }
    }

    // Import file to Document
    pub fn from_file(path: &str) -> Self {
        let mut lines = vec![]; // Lines of text
        // If file already exists, read it
        if std::path::Path::new(path).exists() {
            let file = std::fs::read_to_string(path).expect(&format!("Could not read file {}", path));
            for line in file.lines() { // Iterate over lines
                lines.push(line.to_string()); // Add file line to lines vector
            }
        }
        if lines.len() == 0 { // Make sure lines vector is not empty
            lines.push("".to_string());
        }
        Self {
            lines,
            path: path.to_string(),
        }
    }

    // Save open document to file
    pub fn save(&self) {
        let mut output_file = std::fs::File::create(self.path.clone()).expect("Could not create file"); // Create/Open file

        let mut first_line_written = false; // Used to determine if a newline should be written
        for line in self.lines.clone() { // Iterate over lines clone
            if first_line_written { // If first line has been written, write a newline
                output_file.write_all("\n".as_bytes()).expect("Could not write to file");
            }
            output_file.write_all(line.as_bytes()).expect("Could not write to file"); // Write line to file
            first_line_written = true; // Set first line written to true (doesn't matter if it was already true)
        }
    }
}
