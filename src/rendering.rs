use {
    crate::{TextureCreateError, UserTexSource},
    egui::{epaint::Primitive, ImageData, TextureId},
    sfml::{
        cpp::FBox,
        graphics::{
            blend_mode::Factor, BlendMode, Color, PrimitiveType, RenderStates, RenderTarget as _,
            RenderWindow, Texture, Vertex,
        },
    },
    std::collections::HashMap,
};

pub(super) fn update_tex_from_delta(
    tex: &mut Texture,
    delta: &egui::epaint::ImageDelta,
) -> Result<(), TextureCreateError> {
    let [w, h] = delta.image.size();
    let [x, y] = delta.pos.map_or([0, 0], |[x, y]| [x as u32, y as u32]);
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
                let ok = tex.create(w as u32, h as u32).is_ok();
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

pub(super) fn draw(
    window: &mut RenderWindow,
    egui_ctx: &egui::Context,
    shapes: Vec<egui::epaint::ClippedShape>,
    user_tex_source: &mut dyn UserTexSource,
    textures: &HashMap<TextureId, FBox<Texture>>,
    pixels_per_point: f32,
) {
    let _ = window.set_active(true);
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
        let rs = RenderStates {
            blend_mode: BlendMode {
                color_src_factor: Factor::One,
                color_dst_factor: Factor::OneMinusSrcAlpha,
                alpha_src_factor: Factor::OneMinusDstAlpha,
                alpha_dst_factor: Factor::One,
                ..Default::default()
            },
            texture: Some(tex),
            ..Default::default()
        };
        window.draw_primitives(&vertices, PrimitiveType::TRIANGLES, &rs);
        vertices.clear();
    }
    unsafe {
        glu_sys::glDisable(glu_sys::GL_SCISSOR_TEST);
    }
    let _ = window.set_active(false);
}
