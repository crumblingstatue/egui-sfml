use egui_sfml::egui::{Context, Window};
use egui_sfml::SfEgui;
use sfml::{
    graphics::{Color, RenderTarget, RenderWindow},
    window::{ContextSettings, Event, Style},
};

fn ui(ctx: &Context) {
    let win = Window::new("Hello egui-sfml!");
    win.show(ctx, |ui| {
        ui.label("Hello world!");
        let _ = ui.button("Click me!");
    });
}

fn main() {
    let mut rw = RenderWindow::new(
        (800, 600),
        "Hello egui!",
        Style::CLOSE,
        &ContextSettings::default(),
    );
    rw.set_vertical_sync_enabled(true);
    // Step 1: Create an SfEgui
    let mut sfegui = SfEgui::new(&rw);

    while rw.is_open() {
        while let Some(event) = rw.poll_event() {
            // Step 2: Collect events from the event loop
            sfegui.add_event(&event);
            if matches!(event, Event::Closed) {
                rw.close();
            }
        }
        // Step 3: Do an egui frame with the desired ui function
        sfegui.do_frame(ui);
        rw.clear(Color::BLACK);
        // Step 4: Draw
        sfegui.draw(&mut rw, None);
        rw.display();
    }
}
