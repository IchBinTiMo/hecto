use editor::Editor;

mod editor;
mod prelude;

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
    Editor::new().unwrap().run();
}
