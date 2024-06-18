mod editor;

use editor::Editor;

#[warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::restriction,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]
fn main() {
    // let editor: Editor = Editor::default();
    // editor.run(); // same as Editor::run(&editor);

    Editor::new().unwrap().run();
}
