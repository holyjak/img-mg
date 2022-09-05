use fltk::{image::SharedImage, prelude::*, *};

fn main() -> anyhow::Result<()> {
    let a = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut win = window::Window::default().with_size(640, 480);
    let mut col = group::Flex::default_fill().column();
    main_panel(&mut col)?;
    col.end();
    win.resizable(&col);
    win.set_color(enums::Color::from_rgb(250, 250, 250));
    win.end();
    win.show();
    win.size_range(600, 400, 0, 0);
    a.run().unwrap();
    Ok(())
}

fn main_panel(parent: &mut group::Flex) -> anyhow::Result<()> {
    frame::Frame::default();

    let mut mp = group::Flex::default().row();
    middle_panel(&mut mp)?;
    mp.end();

    frame::Frame::default();

    parent.set_size(&mp, 200);

    Ok(())
}

fn middle_panel(parent: &mut group::Flex) -> anyhow::Result<()> {
    frame::Frame::default();

    let mut frame = frame::Frame::default().with_label("Image");
    frame.set_frame(enums::FrameType::BorderBox);
    frame.set_color(enums::Color::from_rgb(0, 200, 0));
    let spacer = frame::Frame::default();

    let mut image = SharedImage::load("img.jpg")?;
    image.scale(190, 190, true, true);

    frame.set_image(Some(image));

    frame::Frame::default();

    parent.set_size(&frame, 220);
    parent.set_size(&spacer, 10);

    Ok(())
}
