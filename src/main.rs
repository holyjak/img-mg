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

#[derive(Debug)]
struct State {
    thumb_size: i32,
    thumb_margin: i32,

    scroll_pos: i32,
    win_size: (i32, i32),

    nr_images: i32, // added to simpify testing
    image_paths: Option<Vec<PathBuf>>,
    img_frames: Vec<Arc<Mutex<frame::Frame>>>,
}

impl State {
    fn new(win_size: (i32, i32), thumb_size: i32, thumb_margin: i32) -> Self {
        State {
            thumb_size,
            thumb_margin,
            scroll_pos: 0,
            win_size,
            nr_images: 0,
            image_paths: Option::None,
            img_frames: vec![],
        }
    }
    fn with_image_paths(mut self, image_paths: Option<Vec<PathBuf>>) -> State {
        self.nr_images = State::count_these_images(&image_paths);
        self.image_paths = image_paths;
        self
    }
    #[cfg(test)]
    fn testing_with_nr_images(mut self, nr_images: i32) -> State {
        self.nr_images = nr_images;
        self
    }
    fn _rows_in_view(&self) -> i32 {
        (self.win_size.1 + (self.row_height() - 1)) / self.row_height()
    }
    fn calc_visible_rows(&self) -> (i32, i32) {
        let top_y = self.scroll_pos;
        let skipped_rows = top_y / self.row_height();

        let top_visible_row = skipped_rows; // b/c 0-based
        let bottom_visible_row = top_visible_row + self._rows_in_view(); // not -1 b/c we want extra row at bottom

        (top_visible_row, bottom_visible_row)
    }
    fn calc_visible(&self) -> (i32, i32) {
        let (top_visible_row, bottom_visible_row) = self.calc_visible_rows();

        let nr_visible_rows = bottom_visible_row - top_visible_row + 1;

        let first_visible_image = top_visible_row * self.per_row(); // 0-based
        let last_visible_image = first_visible_image + (nr_visible_rows * self.per_row() - 1);

        // println!(
        //     "calc_visible: rows {} -> {}, imgs {} -> {}",
        //     top_visible_row, bottom_visible_row, first_visible_image, last_visible_image
        // );

        (first_visible_image, last_visible_image)
    }
    fn total_height(&self) -> i32 {
        self.count_rows() * self.row_height()
    }
    fn per_row(&self) -> i32 {
        self.win_size.0 / (self.thumb_size + self.thumb_margin)
    }
    fn row_height(&self) -> i32 {
        self.thumb_size + 2 * self.thumb_margin
    }

    fn count_images(&self) -> i32 {
        self.nr_images
    }
    fn count_these_images(image_paths: &Option<Vec<PathBuf>>) -> i32 {
        image_paths.as_ref().map_or(0, |v| v.len()) as i32
    }
    fn count_rows(&self) -> i32 {
        (self.count_images() + (self.per_row() - 1)) / self.per_row()
    }
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
        //.take(15) // FIXME
        .chunks(state.per_row() as usize)
        .into_iter()
        .enumerate()
    {
        let (first_row, last_row) = state.calc_visible_rows();
        //group::Flex::debug(true);
        let nr = row_nr as i32;
        let is_visible = nr >= first_row && nr <= last_row;
        // println!(
        //     "Rendering row nr {} vis: {} <> in {:?}",
        //     row_nr, is_visible, state.visible_rows
        // );
        let mut row = group::Flex::default().row();
        parent.set_size(&mut row, state.row_height());
        //parent.resizable(&row);
        for image_path in chunk {
            let f = add_image(state, &mut row, image_path, is_visible)?;
            state.img_frames.push(Arc::new(Mutex::new(f)));
        }
        row.end();
    }

    Ok(())
}

#[derive(Clone, Default, Debug)]
struct ScrollState {
    scrolling: bool,
    scroll_pos: i32,
}
enum ScrollReaction {
    None,
    Render(i32),
}
impl ScrollState {
    fn update(&mut self, current_scroll_position: i32) -> ScrollReaction {
        let changed = self.scroll_pos != current_scroll_position;
        match self {
            ScrollState {
                scrolling: false, ..
            } if !changed => ScrollReaction::None,
            ScrollState {
                scrolling: false, ..
            } if changed => {
                //println!("Scrolling started... ");
                self.scroll_pos = current_scroll_position;
                self.scrolling = true;
                ScrollReaction::None
            }
            ScrollState {
                scrolling: true, ..
            } if !changed => {
                //println!("Scrolling stopped => render!");
                self.scrolling = false;
                ScrollReaction::Render(current_scroll_position)
            }
            ScrollState {
                scrolling: true, ..
            } if changed => {
                self.scroll_pos = current_scroll_position;
                ScrollReaction::None
            }
            _ => panic!(),
        }
    }
}

/**
 * Check scroll position at regular intervals to detect scrolling happening.
 *
 * See https://groups.google.com/g/fltkgeneral/c/z7B7QT45cpk/m/eKZlaO4xCAAJ for
 * an alternative solution that subclasses Fl_Scroll and overrides its
 * scrollbar's callback to also perform an action upon scrolling.
 */
fn tick(
    scroll: group::Scroll,
    handle: app::TimeoutHandle,
    scroll_state_ptr: Arc<Mutex<ScrollState>>,
    sender: app::Sender<Message>,
) {
    let mut scroll_state = scroll_state_ptr.lock().unwrap();
    match scroll_state.deref_mut().update(scroll.yposition()) {
        ScrollReaction::Render(pos) => sender.send(Message::RenderBelow(pos)),
        _ => (),
    }
    app::repeat_timeout3(0.002, handle);
}

#[derive(Clone, Debug)]
enum Message {
    RenderBelow(i32),
}

fn main() -> anyhow::Result<()> {
    let win_width = 640;
    let win_height = 480;
    let thumb_size = 200;
    let thumb_margin = 10;
    let image_paths = dir_images("./Pictures/mobil/2022/08").ok();
    let mut state =
        State::new((win_width, win_height), thumb_size, thumb_margin).with_image_paths(image_paths);

    let (sender, receiver) = app::channel::<Message>();

    state.calc_visible(); // TODO rm

    let a = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut win = window::Window::default().with_size(win_width, win_height);

    let mut scroll = group::Scroll::default_fill(); //new(0, 0, win_width, win_height, None);
                                                    // .with_pos(0, 0)
                                                    // .with_size(win_width, win_height);
    scroll.set_type(group::ScrollType::Vertical);
    // NOTE We must manually set col.height to > win_h for scrollbar to appear
    let mut col = group::Flex::new(0, 0, win_width, state.total_height(), None).column();
    add_img_rows(&mut col, &mut state)?;
    col.end();
    scroll.end();

    {
        let scroll_state = Arc::new(Mutex::new(ScrollState::default()));
        app::add_timeout3(0.001, move |handle| {
            tick(scroll.clone(), handle, scroll_state.clone(), sender.clone())
        });
    }

    win.resizable(&col); // make the window resizable
    win.set_color(enums::Color::from_rgb(250, 250, 250));
    win.end();
    win.show();
    win.size_range(600, 400, 0, 0);

    // FIXME rm DEMO code - ex. of mutating a displayed image
    // state
    //     .img_frames
    //     .last()
    //     .unwrap()
    //     .lock()
    //     .and_then(|mut g| {
    //         let path = PathBuf::from("img.jpg");
    //         let mut frame: &mut frame::Frame = g.deref_mut();
    //         set_image(&state, frame, &path).unwrap();
    //         println!("Label updated to: {}", g.label());
    //         //g.redraw();
    //         //g.parent().unwrap().redraw();
    //         //g.parent().unwrap().parent().unwrap().redraw();
    //         Ok(())
    //     })
    //     .unwrap();
    // last_frame.last_frame.set_label("changed");

    // BLOCK UNTIL CLOSED:
    while a.wait() {
        if let Some(msg) = receiver.recv() {
            match msg {
                Message::RenderBelow(pos) => {
                    state.scroll_pos = pos;
                    state.calc_visible();
                    println!("TODO: Calculate rendering from ypos {}", pos);
                }
            }
        }
    }
    Ok(())
}

// struct State {
//     thumb_size: i32,
//     per_row: i32,
//     image_paths: Option<Vec<PathBuf>>,
//     row_height: i32,
//     total_height: i32,
//     visible_rows: (i32, i32),
//     img_frames: Vec<Arc<Mutex<frame::Frame>>>,
// }

// FIXME: BREAK OFF POINT IS TOO LATE
// (row_height: 220)
// Only when viewing rows 2+3 and a bit (ie 0+1 off screen) does vis. rows change:
// ypos 432 => calc_visible: rows 0 -> 2, imgs 0 -> 8
// ypos 448 => calc_visible: rows 1 -> 3, imgs 3 -> 11
//       => top 2 rows hidden (2*220), 2.x more rows visible <=> `rows 1 -> 3` is off
//       => ??? why not change at 221 ???

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn calc_visible() {
        let mut state = State::new((30, 20), 10, 0).testing_with_nr_images(3 * (2 + 1 + 1));
        // Visible rows are the whole window (2) + 1 extra => 0 .. 2
        assert_eq!(state._rows_in_view(), 2);
        assert_eq!(
            state.calc_visible_rows(),
            (0, 1 + 1),
            "Visible rows are the whole window (2) + 1 extra => rows 0 .. 2"
        );
        assert_eq!(
            state.calc_visible(),
            (0, 3 * 3 - 1),
            "3 visible rows à 3 img => 9 images, namely 0 .. 8"
        );

        // As long as even 1px is visible of the top row, it is still "visible":
        state.scroll_pos = 1; // > 0
        assert_eq!(state.calc_visible_rows(), (0, 2)); // unchanged
        state.scroll_pos = 9; // row_height - 1
        assert_eq!(state.calc_visible_rows(), (0, 2)); // unchanged

        // When it fully scrolls out of view, the next one is visible:
        state.scroll_pos = 10; // row_height
                               // ???? assert_eq!(state.calc_visible_rows(), (0 + 1, 2 + 1)); // inc by 1
        assert_eq!(state.calc_visible_rows(), (0 + 1, 2 + 1)); // inc by 1
        state.scroll_pos = 11; // row_height + 1
    }
}
