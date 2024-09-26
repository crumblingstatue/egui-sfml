use {
    egui::TextBuffer,
    egui_sfml::SfEgui,
    sfml::{
        graphics::{Color, RenderTarget, RenderWindow},
        window::{ContextSettings, Event, Style},
    },
};

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

    let mut message = String::new();
    let mut messages = Vec::new();

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
                let win = egui::Window::new("Hello egui-sfml!");
                win.show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Message");
                        let te_re = ui.text_edit_singleline(&mut message);
                        if ui.button("Send").clicked()
                            || ui.input(|inp| inp.key_pressed(egui::Key::Enter))
                        {
                            messages.push(message.take());
                            te_re.request_focus();
                        }
                    });
                    for msg in &messages {
                        ui.separator();
                        ui.label(msg);
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
