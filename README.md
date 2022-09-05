My project to learn Rust - an image manager GUI.

# Roadmap

- [x] Display an image
- [ ] Make the image maximized wrt. screen size
- [ ] Display all images in a folder in a grid of thumbnails (of customizable size)
- [ ] View a selected image in max size
- [ ] View multiple images, for visual comparison (and selection)
- [ ] Operations: Create sub-folder; rm, mv, rename image
  - [ ] Persistent selection of multiple images + batch operations


# TODO

* Handle secondary display
  * What if window moved to another monitor after creation?
* Image view operations - zoom, rotate, save, undo

# Struggles log

 * How to ensure no-copy display for efficiency? Handle::from_fixels requires clone, so does image of handle ... => clone of Handle is cheap
 * How to pass pre-loaded image to the app? => Sandbox -> Application & use Flags
 * Image not rendering => incompatible versions of `image` create between me and iced-wgpu => use theirs
 * Image display window having unexpected horizontal margin - b/c the image was auto-scaled to fit into the display that is smaller than them image but the width that was set to less than the physical screen width was left as it was => find out the phys. size and scale the window manually 

 ---

 Copyleft Jakub Holý 2022 - published under the Unlicense