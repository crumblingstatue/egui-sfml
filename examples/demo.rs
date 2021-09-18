use std::sync::Arc;

use egui::CtxRef;
use egui_demo_lib::WrapApp;
use egui_sfml::UserTexSource;
use epi::backend;
use epi::{App, IntegrationInfo, RepaintSignal, TextureAllocator};
use sfml::{
    graphics::{Color, Rect, RenderTarget, RenderWindow, View},
    window::{Event, Style, VideoMode},
};

struct RepaintSig {}

impl RepaintSignal for RepaintSig {
    fn request_repaint(&self) {}
}

struct TexAlloc {}

impl TextureAllocator for TexAlloc {
    fn alloc_srgba_premultiplied(
        &mut self,
        _size: (usize, usize),
        _srgba_pixels: &[egui::Color32],
    ) -> egui::TextureId {
        todo!()
    }
    fn free(&mut self, _id: egui::TextureId) {
        todo!()
    }
}

struct TexSrc {}

impl UserTexSource for TexSrc {
    fn get_texture(&mut self, _id: u64) -> (f32, f32, &sfml::graphics::Texture) {
        todo!()
    }
}

fn main() {
    let mut app = WrapApp::default();
    let vm = VideoMode::desktop_mode();
    let mut rw = RenderWindow::new(vm, "Egui test", Style::NONE, &Default::default());
    rw.set_position((0, 0).into());
    rw.set_vertical_sync_enabled(true);
    let mut app_out = backend::AppOutput::default();
    let mut ta = TexAlloc {};
    let mut frame = backend::FrameBuilder {
        info: IntegrationInfo {
            cpu_usage: None,
            native_pixels_per_point: None,
            seconds_since_midnight: None,
            prefer_dark_mode: None,
            web_info: None,
        },
        output: &mut app_out,
        repaint_signal: Arc::new(RepaintSig {}),
        tex_allocator: &mut ta,
    }
    .build();
    let mut ctx_ref = CtxRef::default();
    egui_sfml::get_first_texture(&mut ctx_ref, &rw);
    let mut tex_src = TexSrc {};
    while rw.is_open() {
        let mut raw_input = egui_sfml::make_raw_input(&rw);
        while let Some(ev) = rw.poll_event() {
            egui_sfml::handle_event(&mut raw_input, &ev);
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
        let tex = egui_sfml::get_new_texture(&ctx_ref);
        ctx_ref.begin_frame(raw_input);
        app.update(&ctx_ref, &mut frame);
        rw.clear(Color::BLACK);
        let ef = ctx_ref.end_frame();
        egui_sfml::draw(&mut rw, &ctx_ref, &tex, ef.1, &mut tex_src);
        rw.display();
    }
}
