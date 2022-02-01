//! egui SFML integration helpers
//!
//! Contains various types and functions that helps with integrating egui with SFML.

#![warn(missing_docs)]

use std::collections::HashMap;
use std::mem;

use egui::epaint::ClippedShape;
use egui::{
    Context, Event as EguiEv, ImageData, Modifiers, Output, PointerButton, Pos2, RawInput,
    TextureId,
};
use sfml::graphics::blend_mode::Factor;
use sfml::graphics::{
    BlendMode, Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Texture, Vertex,
};
use sfml::window::clipboard;
use sfml::{
    window::{mouse, Event, Key},
    SfBox,
};

pub use egui;
pub use sfml;

fn button_conv(button: mouse::Button) -> PointerButton {
    match button {
        mouse::Button::Left => PointerButton::Primary,
        mouse::Button::Right => PointerButton::Secondary,
        mouse::Button::Middle => PointerButton::Middle,
        _ => panic!("Unhandled pointer button: {:?}", button),
    }
}

fn key_conv(code: Key) -> Option<egui::Key> {
    use egui::Key as EKey;
    Some(match code {
        Key::Down => EKey::ArrowDown,
        Key::Left => EKey::ArrowLeft,
        Key::Right => EKey::ArrowRight,
        Key::Up => EKey::ArrowUp,
        Key::Escape => EKey::Escape,
        Key::Tab => EKey::Tab,
        Key::Backspace => EKey::Backspace,
        Key::Enter => EKey::Enter,
        Key::Space => EKey::Space,
        Key::Insert => EKey::Insert,
        Key::Delete => EKey::Delete,
        Key::Home => EKey::Home,
        Key::End => EKey::End,
        Key::PageUp => EKey::PageUp,
        Key::PageDown => EKey::PageDown,
        Key::Num0 => EKey::Num0,
        Key::Num1 => EKey::Num1,
        Key::Num2 => EKey::Num2,
        Key::Num3 => EKey::Num3,
        Key::Num4 => EKey::Num4,
        Key::Num5 => EKey::Num5,
        Key::Num6 => EKey::Num6,
        Key::Num7 => EKey::Num7,
        Key::Num8 => EKey::Num8,
        Key::Num9 => EKey::Num9,
        Key::A => EKey::A,
        Key::B => EKey::B,
        Key::C => EKey::C,
        Key::D => EKey::D,
        Key::E => EKey::E,
        Key::F => EKey::F,
        Key::G => EKey::G,
        Key::H => EKey::H,
        Key::I => EKey::I,
        Key::J => EKey::J,
        Key::K => EKey::K,
        Key::L => EKey::L,
        Key::M => EKey::M,
        Key::N => EKey::N,
        Key::O => EKey::O,
        Key::P => EKey::P,
        Key::Q => EKey::Q,
        Key::R => EKey::R,
        Key::S => EKey::S,
        Key::T => EKey::T,
        Key::U => EKey::U,
        Key::V => EKey::V,
        Key::W => EKey::W,
        Key::X => EKey::X,
        Key::Y => EKey::Y,
        Key::Z => EKey::Z,
        _ => return None,
    })
}

fn modifier(alt: bool, ctrl: bool, shift: bool) -> egui::Modifiers {
    egui::Modifiers {
        alt,
        ctrl,
        shift,
        command: ctrl,
        mac_cmd: false,
    }
}

/// Converts an SFML event to an egui event and adds it to the `RawInput`.
fn handle_event(raw_input: &mut egui::RawInput, event: &sfml::window::Event) {
    match *event {
        Event::KeyPressed {
            code,
            alt,
            ctrl,
            shift,
            system: _,
        } => {
            if ctrl {
                match code {
                    Key::V => raw_input
                        .events
                        .push(egui::Event::Text(clipboard::get_string().to_rust_string())),
                    Key::C => raw_input.events.push(egui::Event::Copy),
                    Key::X => raw_input.events.push(egui::Event::Cut),
                    _ => {}
                }
            }
            if let Some(key) = key_conv(code) {
                raw_input.events.push(egui::Event::Key {
                    key,
                    modifiers: modifier(alt, ctrl, shift),
                    pressed: true,
                });
            }
        }
        Event::KeyReleased {
            code,
            alt,
            ctrl,
            shift,
            system: _,
        } => {
            if let Some(key) = key_conv(code) {
                raw_input.events.push(egui::Event::Key {
                    key,
                    modifiers: modifier(alt, ctrl, shift),
                    pressed: false,
                });
            }
        }
        Event::MouseMoved { x, y } => {
            raw_input
                .events
                .push(EguiEv::PointerMoved(Pos2::new(x as f32, y as f32)));
        }
        Event::MouseButtonPressed { x, y, button } => {
            raw_input.events.push(EguiEv::PointerButton {
                pos: Pos2::new(x as f32, y as f32),
                button: button_conv(button),
                pressed: true,
                modifiers: Modifiers::default(),
            });
        }
        Event::MouseButtonReleased { x, y, button } => {
            raw_input.events.push(EguiEv::PointerButton {
                pos: Pos2::new(x as f32, y as f32),
                button: button_conv(button),
                pressed: false,
                modifiers: Modifiers::default(),
            });
        }
        Event::TextEntered { unicode } => {
            if !unicode.is_control() {
                raw_input.events.push(EguiEv::Text(unicode.to_string()));
            }
        }
        Event::MouseWheelScrolled { delta, .. } => {
            if sfml::window::Key::LControl.is_pressed() {
                raw_input
                    .events
                    .push(EguiEv::Zoom(if delta > 0.0 { 1.1 } else { 0.9 }));
            }
        }
        _ => {}
    }
}

/// Creates a `RawInput` that fits the window.
fn make_raw_input(window: &RenderWindow) -> RawInput {
    RawInput {
        screen_rect: Some(egui::Rect {
            min: Pos2::new(0., 0.),
            max: Pos2::new(window.size().x as f32, window.size().y as f32),
        }),
        ..Default::default()
    }
}

/// A source for egui user textures.
///
/// You can create a struct that contains all the necessary information to get a user texture from
/// an id, and implement this trait for it.
pub trait UserTexSource {
    /// Get the texture that corresponds to `id`.
    ///
    /// Returns (width, height, texture).
    fn get_texture(&mut self, id: u64) -> (f32, f32, &Texture);
}

/// A dummy texture source in case you don't care about providing user textures
struct DummyTexSource {
    tex: SfBox<Texture>,
}

impl Default for DummyTexSource {
    fn default() -> Self {
        Self {
            tex: Texture::new().unwrap(),
        }
    }
}

impl UserTexSource for DummyTexSource {
    fn get_texture(&mut self, _id: u64) -> (f32, f32, &Texture) {
        (0., 0., &self.tex)
    }
}

/// `Egui` integration for SFML.
pub struct SfEgui {
    ctx: Context,
    raw_input: RawInput,
    egui_result: (Output, Vec<ClippedShape>),
    textures: HashMap<TextureId, SfBox<Texture>>,
}

impl SfEgui {
    /// Create a new `SfEgui`.
    ///
    /// The size of the egui ui will be the same as `window`'s size.
    pub fn new(window: &RenderWindow) -> Self {
        Self {
            raw_input: make_raw_input(window),
            ctx: Context::default(),
            egui_result: Default::default(),
            textures: HashMap::default(),
        }
    }
    /// Convert an SFML event into an egui event and add it for later use by egui.
    ///
    /// Call this in an event polling loop for each event.
    pub fn add_event(&mut self, event: &Event) {
        handle_event(&mut self.raw_input, event);
    }
    /// Does an egui frame with a user supplied ui function.
    ///
    /// The `f` parameter is a user supplied ui function that does the desired ui
    pub fn do_frame(&mut self, f: impl FnOnce(&Context)) {
        self.egui_result = self.ctx.run(self.raw_input.take(), f);
        let clip_str = &self.egui_result.0.copied_text;
        if !clip_str.is_empty() {
            clipboard::set_string(clip_str);
        }
        for (id, delta) in &self.egui_result.0.textures_delta.set {
            let [w, h] = delta.image.size();
            let tex = self.textures.entry(*id).or_insert_with(|| {
                let mut tex = Texture::new().unwrap();
                if !tex.create(w as u32, h as u32) {
                    panic!();
                }
                tex
            });
            update_tex_from_delta(tex, delta);
        }
    }
    /// Draw the ui to a `RenderWindow`.
    ///
    /// Takes an optional [`UserTexSource`] to act as a user texture source.
    pub fn draw(
        &mut self,
        window: &mut RenderWindow,
        user_tex_src: Option<&mut dyn UserTexSource>,
    ) {
        draw(
            window,
            &self.ctx,
            mem::take(&mut self.egui_result.1),
            user_tex_src.unwrap_or(&mut DummyTexSource::default()),
            &self.textures,
        )
    }
    /// Returns a handle to the egui context
    ///
    /// `CtxRef` can be cloned, but beware that it will be outdated after a call to
    /// [`do_frame`](Self::do_frame)
    pub fn context(&self) -> &Context {
        &self.ctx
    }
}

fn update_tex_from_delta(tex: &mut SfBox<Texture>, delta: &egui::epaint::ImageDelta) {
    let mut x = 0;
    let mut y = 0;
    let [w, h] = delta.image.size();
    if let Some([xx, yy]) = delta.pos {
        x = xx as u32;
        y = yy as u32;
    }
    match &delta.image {
        ImageData::Color(color) => {
            let srgba: Vec<u8> = color.pixels.iter().flat_map(|c32| c32.to_array()).collect();
            unsafe {
                tex.update_from_pixels(&srgba, w as u32, h as u32, x, y);
            }
        }
        ImageData::Alpha(alpha) => {
            let srgba: Vec<u8> = alpha
                .srgba_pixels(1.0)
                .flat_map(|c32| c32.to_array())
                .collect();
            unsafe {
                tex.update_from_pixels(&srgba, w as u32, h as u32, x, y);
            }
        }
    }
}

fn draw(
    window: &mut RenderWindow,
    egui_ctx: &egui::Context,
    shapes: Vec<egui::epaint::ClippedShape>,
    user_tex_source: &mut dyn UserTexSource,
    textures: &HashMap<TextureId, SfBox<Texture>>,
) {
    window.set_active(true);
    unsafe {
        glu_sys::glEnable(glu_sys::GL_SCISSOR_TEST);
    }
    let mut vertices = Vec::new();
    for egui::ClippedMesh(rect, mesh) in egui_ctx.tessellate(shapes) {
        let (tw, th, tex) = match mesh.texture_id {
            TextureId::Managed(id) => {
                let tex = &*textures[&TextureId::Managed(id)];
                let (egui_tex_w, egui_tex_h) = (tex.size().x as f32, tex.size().y as f32);
                (egui_tex_w, egui_tex_h, &*tex)
            }
            TextureId::User(id) => user_tex_source.get_texture(id),
        };
        for idx in mesh.indices {
            let v = mesh.vertices[idx as usize];
            let sf_v = Vertex::new(
                (v.pos.x, v.pos.y).into(),
                Color::rgba(v.color.r(), v.color.g(), v.color.b(), v.color.a()),
                (v.uv.x * tw, v.uv.y * th).into(),
            );
            vertices.push(sf_v);
        }
        let mut rs = RenderStates::default();
        rs.set_blend_mode(BlendMode {
            color_src_factor: Factor::One,
            color_dst_factor: Factor::OneMinusSrcAlpha,
            alpha_src_factor: Factor::OneMinusDstAlpha,
            alpha_dst_factor: Factor::One,
            ..Default::default()
        });
        rs.set_texture(Some(tex));
        let pixels_per_point = 1.;
        let win_size = window.size();
        let width_in_pixels = win_size.x;
        let height_in_pixels = win_size.y;
        // Code copied from egui_glium (https://github.com/emilk/egui)
        // Transform clip rect to physical pixels:
        let clip_min_x = pixels_per_point * rect.min.x;
        let clip_min_y = pixels_per_point * rect.min.y;
        let clip_max_x = pixels_per_point * rect.max.x;
        let clip_max_y = pixels_per_point * rect.max.y;

        // Make sure clip rect can fit within a `u32`:
        let clip_min_x = clip_min_x.clamp(0.0, width_in_pixels as f32);
        let clip_min_y = clip_min_y.clamp(0.0, height_in_pixels as f32);
        let clip_max_x = clip_max_x.clamp(clip_min_x, width_in_pixels as f32);
        let clip_max_y = clip_max_y.clamp(clip_min_y, height_in_pixels as f32);

        let clip_min_x = clip_min_x.round() as u32;
        let clip_min_y = clip_min_y.round() as u32;
        let clip_max_x = clip_max_x.round() as u32;
        let clip_max_y = clip_max_y.round() as u32;
        unsafe {
            glu_sys::glScissor(
                clip_min_x as _,
                (height_in_pixels - clip_max_y) as _,
                (clip_max_x - clip_min_x) as _,
                (clip_max_y - clip_min_y) as _,
            );
        }
        window.draw_primitives(&vertices, PrimitiveType::TRIANGLES, &rs);
        vertices.clear();
    }
    unsafe {
        glu_sys::glDisable(glu_sys::GL_SCISSOR_TEST);
    }
    window.set_active(false);
}
