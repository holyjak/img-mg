use std::fmt::Debug;

use anyhow::{self, Context};
use iced::{
    alignment::{Horizontal, Vertical},
    executor,
    pure::Application,
    Command,
};
#[allow(unused_imports)]
use iced::{
    pure::{column, container, image, text, Element, Sandbox},
    window, Color, Length, Settings,
};
use iced_futures::futures::future;
use iced_native;
use image::{io::Reader as ImageReader, DynamicImage};
use thiserror::Error;

// ----------------- Iced
type ImageHandle = iced_native::image::Handle;

#[derive(Clone, Debug)]
pub struct MyImage {
    file_name: String,
    size: (u32, u32),
    handle: ImageHandle,
}
impl MyImage {
    fn new(file_name: &str, raw_image: DynamicImage) -> Self {
        // NOTE: image v. 0.24.3 lacks to_bgra8 and Dyn.Image itself has .width(), .height()
        let bgra_img = raw_image.to_bgra8();
        MyImage {
            file_name: file_name.to_owned(),
            size: (bgra_img.width(), bgra_img.height()),
            handle: ImageHandle::from_pixels(
                bgra_img.width(),
                bgra_img.height(),
                bgra_img.into_vec(),
            ),
        }
    }
}
#[derive(Default)]
pub struct ImageView {
    image: Option<MyImage>,
}

#[derive(Clone)]
pub enum Message {
    Loaded(Result<MyImage, ImgMgError>), // We never expect an error but Command::perform forces a Result onto us
}
impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loaded(_) => f.write_str("Loaded"),
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum ImgMgError {}

fn load_image(file: &str) -> anyhow::Result<MyImage> {
    let raw_image = ImageReader::open(file.clone())
        .with_context(|| format!("Failed to open image {}", file))?
        .decode()
        .with_context(|| format!("Failed to decode image {}", file))?;

    Ok(MyImage::new(file, raw_image))
}

pub struct Flags {
    img: MyImage,
}

impl Application for ImageView {
    type Message = Message;

    fn title(&self) -> String {
        String::from(format!(
            "My Rust Image Manager: {}",
            self.image
                .as_ref()
                .map_or("N/A".to_owned(), |i| i.file_name.to_owned())
        ))
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match message {
            Message::Loaded(img_result) => self.image = img_result.ok(),
        };
        Command::none()
    }

    fn view(&self) -> iced::pure::Element<'_, Self::Message> {
        println!("View rendering... Has image: {}", self.image.is_some());
        let content: Element<Message> = if let Some(img) = &self.image {
            image(img.handle.clone())
                .width(Length::Fill)
                // .center_x() - N/A ?!
                //.content_fit(iced::ContentFit::Fill)
                .into()

            // let content: Element<_> = column()
            //     .max_width(540)
            //     .spacing(20)
            //     .padding(20)
            //     .push(image)
            //     .into();
        } else {
            text("Image not loaded yet / failed")
                // .horizontal_alignment(Horizontal::Center) // has no effect ?!
                // .vertical_alignment(Vertical::Center)
                .size(30)
                .color([1., 0., 0.])
                .into()
        };

        container(content)
            // .width(Length::Fill)
            // .height(Length::Fill)
            .into()
    }

    fn background_color(&self) -> Color {
        Color::from([0.5, 0.5, 0.5])
    }

    type Executor = executor::Default;

    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            ImageView::default(),
            Command::perform(future::ok(flags.img), Message::Loaded),
        )
    }
}

#[derive(Debug, Error)]
enum ImgmgError {
    // #[error("Unable to create window.")]
    // WindowError(#[from] OsError),
}

fn main() -> anyhow::Result<()> {
    let img = load_image("img.jpg")?;
    println!("Hello you, world!");
    ImageView::run(Settings {
        window: window::Settings {
            size: img.size,
            ..Default::default()
        },
        ..Settings::with_flags(Flags { img })
    })?;
    Ok(())
}
