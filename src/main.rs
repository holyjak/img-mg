use std::{fmt::Debug, time::Instant};

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
use winit::{dpi::PhysicalSize, event_loop::EventLoop};

// ----------------- Iced
type ImageHandle = iced_native::image::Handle;

#[derive(Clone, Debug)]
pub struct MyImage {
    file_name: String,
    size: (u32, u32),
    handle: ImageHandle,
}
impl MyImage {
    // timing: debug 0.5s, rel 10ms
    fn new(file_name: &str, raw_image: DynamicImage) -> Self {
        // NOTE: image v. 0.24.3 lacks to_bgra8 and Dyn.Image itself has .width(), .height()
        let start = Instant::now();
        let bgra_img = raw_image.to_bgra8();
        let res = MyImage {
            file_name: file_name.to_owned(),
            size: (bgra_img.width(), bgra_img.height()),
            handle: ImageHandle::from_pixels(
                bgra_img.width(),
                bgra_img.height(),
                bgra_img.into_vec(),
            ),
        };
        println!("MyImage.new {:?}", start.elapsed());
        res
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

// timing: debug 1s, rel. 50ms
fn load_image(file: &str) -> anyhow::Result<MyImage> {
    let start = Instant::now();
    let raw_image = ImageReader::open(file.clone())
        .with_context(|| format!("Failed to open image {}", file))?
        .decode()
        .with_context(|| format!("Failed to decode image {}", file))?;
    println!("load_image {:?}", start.elapsed());

    Ok(MyImage::new(file, raw_image))
}

pub struct Flags {
    img: MyImage,
    //monitor_size: (u32, u32),
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
            Message::Loaded(img_result) => {
                self.image = img_result.ok();
                // if let Some(img) = &self.image {
                //     iced::window::resize(img.size.0, img.size.1)
                //     // there is also ::move_to
                // } else {
                //     Command::none()
                // }
            }
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

// type Rectangle = (u32, u32);

// fn maybe_scale(image_size: &Rectangle, monitor_size: &Rectangle) -> Rectangle {
//     let oversize_factor = f32::max(
//         f32::max(image_size.0 as f32 / monitor_size.0 as f32, 1.0),
//         f32::max(image_size.1 as f32 / monitor_size.1 as f32, 1.0),
//     );
//     PhysicalSize::<u32>::from(image_size.clone())
//         .to_logical::<u32>(1.0 / oversize_factor as f64)
//         .into()
// }

fn main() -> anyhow::Result<()> {
    // PhysicalSize { width: 1920, height: 1080 }
    // let monitor_size = EventLoop::new()
    //     .primary_monitor()
    //     .map(|m| m.size())
    //     .unwrap_or((1024, 768).into());
    // FIXME: Find display size. Can't create EventLoop for that b/c there must be 1/app
    let img = load_image("img.jpg")?;
    println!("Hello you, world!");
    ImageView::run(Settings {
        window: window::Settings {
            //size: maybe_scale(&img.size, &monitor_size.into()),
            ..Default::default()
        },
        ..Settings::with_flags(Flags {
            img,
            //monitor_size: (1024, 768), //monitor_size.into(),
        })
    })?;
    Ok(())
}
