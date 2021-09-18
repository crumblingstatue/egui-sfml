use egui::{CtxRef, Window};
use sfml::{
    graphics::{Color, RenderTarget, RenderWindow},
    window::{ContextSettings, Event, Style},
};

fn ui(ctx: &CtxRef) {
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
    let mut ctx = CtxRef::default();

    while rw.is_open() {
        let mut raw_input = egui_sfml::make_raw_input(&rw);
        while let Some(event) = rw.poll_event() {
            egui_sfml::handle_event(&mut raw_input, &event);
            if matches!(event, Event::Closed) {
                rw.close();
            }
        }
        ctx.begin_frame(raw_input);
        ui(&ctx);
        let frame_result = ctx.end_frame();
        rw.clear(Color::BLACK);
        egui_sfml::draw(
            &mut rw,
            &ctx,
            frame_result.1,
            &mut egui_sfml::DummyTexSource::default(),
        );
        rw.display();
    }
}
