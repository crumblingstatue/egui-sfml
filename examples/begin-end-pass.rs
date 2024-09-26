use {
    egui_sfml::SfEgui,
    sfml::{
        graphics::{Color, RenderTarget, RenderWindow},
        window::{ContextSettings, Event, Style},
    },
};

fn window_1(ctx: &egui::Context) {
    egui::Window::new("Window 1").show(ctx, |ui| {
        ui.label("Hello from window 1!");
    });
}

fn window_2(ctx: &egui::Context) {
    egui::Window::new("Window 2").show(ctx, |ui| {
        ui.label("Hello from window 2!");
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
        // Step 3: Begin the pass
        sfegui.begin_pass();
        // Step 4: Do UI stuff
        window_1(sfegui.context());
        window_2(sfegui.context());
        // Step 5: End the pass
        sfegui.end_pass(&mut rw).unwrap();
        // Step 6: Draw
        rw.clear(Color::rgb(95, 106, 62));
        sfegui.draw(&mut rw, None);
        rw.display();
    }
}
