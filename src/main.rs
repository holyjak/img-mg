use anyhow::Context;
use fltk::{image::SharedImage, prelude::*, *};
use std::{
    fs,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{Arc, Mutex},
};
extern crate itertools;
use itertools::Itertools;

fn dir_images(dir_path: &str) -> anyhow::Result<Vec<PathBuf>> {
    let paths: Vec<PathBuf> = fs::read_dir(dir_path)?
        // TODO Handle DirEntry that we fail to read (currently we just ignore it)
        .filter_map(|entry_res| entry_res.ok().map(|e| e.path()))
        .into_iter()
        .filter(|f| f.extension().map(|ext| ext == "jpg").unwrap_or(false)) // FIXME Support all supported extensions
        .collect();
    //.ok()
    Ok(paths)
}

struct State {
    thumb_size: i32,
    per_row: i32,
    image_paths: Option<Vec<PathBuf>>,
    row_height: i32,
    total_height: i32,
    visible_rows: (i32, i32),
    img_frames: Vec<Arc<Mutex<frame::Frame>>>,
}

fn add_image(
    state: &State,
    parent: &mut group::Flex,
    image_path: &PathBuf,
    is_visible: bool,
) -> anyhow::Result<frame::Frame> {
    let mut frame = frame::Frame::default();
    //parent.set_size(&mut frame, state.thumb_size + 100); // sets width b/c parent is row
    frame.set_frame(enums::FrameType::FlatBox);
    //frame.set_align(enums::Align::Wrap); // should wrap label but has 0 effect? Perhaps b/c no spaces in it???
    frame.set_align(enums::Align::Clip);
    frame.set_color(enums::Color::White);

    if is_visible {
        set_image(state, &mut frame, image_path)?;
    } else {
        frame.set_label("@refresh");
        frame.set_label_size(50);
    }

    Ok(frame)
}

fn set_image(state: &State, frame: &mut frame::Frame, image_path: &PathBuf) -> anyhow::Result<()> {
    let fname = image_path.file_name().unwrap().to_string_lossy();
    //let fname_no_ext = image_path.file_prefix().unwrap().to_string_lossy(); // Unstable on rustc 1.63.0, 2022-09
    let fpath = image_path.to_string_lossy();

    // TODO Label: hide ext. to save space
    frame.set_label(fname.deref());
    frame.set_label_size(14);
    frame.set_tooltip(fname.deref());
    let mut image = SharedImage::load(fpath.deref()).with_context({
        let f = fpath.deref().to_owned();
        || f
    })?;
    image.scale(state.thumb_size, state.thumb_size, true, true); // TODO Rescale when window expands?

    frame.set_image(Some(image)); // This shows no image: frame.set_image_scaled(Some(image));

    Ok(())
}

fn add_img_rows(parent: &mut group::Flex, state: &mut State) -> anyhow::Result<()> {
    if state.image_paths.is_none() {
        return Ok(());
    }

    for (row_nr, chunk) in state
        .image_paths
        .as_ref()
        .unwrap()
        .iter()
        .take(15) // FIXME
        .chunks(state.per_row as usize)
        .into_iter()
        .enumerate()
    {
        //group::Flex::debug(true);
        let nr = row_nr as i32;
        let is_visible = nr >= state.visible_rows.0 && nr <= state.visible_rows.1;
        // println!(
        //     "Rendering row nr {} vis: {} <> in {:?}",
        //     row_nr, is_visible, state.visible_rows
        // );
        let mut row = group::Flex::default().row();
        parent.set_size(&mut row, state.row_height);
        //parent.resizable(&row);
        for image_path in chunk {
            let f = add_image(state, &mut row, image_path, is_visible)?;
            state.img_frames.push(Arc::new(Mutex::new(f)));
        }
        row.end();
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let win_width = 640;
    let win_height = 480;
    let thumb_size = 200;
    let thumb_margin = 10;
    let thumb_container_size = thumb_size + thumb_margin;
    let per_row = win_width / thumb_container_size; // TODO Include gaps, decorations, ...
    let row_height = thumb_container_size + 10; // some extra space, just in case...

    let image_paths = dir_images("./Pictures/mobil/2022/08").ok();
    let img_cnt = image_paths.as_ref().map_or(0, |v| v.len()) as i32;

    let nr_rows = (img_cnt + (per_row - 1)) / per_row; // rounded up

    let mut state = State {
        image_paths,
        per_row,
        row_height,
        total_height: nr_rows * row_height,
        //thumb_margin,
        thumb_size,
        visible_rows: (0, (win_height + (row_height - 1)) / row_height), // Update as we scroll...
        img_frames: vec![],
    };

    let a = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut win = window::Window::default().with_size(win_width, win_height);

    let mut scroll = group::Scroll::default_fill(); //new(0, 0, win_width, win_height, None);
                                                    // .with_pos(0, 0)
                                                    // .with_size(win_width, win_height);
    scroll.set_type(group::ScrollType::Vertical);
    // NOTE We must manually set col.height to > win_h for scrollbar to appear
    let mut col = group::Flex::new(0, 0, win_width, state.total_height, None).column();
    add_img_rows(&mut col, &mut state)?;
    col.end();
    scroll.end();

    win.resizable(&col); // make the window resizable
    win.set_color(enums::Color::from_rgb(250, 250, 250));
    win.end();
    win.show();
    win.size_range(600, 400, 0, 0);

    // FIXME rm DEMO code - ex. of mutating a displayed image
    state
        .img_frames
        .last()
        .unwrap()
        .lock()
        .and_then(|mut g| {
            let path = PathBuf::from("img.jpg");
            let mut frame: &mut frame::Frame = g.deref_mut();
            set_image(&state, frame, &path).unwrap();
            println!("Label updated to: {}", g.label());
            //g.redraw();
            //g.parent().unwrap().redraw();
            //g.parent().unwrap().parent().unwrap().redraw();
            Ok(())
        })
        .unwrap();
    // last_frame.last_frame.set_label("changed");

    // BLOCK UNTIL CLOSED:
    a.run().unwrap();
    Ok(())
}
