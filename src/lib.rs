//! egui SFML integration helpers
//!
//! Contains various types and functions that helps with integrating egui with SFML.

#![warn(missing_docs)]

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;

use egui::epaint::Primitive;
use egui::{
    Context, Event as EguiEv, FullOutput, ImageData, Modifiers, PointerButton, Pos2, RawInput,
    TextureId,
};
use sfml::graphics::blend_mode::Factor;
use sfml::graphics::{
    BlendMode, Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Texture, Vertex,
};
use sfml::system::{Clock, Vector2};
use sfml::window::clipboard;
use sfml::{
    window::{mouse, Event, Key},
    SfBox,
};

pub use egui;
pub use sfml;

fn button_conv(button: mouse::Button) -> Option<PointerButton> {
    let but = match button {
        mouse::Button::Left => PointerButton::Primary,
        mouse::Button::Right => PointerButton::Secondary,
        mouse::Button::Middle => PointerButton::Middle,
        _ => return None,
    };
    Some(but)
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
        Key::F1 => EKey::F1,
        Key::F2 => EKey::F2,
        Key::F3 => EKey::F3,
        Key::F4 => EKey::F4,
        Key::F5 => EKey::F5,
        Key::F6 => EKey::F6,
        Key::F7 => EKey::F7,
        Key::F8 => EKey::F8,
        Key::F9 => EKey::F9,
        Key::F10 => EKey::F10,
        Key::F11 => EKey::F11,
        Key::F12 => EKey::F12,
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
            scan: _,
        } => {
            if ctrl {
                match code {
                    Key::V => raw_input
                        .events
                        .push(egui::Event::Text(clipboard::get_string())),
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
                    repeat: false,
                    physical_key: None,
                });
            }
        }
        Event::KeyReleased {
            code,
            alt,
            ctrl,
            shift,
            system: _,
            scan: _,
        } => {
            if let Some(key) = key_conv(code) {
                raw_input.events.push(egui::Event::Key {
                    key,
                    modifiers: modifier(alt, ctrl, shift),
                    pressed: false,
                    repeat: false,
                    physical_key: None,
                });
            }
        }
        Event::MouseMoved { x, y } => {
            raw_input
                .events
                .push(EguiEv::PointerMoved(Pos2::new(x as f32, y as f32)));
        }
        Event::MouseButtonPressed { x, y, button } => {
            if let Some(button) = button_conv(button) {
                raw_input.events.push(EguiEv::PointerButton {
                    pos: Pos2::new(x as f32, y as f32),
                    button,
                    pressed: true,
                    modifiers: Modifiers::default(),
                });
            }
        }
        Event::MouseButtonReleased { x, y, button } => {
            if let Some(button) = button_conv(button) {
                raw_input.events.push(EguiEv::PointerButton {
                    pos: Pos2::new(x as f32, y as f32),
                    button,
                    pressed: false,
                    modifiers: Modifiers::default(),
                });
            }
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
        Event::Resized { width, height } => {
            raw_input.screen_rect = Some(raw_input_screen_rect(width, height));
        }
        _ => {}
    }
}

/// Creates a `RawInput` that fits the window.
fn make_raw_input(window: &RenderWindow) -> RawInput {
    let Vector2 { x: w, y: h } = window.size();
    RawInput {
        screen_rect: Some(raw_input_screen_rect(w, h)),
        ..Default::default()
    }
}

fn raw_input_screen_rect(w: u32, h: u32) -> egui::Rect {
    egui::Rect {
        min: Pos2::new(0., 0.),
        max: Pos2::new(w as f32, h as f32),
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
    clock: SfBox<Clock>,
    ctx: Context,
    raw_input: RawInput,
    egui_result: FullOutput,
    textures: HashMap<TextureId, SfBox<Texture>>,
}

impl SfEgui {
    /// Create a new `SfEgui`.
    ///
    /// The size of the egui ui will be the same as `window`'s size.
    pub fn new(window: &RenderWindow) -> Self {
        Self {
            clock: sfml::system::Clock::start(),
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
    pub fn do_frame(&mut self, f: impl FnOnce(&Context)) -> Result<(), DoFrameError> {
        self.prepare_raw_input();
        self.egui_result = self.ctx.run(self.raw_input.take(), f);
        self.handle_output()
    }

    /// Alternative to `do_frame`. If you call this, it should be paired with `end_frame()`.
    pub fn begin_frame(&mut self) {
        self.prepare_raw_input();
        self.ctx.begin_frame(self.raw_input.take());
    }

    /// Alternative to `do_frame`. Call `begin_frame()` first.
    pub fn end_frame(&mut self) -> Result<(), DoFrameError> {
        self.egui_result = self.ctx.end_frame();
        self.handle_output()
    }

    fn handle_output(&mut self) -> Result<(), DoFrameError> {
        let clip_str = &self.egui_result.platform_output.copied_text;
        if !clip_str.is_empty() {
            clipboard::set_string(clip_str);
        }
        for (id, delta) in &self.egui_result.textures_delta.set {
            let [w, h] = delta.image.size();
            let tex = match self.textures.entry(*id) {
                Entry::Occupied(en) => en.into_mut(),
                Entry::Vacant(en) => {
                    let mut tex = Texture::new().unwrap();
                    if !tex.create(w as u32, h as u32) {
                        return Err(DoFrameError::TextureCreateError(TextureCreateError {
                            width: w,
                            height: h,
                        }));
                    }
                    en.insert(tex)
                }
            };
            update_tex_from_delta(tex, delta)?;
        }
        Ok(())
    }

    fn prepare_raw_input(&mut self) {
        self.raw_input.time = Some(self.clock.elapsed_time().as_seconds() as f64);
        // Update modifiers every frame, otherwise querying them (input.modifiers.*) doesn't seem
        // up-to-date
        self.raw_input.modifiers.alt = Key::LAlt.is_pressed() || Key::RAlt.is_pressed();
        self.raw_input.modifiers.ctrl = Key::LControl.is_pressed() || Key::RControl.is_pressed();
        self.raw_input.modifiers.shift = Key::LShift.is_pressed() || Key::RShift.is_pressed();
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
            mem::take(&mut self.egui_result.shapes),
            user_tex_src.unwrap_or(&mut DummyTexSource::default()),
            &self.textures,
            self.egui_result.pixels_per_point,
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

#[derive(Debug)]
/// Error when failing to create a texture
pub struct TextureCreateError {
    /// The width of the requested texture
    pub width: usize,
    /// The height of the requested texture
    pub height: usize,
}

/// Error that can happen when doing an egui frame
#[non_exhaustive]
#[derive(Debug)]
pub enum DoFrameError {
    /// Failed to create a texture
    TextureCreateError(TextureCreateError),
}

impl From<TextureCreateError> for DoFrameError {
    fn from(src: TextureCreateError) -> Self {
        Self::TextureCreateError(src)
    }
}

fn update_tex_from_delta(
    tex: &mut SfBox<Texture>,
    delta: &egui::epaint::ImageDelta,
) -> Result<(), TextureCreateError> {
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
        ImageData::Font(font_image) => {
            let srgba: Vec<u8> = font_image
                .srgba_pixels(None)
                .flat_map(|c32| c32.to_array())
                .collect();
            if w > tex.size().x as usize || h > tex.size().y as usize {
                // Resize texture
                let ok = tex.create(w as u32, h as u32);
                if !ok {
                    return Err(TextureCreateError {
                        width: w,
                        height: h,
                    });
                }
            }
            unsafe {
                tex.update_from_pixels(&srgba, w as u32, h as u32, x, y);
            }
        }
    }
    Ok(())
}

fn draw(
    window: &mut RenderWindow,
    egui_ctx: &egui::Context,
    shapes: Vec<egui::epaint::ClippedShape>,
    user_tex_source: &mut dyn UserTexSource,
    textures: &HashMap<TextureId, SfBox<Texture>>,
    pixels_per_point: f32,
) {
    window.set_active(true);
    unsafe {
        glu_sys::glEnable(glu_sys::GL_SCISSOR_TEST);
    }
    let mut vertices = Vec::new();
    for egui::ClippedPrimitive {
        clip_rect,
        primitive,
    } in egui_ctx.tessellate(shapes, pixels_per_point)
    {
        let mesh = match primitive {
            Primitive::Mesh(mesh) => mesh,
            _ => continue,
        };
        let (tw, th, tex) = match mesh.texture_id {
            TextureId::Managed(id) => {
                let tex = &*textures[&TextureId::Managed(id)];
                let (egui_tex_w, egui_tex_h) = (tex.size().x as f32, tex.size().y as f32);
                (egui_tex_w, egui_tex_h, tex)
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
        rs.blend_mode = BlendMode {
            color_src_factor: Factor::One,
            color_dst_factor: Factor::OneMinusSrcAlpha,
            alpha_src_factor: Factor::OneMinusDstAlpha,
            alpha_dst_factor: Factor::One,
            ..Default::default()
        };
        rs.set_texture(Some(tex));
        let pixels_per_point = 1.;
        let win_size = window.size();
        let width_in_pixels = win_size.x;
        let height_in_pixels = win_size.y;
        // Code copied from egui_glium (https://github.com/emilk/egui)
        // Transform clip rect to physical pixels:
        let clip_min_x = pixels_per_point * clip_rect.min.x;
        let clip_min_y = pixels_per_point * clip_rect.min.y;
        let clip_max_x = pixels_per_point * clip_rect.max.x;
        let clip_max_y = pixels_per_point * clip_rect.max.y;

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
