/** TODO (immediate)
 *
 * - [x] I see "Setting new image..." yet the frame shows only the name, no img - missing render??
 *       - adding .parent.redraw() seems to have fixed it (redrawing the whole row)
 * - [x] Bug: Jump to last row => 1) only it and not the 2 above show imgs, 2) not the whole last row is in viewport
 *            - FIXME When I jump to the end, it shows 5 imgs are visible but the last one is never loaded
 * - [ ] Bug: Sometimes img displays only partially - failed re-rendering trigger x timing??
 *            => add .parent.redraw on double-click and self.redraw on single click or st. to test?
 * - [ ] Bug: Sometimes I set image ("Newly setting image..") yet it does not show and on scroll out&in it logs again & shows
 *            I do not see how that would be possible :-( Given the 2nd log, image is still none but that cannot happen (if error => should panic)
 * - [ ] Performance - is it faster if I load images manually?
 *
*/
use anyhow::Context;
use fltk::{image::SharedImage, prelude::*, *};
use std::{
    cmp::{max, min},
    fs,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{Arc, Mutex},
};
extern crate itertools;
use itertools::Itertools;

const THUMB_PAD: i32 = 5;

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

#[derive(Clone, Debug)]
struct ImageDisplayCont {
    frame: Arc<Mutex<frame::Frame>>,
    image_path: PathBuf,
}

#[derive(Debug)]
struct State {
    thumb_size: i32,
    thumb_margin: i32,

    scroll_pos: i32,
    win_size: (i32, i32),

    nr_images: i32, // added to simpify testing
    image_paths: Option<Vec<PathBuf>>,
    img_frames: Vec<ImageDisplayCont>,
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
        (self.win_size.1 + (self.row_height_outer() - 1)) / self.row_height_outer()
    }
    fn calc_visible_rows(&self) -> (i32, i32) {
        let last_row = max(self.count_rows() - 1, 0);
        let top_y = self.scroll_pos;
        let skipped_rows = min(top_y / self.row_height_outer(), last_row);

        let top_visible_row = skipped_rows; // b/c 0-based
        let bottom_visible_row = min(top_visible_row + self._rows_in_view(), last_row); // not -1 b/c we want extra row at bottom

        (top_visible_row, bottom_visible_row)
    }
    fn calc_visible(&self) -> (i32, i32) {
        let last_img = max(self.count_images() - 1, 0);
        let (top_visible_row, bottom_visible_row) = self.calc_visible_rows();

        let nr_visible_rows = bottom_visible_row - top_visible_row + 1;

        let first_visible_image = min(top_visible_row * self.per_row(), last_img); // 0-based
        let last_visible_image = min(
            first_visible_image + (nr_visible_rows * self.per_row() - 1),
            last_img,
        );

        // println!(
        //     "calc_visible({}): rows {} -> {}, imgs {} -> {} (win height: {})",
        //     self.scroll_pos,
        //     top_visible_row,
        //     bottom_visible_row,
        //     first_visible_image,
        //     last_visible_image,
        //     self.win_size.1
        // );

        (first_visible_image, last_visible_image)
    }
    fn total_height(&self) -> i32 {
        self.count_rows() * self.row_height_outer() - THUMB_PAD // padding is only between rows
    }
    fn per_row(&self) -> i32 {
        max(1, self.win_size.0 / (self.thumb_size + self.thumb_margin))
    }
    fn row_height_inner(&self) -> i32 {
        self.thumb_size + 2 * self.thumb_margin
    }
    fn row_height_outer(&self) -> i32 {
        self.thumb_size + 2 * self.thumb_margin + THUMB_PAD
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
        set_image(&mut frame, image_path, state.thumb_size)?;
    } else {
        frame.set_label(&format!(
            "@refresh {}",
            image_path.file_name().unwrap().to_string_lossy().deref()
        ));
        // FIXME uncomment after troubleshooting done:
        // frame.set_label("@refresh");
        // frame.set_label_size(50);
    }

    Ok(frame)
}

fn set_image(
    frame: &mut frame::Frame,
    image_path: &PathBuf,
    thumb_size: i32,
) -> anyhow::Result<()> {
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
    image.scale(thumb_size, thumb_size, true, true); // TODO Rescale when window expands?

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
        // Parent is a Flex column => sets the row's height (width will grow dynamically)
        parent.set_size(&mut row, state.row_height_inner());
        row.set_pad(THUMB_PAD);

        // FIXME TMP row nr display:
        let mut row_nr_frame = frame::Frame::default().with_label(&format!("{}", nr));
        row_nr_frame.set_frame(enums::FrameType::FlatBox);
        row_nr_frame.set_align(enums::Align::Wrap);
        row.set_size(&row_nr_frame, 36); // need cca 6 per char

        for image_path in chunk {
            let f = add_image(state, image_path, is_visible)?;
            state.img_frames.push(ImageDisplayCont {
                frame: Arc::new(Mutex::new(f)),
                image_path: image_path.clone(),
            });
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let thumb_size = 200;
    let thumb_margin = 10;
    let win_width = (thumb_size + thumb_margin * 2) * 3 + THUMB_PAD * 2;
    let win_height = (thumb_size + thumb_margin * 2) * 2 + THUMB_PAD + 30 /* dbg_panel */;
    let image_paths = dir_images("./Pictures/mobil/2022/08")
        .map(|x| {
            x.into_iter() /*.take(50)*/
                .sorted()
                .collect_vec()
        }) // FIXME rm take
        .ok();
    let mut state =
        State::new((win_width, win_height), thumb_size, thumb_margin).with_image_paths(image_paths);

    let (sender, receiver) = app::channel::<Message>();

    let a = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut win = window::Window::default()
        .with_size(win_width, win_height)
        .center_screen();

    let mut dbg_panel = frame::Frame::new(0, 0, win_width, 30, None);
    {
        //win.set_size(&mut frame, 30); // sets width b/c parent is row
        dbg_panel.set_frame(enums::FrameType::BorderBox);
        // frame.set_align(enums::Align::Clip);
        // frame.set_color(enums::Color::White);
        dbg_panel.set_label("Info panel...");
    }

    let mut scroll = group::Scroll::new(
        0,
        dbg_panel.h(),
        win_width,
        win_height - dbg_panel.h(),
        None,
    );
    scroll.set_frame(enums::FrameType::NoBox); // If all of the child widgets are packed together into a solid rectangle => use FL_NO_BOX *_FRAME
    scroll.set_type(group::ScrollType::Vertical);

    {
        // NOTE We must manually set col.height to > win_h for scrollbar to appear
        let mut col = group::Flex::new(
            scroll.x(),
            scroll.y(),
            win_width,
            state.total_height() + THUMB_PAD, /* for little extra space at the bottom */
            None,
        )
        .column();
        col.set_margin(0);
        col.set_pad(THUMB_PAD);
        add_img_rows(&mut col, &mut state)?;
        col.end();
    }
    scroll.end();
    win.resizable(&scroll); // make the window resizable

    {
        let scroll_state = Arc::new(Mutex::new(ScrollState::default()));
        app::add_timeout3(0.001, move |handle| {
            tick(scroll.clone(), handle, scroll_state.clone(), sender.clone())
        });
    }

    win.set_color(enums::Color::from_rgb(250, 250, 250));
    win.end();
    win.show();
    win.size_range(600, 400, 0, 0);

    // BLOCKS UNTIL CLOSED:
    while a.wait() {
        if let Some(msg) = receiver.recv() {
            match msg {
                Message::RenderBelow(pos) => {
                    state.scroll_pos = pos;
                    let (first_visible_image, last_visible_image) = state.calc_visible();
                    let (first_vis_row, last_vis_row) = state.calc_visible_rows();
                    dbg_panel.set_label(&format!(
                        "Scroll {} / {}, rows {} - {} à {}, imgs {} - {}",
                        pos,
                        state.total_height(),
                        first_vis_row,
                        last_vis_row,
                        state.row_height_inner(),
                        first_visible_image,
                        last_visible_image
                    ));
                    let imgs2show = &state.img_frames
                        [(first_visible_image as usize)..=(last_visible_image as usize)];
                    load_missing_images(&state, imgs2show).unwrap(); // TODO Handle errors better
                }
            }
        }
    }
    Ok(())
}

fn load_missing_images(state: &State, imgs2show: &[ImageDisplayCont]) -> anyhow::Result<()> {
    let thumb_size = state.thumb_size;
    for img_cont in imgs2show {
        let img_cont2 = img_cont.clone();
        tokio::spawn(async move {
            let path = &img_cont2.image_path;
            img_cont2
                .frame
                .lock()
                .and_then(|mut frame| {
                    let frame = &mut frame.deref_mut();
                    if frame.image().is_none() {
                        println!("Info: Newly setting image for {}", path.display());
                        set_image(frame, &path, thumb_size).unwrap(); // TODO Handle err better
                        frame.parent().unwrap().redraw();
                    }
                    Ok(())
                })
                .map_err(|err| {
                    anyhow::anyhow!(
                        "Lock on a frame for {} is poisoned: {}",
                        path.display(),
                        err
                    )
                })
                .unwrap(); // FIXME unwrap not
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    const THUMB_SIZE: i32 = 10;
    const ROW_HEIGHT: i32 = THUMB_SIZE;

    #[test]
    fn calc_visible_basics() {
        // Win 3 x 2 imgs with 12 imgs => 4 rows (2 win heights)
        let mut state = State::new((3 * THUMB_SIZE, 2 * ROW_HEIGHT), THUMB_SIZE, 0)
            .testing_with_nr_images(3 * (2 + 1 + 1));
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
        state.scroll_pos = ROW_HEIGHT - 1;
        assert_eq!(state.calc_visible_rows(), (0, 2)); // unchanged

        // When it fully scrolls out of view, the next one is visible:
        state.scroll_pos = ROW_HEIGHT + THUMB_PAD;
        assert_eq!(state.calc_visible_rows(), (0 + 1, 2 + 1)); // inc by 1
        state.scroll_pos = ROW_HEIGHT + THUMB_PAD + 1;
    }

    /** Test out-of-boundaries cases such as when win is bigger than all images or there are no images. */
    #[test]
    fn calc_visible_boundaries() {
        {
            let state_empty = State::new((3 * THUMB_SIZE, 2 * ROW_HEIGHT), THUMB_SIZE, 0)
                .testing_with_nr_images(0);
            // Visible rows are the whole window (2) + 1 extra => 0 .. 2
            assert_eq!(state_empty._rows_in_view(), 2); // this only depends on thumb size...
            assert_eq!(state_empty.calc_visible_rows(), (0, 0));
            assert_eq!(state_empty.calc_visible(), (0, 0));
        }

        {
            // with 1.3 rows of images only
            let state_half_empty =
                State::new((3 * THUMB_SIZE, 2 * ROW_HEIGHT), 10, 0).testing_with_nr_images(4);
            assert_eq!(state_half_empty._rows_in_view(), 2);
            assert_eq!(state_half_empty.calc_visible_rows(), (0, 1));
            assert_eq!(state_half_empty.calc_visible(), (0, 3));
        }

        {
            // exactly 2 rows of images == full window size
            let mut state_window_full =
                State::new((3 * THUMB_SIZE, 2 * ROW_HEIGHT), 10, 0).testing_with_nr_images(6);
            assert_eq!(state_window_full.calc_visible_rows(), (0, 1));
            assert_eq!(state_window_full.calc_visible(), (0, 5));
            // The user increased the window size and scrolled down so ypos is below existing rows...
            state_window_full.scroll_pos = 3 * ROW_HEIGHT;
            assert_eq!(
                state_window_full.calc_visible_rows(),
                (1, 1),
                "Stops at max rows"
            );
            assert_eq!(
                state_window_full.calc_visible(),
                (3, 5),
                "Stops at max - images for the last row only"
            );
        }
    }
}
