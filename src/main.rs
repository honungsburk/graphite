use graphite::editor::Editor;
use iced::{Application, Settings};

pub fn main() -> iced::Result {
    let settings = Settings::default();

    Editor::run(settings)
}
