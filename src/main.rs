#[allow(unused_imports)]
use iced::{
    pure::{column, container, image, Element, Sandbox},
    window, Color, Length, Settings,
};
use thiserror::Error;

// ----------------- Iced
#[derive(Default)]
pub struct ImageView {}

#[derive(Debug, Clone, Copy)]
pub struct Message {}

impl Sandbox for ImageView {
    type Message = Message;

    fn new() -> Self {
        ImageView::default()
    }

    fn title(&self) -> String {
        String::from("ImgMg: img.jpg")
    }

    fn update(&mut self, _message: Self::Message) {
        todo!()
    }

    fn view(&self) -> iced::pure::Element<'_, Self::Message> {
        let image: Element<Message> = image("img.jpg").content_fit(iced::ContentFit::Fill).into();

        // let content: Element<_> = column()
        //     .max_width(540)
        //     .spacing(20)
        //     .padding(20)
        //     .push(image)
        //     .into();

        container(image /*content*/)
            // .width(Length::Fill)
            // .height(Length::Fill)
            .into()
    }

    fn background_color(&self) -> Color {
        Color::from([0.5, 0.5, 0.5])
    }
}

#[derive(Debug, Error)]
enum ImgmgError {
    // #[error("Unable to create window.")]
    // WindowError(#[from] OsError),
}

fn main() -> iced::Result {
    println!("Hello you, world!");
    ImageView::run(Settings {
        window: window::Settings {
            size: (200, 200),
            ..Default::default()
        },
        ..Default::default()
    })
}
