//! egui SFML integration helpers
//!
//! Contains various types and functions that helps with integrating egui with SFML.

#![warn(missing_docs)]

use egui::{Event as EguiEv, Modifiers, PointerButton, Pos2, RawInput, TextureId};
use sfml::graphics::blend_mode::Factor;
use sfml::graphics::{
    BlendMode, Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Texture, Vertex,
};
use sfml::{
    window::{mouse, Event, Key},
    SfBox,
};

fn button_conv(button: mouse::Button) -> PointerButton {
    match button {
        mouse::Button::LEFT => PointerButton::Primary,
        mouse::Button::RIGHT => PointerButton::Secondary,
        mouse::Button::MIDDLE => PointerButton::Middle,
        _ => panic!("Unhandled pointer button: {:?}", button),
    }
}

fn key_conv(code: Key) -> Option<egui::Key> {
    use egui::Key as EKey;
    Some(match code {
        Key::DOWN => EKey::ArrowDown,
        Key::LEFT => EKey::ArrowLeft,
        Key::RIGHT => EKey::ArrowRight,
        Key::UP => EKey::ArrowUp,
        Key::ESCAPE => EKey::Escape,
        Key::TAB => EKey::Tab,
        Key::BACKSPACE => EKey::Backspace,
        Key::ENTER => EKey::Enter,
        Key::SPACE => EKey::Space,
        Key::INSERT => EKey::Insert,
        Key::DELETE => EKey::Delete,
        Key::HOME => EKey::Home,
        Key::END => EKey::End,
        Key::PAGEUP => EKey::PageUp,
        Key::PAGEDOWN => EKey::PageDown,
        Key::NUM0 => EKey::Num0,
        Key::NUM1 => EKey::Num1,
        Key::NUM2 => EKey::Num2,
        Key::NUM3 => EKey::Num3,
        Key::NUM4 => EKey::Num4,
        Key::NUM5 => EKey::Num5,
        Key::NUM6 => EKey::Num6,
        Key::NUM7 => EKey::Num7,
        Key::NUM8 => EKey::Num8,
        Key::NUM9 => EKey::Num9,
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
pub fn handle_event(raw_input: &mut egui::RawInput, event: &sfml::window::Event) {
    match *event {
        Event::KeyPressed {
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
        _ => {}
    }
}

/// Creates a `RawInput` that fits the window.
pub fn make_raw_input(window: &RenderWindow) -> RawInput {
    RawInput {
        screen_rect: Some(egui::Rect {
            min: Pos2::new(0., 0.),
            max: Pos2::new(window.size().x as f32, window.size().y as f32),
        }),
        ..Default::default()
    }
}

fn egui_tex_to_rgba_vec(tex: &egui::Texture) -> Vec<u8> {
    let srgba = tex.srgba_pixels(1.0);
    let mut vec = Vec::new();
    for c in srgba {
        vec.extend_from_slice(&c.to_array());
    }
    vec
}

/// Update the texture every frame with this
pub fn get_new_texture(ctx: &egui::CtxRef) -> SfBox<Texture> {
    let egui_tex = ctx.texture();
    let mut tex = Texture::new().unwrap();
    if !tex.create(egui_tex.width as u32, egui_tex.height as u32) {
        panic!("Failed to create texture");
    }
    let tex_pixels = egui_tex_to_rgba_vec(&egui_tex);
    unsafe {
        tex.update_from_pixels(
            &tex_pixels,
            egui_tex.width as u32,
            egui_tex.height as u32,
            0,
            0,
        );
    }
    tex
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

/// Draw the egui ui using a `RenderWindow`.
///
/// # Parameters
///
/// - `window`: The `RenderWindow` to draw to.
/// - `egui_ctx`: The egui context
/// - `tex`: The egui texture that contains the font, etc.
/// - `shapes`: The shapes contained by the output of `egui_ctx.end_frame()`/
/// - `user_tex_source`: This is used to get the texture for a user-defined texture.
///   See [`UserTexSource`].
pub fn draw<TexSrc: UserTexSource>(
    window: &mut RenderWindow,
    egui_ctx: &egui::CtxRef,
    shapes: Vec<egui::epaint::ClippedShape>,
    user_tex_source: &mut TexSrc,
) {
    let tex = get_new_texture(egui_ctx);
    window.set_active(true);
    unsafe {
        glu_sys::glEnable(glu_sys::GL_SCISSOR_TEST);
    }
    let mut vertices = Vec::new();
    let (egui_tex_w, egui_tex_h) = (tex.size().x as f32, tex.size().y as f32);
    for egui::ClippedMesh(rect, mesh) in egui_ctx.tessellate(shapes) {
        let (tw, th, tex) = match mesh.texture_id {
            TextureId::Egui => (egui_tex_w, egui_tex_h, &*tex),
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
