use iced::{
    pure::{column, container, image, Element, Sandbox},
    Settings,
};
use thiserror::Error;

// ----------------- Iced
pub struct ImageView {}

#[derive(Debug, Clone, Copy)]
pub struct Message {}

impl Sandbox for ImageView {
    type Message = Message;

    fn new() -> Self {
        ImageView {}
    }

    fn title(&self) -> String {
        String::from("ImageView 123")
    }

    fn update(&mut self, _message: Self::Message) {
        todo!()
    }

    fn view(&self) -> iced::pure::Element<'_, Self::Message> {
        let image: Element<Message> = image("img.jpg").into();

        let content: Element<_> = column()
            .max_width(540)
            .spacing(20)
            .padding(20)
            .push(image)
            .into();

        container(content).into()
    }
}

#[derive(Debug, Error)]
enum ImgmgError {
    // #[error("Unable to create window.")]
    // WindowError(#[from] OsError),
}

fn main() -> iced::Result {
    println!("Hello you, world!");
    ImageView::run(Settings::default())
}
