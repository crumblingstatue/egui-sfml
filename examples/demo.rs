#![feature(new_uninit)]

use std::rc::Rc;

use egui_demo_lib::WrapApp;
use egui_sfml::SfEgui;
use epi::{backend, App, IntegrationInfo};
use sfml::{
    graphics::{Color, Rect, RenderTarget, RenderWindow, View},
    window::{Event, Style, VideoMode},
};

fn main() {
    let mut app = WrapApp::default();
    let vm = VideoMode::desktop_mode();
    let mut rw = RenderWindow::new(vm, "Egui test", Style::NONE, &Default::default());
    rw.set_position((0, 0).into());
    rw.set_vertical_sync_enabled(true);
    let mut frame = epi::Frame {
        info: IntegrationInfo {
            cpu_usage: None,
            native_pixels_per_point: None,
            prefer_dark_mode: None,
            web_info: None,
            name: "egui-sfml",
        },
        output: backend::AppOutput::default(),
        storage: None,
        gl: unsafe {
            eprintln!("TODO: Fucked up workaround");
            Rc::new_uninit().assume_init()
        },
    };
    let mut sfegui = SfEgui::new(&rw);
    while rw.is_open() {
        while let Some(ev) = rw.poll_event() {
            sfegui.add_event(&ev);
            match ev {
                Event::Closed => {
                    rw.close();
                }
                Event::Resized { width, height } => {
                    rw.set_view(&View::from_rect(&Rect::new(
                        0.,
                        0.,
                        width as f32,
                        height as f32,
                    )));
                }
                _ => {}
            }
        }
        sfegui.do_frame(|ctx| {
            app.update(ctx, &mut frame);
        });
        rw.clear(Color::BLACK);
        sfegui.draw(&mut rw, None);
        rw.display();
    }
}
