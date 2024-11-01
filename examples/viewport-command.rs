use {
    egui::ViewportCommand,
    egui_sfml::SfEgui,
    sfml::{
        graphics::{Color, Rect, RenderTarget, RenderWindow, View},
        window::{ContextSettings, Event, Style},
    },
    std::time::Instant,
};

#[derive(Default)]
struct UiState {
    title: String,
    invisible_set_instant: Option<Instant>,
    focus_req_instant: Option<Instant>,
}

fn main() {
    let mut rw = RenderWindow::new(
        (640, 480),
        "ViewportCommand test",
        Style::DEFAULT,
        &ContextSettings::default(),
    )
    .unwrap();
    rw.set_vertical_sync_enabled(true);
    let mut sf_egui = SfEgui::new(&rw);
    let mut ui_state = UiState::default();
    let mut view = View::new().unwrap();

    while rw.is_open() {
        while let Some(ev) = rw.poll_event() {
            sf_egui.add_event(&ev);
            match ev {
                Event::Closed => rw.close(),
                Event::Resized { width, height } => {
                    view.reset(Rect::new(0., 0., width as f32, height as f32));
                    rw.set_view(&view);
                }
                _ => {}
            }
        }
        let di = sf_egui
            .run(&mut rw, |_rw, ctx| ui(ctx, &mut ui_state))
            .unwrap();
        rw.clear(Color::BLACK);
        sf_egui.draw(di, &mut rw, None);
        rw.display();
    }
}

fn ui(ctx: &egui::Context, state: &mut UiState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if ui.button("Close window (quit)").clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        ui.label("Window title");
        if ui.text_edit_singleline(&mut state.title).changed() {
            ctx.send_viewport_cmd(ViewportCommand::Title(state.title.clone()));
        }
        if ui.button("Hide for 2 seconds").clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
            state.invisible_set_instant = Some(Instant::now());
        }
        if let Some(instant) = state.invisible_set_instant {
            if instant.elapsed().as_secs() >= 2 {
                ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                state.invisible_set_instant = None;
            }
        }
        if ui.button("Focus in 2 seconds").clicked() {
            state.focus_req_instant = Some(Instant::now());
        }
        if let Some(instant) = state.focus_req_instant {
            if instant.elapsed().as_secs() >= 2 {
                ctx.send_viewport_cmd(ViewportCommand::Focus);
                state.focus_req_instant = None;
            }
        }
    });
}
