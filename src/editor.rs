use iced::{widget::text, Element, Sandbox};

pub struct Editor;

#[derive(Debug, Clone, Copy)]
pub enum Message {}

impl Sandbox for Editor {
    type Message = Message;

    fn new() -> Self {
        Self
    }

    fn title(&self) -> String {
        String::from("Graphite")
    }

    fn update(&mut self, message: Message) {
        match message {}
    }

    fn view(&self) -> Element<Message> {
        text("Hello, Graphite!").size(50).into()
    }
}
