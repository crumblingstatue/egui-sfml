use {
    egui_sfml::SfEgui,
    sfml::{
        graphics::{Color, RenderTarget, RenderWindow},
        window::{ContextSettings, Event, Style},
    },
};

fn main() {
    let mut rw = RenderWindow::new(
        (800, 600),
        "Input test",
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
        sfegui
            .do_pass(&mut rw, |ctx| {
                let win = egui::Window::new("Input test");
                win.show(ctx, |ui| {
                    let evs = ui.input(|inp| inp.raw.events.clone());
                    for ev in evs {
                        ui.label(format!("{ev:?}"));
                    }
                });
            })
            .unwrap();
        // Step 4: Draw
        rw.clear(Color::rgb(95, 106, 62));
        sfegui.draw(&mut rw, None);
        rw.display();
    }
}
