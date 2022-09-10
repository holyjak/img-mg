My project to learn Rust - an image manager GUI.

# Roadmap

- [x] Display all images in a folder in a grid of thumbnails (of customizable size)
  - [x] Performance: Don't waste time rendering what is off viewport
  - [x] Scrollbar
  - [x] (Pre-)render more images as the user is scrolling (start w/ an empty frame / loading icon - eg. `@refresh`) & consider garbage collecting those off screen
    - Make smoother
    - Do async so UI thread not blocked
    - Show that the images are being loaded
- [ ] Resize the window => recalc. rows
- [ ] Display an image
- [ ] Make the image maximized wrt. screen size
- [ ] View a selected image in max size
- [ ] View multiple images, for visual comparison (and selection)
- [ ] Operations: Create sub-folder; rm, mv, rename image, rotate
  - [ ] Persistent selection of multiple images + batch operations


# TODO

* Handle secondary display
  * What if window moved to another monitor after creation?
* Image view operations - zoom, rotate, save, undo

# Notes

## Design

### FLTK resources

**Flex** - see screenshots in https://github.com/osen/FL_Flex and Rust code in https://github.com/fltk-rs/fltk-flex/

* http://seriss.com/people/erco/fltk/#ScrollableImage

**Flow** - a new, rules based layout manager, somehow similar to css flexbox, 
see https://github.com/fltk-rs/fltk-flow and especially https://github.com/osen/Fl_Flow

### What GUI platform to use?

Alternatives: 

* Iced - Pros: he two most mature GUI libraries for Rust seem to be Iced and egui but the former use an immediate-mode display, where the app is rendered from scratch on each frame. Great for game engines but not for a mostly static UI where I want the best performance => no unnecessary re-rendering. CONS: No support for multiple windows (to preview an image), yet no way to get phys monitor size (though likely simple to add).
* GTK/Qt (cons: huge binary)
* Tauri - but how to load + manipulate image in Rust into memory once, then display it in the web-rendered part?!
* fltk - small, statically link-able C++ lib (1MB)
  * size, position manually or use flexbox layout
  * supports multiple windows
  * supports images
  * exposes monitor size etc https://docs.rs/fltk/latest/src/screens_info/screens_info.rs.html#28

## Struggles log

### Iced

 * How to ensure no-copy display for efficiency? Handle::from_fixels requires clone, so does image of handle ... => clone of Handle is cheap
 * How to pass pre-loaded image to the app? => Sandbox -> Application & use Flags
 * Image not rendering => incompatible versions of `image` create between me and iced-wgpu => use theirs
 * Image display window having unexpected horizontal margin - b/c the image was auto-scaled to fit into the display that is smaller than them image but the width that was set to less than the physical screen width was left as it was => find out the phys. size and scale the window manually 

 ---

 Copyleft Jakub Holý 2022 - published under the Unlicense