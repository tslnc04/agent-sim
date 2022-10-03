# TODO

- [X] quadtree implementation? or cell grid
- [X] contact tracing graph
- [X] moving/living agents (i.e., have work/home/school)
- [ ] characteristics of infection
- [ ] evolution of infection
- [X] random movements around world
- [X] actually implement basic vector math/positions library
- [X] `fmt::Display` implementation as a grid
- [X] display with colors
- [-] refactor `lib.rs` into multiple files
- [ ] handle agent lifecycle (necessary for implementing work/home/school properly)
- [X] implement schools, refactor so that there is an array of all special
  places rather than separate vectors for each type of building
- [ ] maybe make a config so that fewer things are hardcoded
- [X] create a bounds object to simplify geometry
- [ ] remove unwraps and bubble errors
- [X] fix time? somehow it overflows
- [X] make movements get mirrored in the quadtree
- [ ] make splitting and joining dynamic in the quadtree
