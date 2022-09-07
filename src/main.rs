use anyhow::Context;
use fltk::{image::SharedImage, prelude::*, *};
use std::{fs, ops::Deref, path::PathBuf};
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
    thumb_margin: i32,
    thumb_size: i32,
    per_row: i32,
    image_paths: Option<Vec<PathBuf>>,
}

fn main() -> anyhow::Result<()> {
    let win_width = 640;
    let win_height = 480;
    let thumb_size = 100;
    let per_row = win_width / (thumb_size + 10); // TODO Include gaps, decorations, ...
    let state = State {
        thumb_margin: 10,
        thumb_size,
        per_row,
        image_paths: dir_images("./Pictures/mobil/2022/08").ok(),
    };

    let a = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut win = window::Window::default().with_size(win_width, win_height);
    let mut col = group::Flex::default_fill().column();
    add_img_rows(&mut col, &state)?;
    col.end();
    win.resizable(&col);
    win.set_color(enums::Color::from_rgb(250, 250, 250));
    win.end();
    win.show();
    win.size_range(600, 400, 0, 0);
    a.run().unwrap();
    Ok(())
}

fn add_img_rows(parent: &mut group::Flex, state: &State) -> anyhow::Result<()> {
    if state.image_paths.is_none() {
        return Ok(());
    }

    let img_cnt = state.image_paths.as_ref().unwrap().len() as i32;
    let nr_rows = img_cnt / state.per_row;

    println!(
        "img_cnt {}, rows {}, per row: {}",
        img_cnt, nr_rows, state.per_row
    );

    for chunk in &state
        .image_paths
        .as_ref()
        .unwrap()
        .iter()
        .take(60) // FIXME
        .chunks(state.per_row as usize)
    {
        //group::Flex::debug(true);
        let mut row = group::Flex::default().row();
        parent.set_size(&mut row, state.thumb_size + 2 * state.thumb_margin);
        //parent.resizable(&row);
        for image_path in chunk {
            add_image(state, &mut row, image_path)?;
        }
        row.end();
    }

    Ok(())
}

fn add_image(state: &State, parent: &mut group::Flex, image_path: &PathBuf) -> anyhow::Result<()> {
    let fname = image_path.file_name().unwrap().to_string_lossy();
    //let fname_no_ext = image_path.file_prefix().unwrap().to_string_lossy(); // Unstable 2022-09
    let fpath = image_path.to_string_lossy();

    let mut frame = frame::Frame::default()
        // .with_size( // No effect, likely because flow overrides it?
        //     state.thumb_size + state.thumb_margin,
        //     state.thumb_size + state.thumb_margin,
        // )
        // TODO Label: hide ext. to save space
        .with_label(fname.deref());
    //parent.set_size(&mut frame, state.thumb_size + 100); // sets width b/c parent is row
    frame.set_frame(enums::FrameType::FlatBox);
    //frame.set_align(enums::Align::Wrap); // should wrap label but has 0 effect? Perhaps b/c no spaces in it???
    frame.set_align(enums::Align::Clip);
    frame.set_tooltip(fname.deref());
    frame.set_color(enums::Color::White);

    let mut image = SharedImage::load(fpath.deref()).with_context({
        let f = fpath.deref().to_owned();
        || f
    })?;
    //image.scale(parent.width(), parent.height(), true, true); // OBS these can be 0
    image.scale(state.thumb_size, state.thumb_size, true, true); // TODO Rescale when window expands?

    frame.set_image(Some(image)); // This shows no image: frame.set_image_scaled(Some(image));

    Ok(())
}
