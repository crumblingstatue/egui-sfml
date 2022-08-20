# egui-sfml

[SFML](https://github.com/jeremyletang/rust-sfml) integration for [egui](https://github.com/emilk/egui).

This library allows using egui for rust-sfml projects.
It's a very easy way to add a functional gui to your rust-sfml game or application!

All you need to do is:
- Create an `SfEgui`
- Feed it SFML events using `add_event`
- Do an egui frame with `do_frame`
- Draw the ui with `draw`

See `examples/hello.rs` for a simple demo.
