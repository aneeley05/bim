// main.rs
// Entry point

mod document;
mod editor;
mod terminal;

use editor::Editor;
pub use document::Document;
pub use terminal::Terminal;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // If args are passed, open the first one
    if args.len() > 1 {
        let mut editor = Editor::default();
        editor.open_document = Document::from_file(&args[1]);
        editor.run();
    } else {
        println!(
            "{}{}{}",
            termion::color::Fg(termion::color::Red),
            "Please provide a file to open or create.",
            termion::color::Fg(termion::color::Reset)
        );
        return;
    }
}
