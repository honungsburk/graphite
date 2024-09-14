use graphite::editor::Editor;
use iced::{Application, Settings};

pub fn main() -> iced::Result {
    Editor::run(Settings::default())
}
