use iced::{executor, widget::text, Application, Command, Element, Theme};

pub struct Editor;

#[derive(Debug, Clone, Copy)]
pub enum Message {}

impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (Self, Command::none())
    }

    fn title(&self) -> String {
        String::from("Graphite")
    }

    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        text("Hello, Graphite!").size(50).into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}
