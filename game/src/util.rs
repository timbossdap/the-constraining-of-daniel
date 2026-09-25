//! Small shared helpers: colors, rect builders, UI/window plumbing.

use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector2, Vector3},
        color::Color,
        pool::Handle,
    },
    engine::GraphicsContext,
    keyboard::KeyCode,
    plugin::PluginContext,
    material::MaterialResource,
    scene::{
        base::BaseBuilder,
        dim2::rectangle::{Rectangle, RectangleBuilder},
        graph::Graph,
        transform::TransformBuilder,
    },
};
pub(crate) fn col(rgb: (u8, u8, u8)) -> Color {
    Color::opaque(rgb.0, rgb.1, rgb.2)
}

#[allow(dead_code)]
pub(crate) fn cola(rgb: (u8, u8, u8), a: u8) -> Color {
    Color::from_rgba(rgb.0, rgb.1, rgb.2, a)
}

pub(crate) fn darken(rgb: (u8, u8, u8), f: f32) -> (u8, u8, u8) {
    (
        (rgb.0 as f32 * f) as u8,
        (rgb.1 as f32 * f) as u8,
        (rgb.2 as f32 * f) as u8,
    )
}

pub(crate) fn lighten(rgb: (u8, u8, u8), amt: u8) -> (u8, u8, u8) {
    (
        rgb.0.saturating_add(amt),
        rgb.1.saturating_add(amt),
        rgb.2.saturating_add(amt),
    )
}

#[allow(dead_code)]
pub(crate) fn make_rect(graph: &mut Graph, name: &str, pos: (f32, f32), scale: (f32, f32), color: Color) -> Handle<Rectangle> {
    make_rect_z(graph, name, pos, scale, color, 0.0)
}

pub(crate) fn make_rect_z(graph: &mut Graph, name: &str, pos: (f32, f32), scale: (f32, f32), color: Color, z: f32) -> Handle<Rectangle> {
    let t = TransformBuilder::new()
        .with_local_position(Vector3::new(pos.0, pos.1, z))
        .with_local_scale(Vector3::new(scale.0, scale.1, 1.0))
        .build();
    RectangleBuilder::new(
        BaseBuilder::new()
            .with_name(name)
            .with_local_transform(t)
            .with_frustum_culling(false),
    )
    .with_color(color)
    .build(graph)
}

/// Textured twin of [`make_rect_z`]: same opaque rect, but sampling a
/// material texture (multiplied by vertex color) for surface detail.
pub(crate) fn make_rect_mat(graph: &mut Graph, name: &str, pos: (f32, f32), scale: (f32, f32), color: Color, z: f32, mat: &MaterialResource) -> Handle<Rectangle> {
    let t = TransformBuilder::new()
        .with_local_position(Vector3::new(pos.0, pos.1, z))
        .with_local_scale(Vector3::new(scale.0, scale.1, 1.0))
        .build();
    RectangleBuilder::new(
        BaseBuilder::new()
            .with_name(name)
            .with_local_transform(t)
            .with_frustum_culling(false),
    )
    .with_color(color)
    .with_material(mat.clone())
    .build(graph)
}

pub(crate) fn set_pos(r: &mut Rectangle, x: f32, y: f32, z: f32) {
    r.local_transform_mut().set_position(Vector3::new(x, y, z));
}

pub(crate) fn set_rot(r: &mut Rectangle, angle: f32) {
    r.local_transform_mut()
        .set_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), angle));
}

pub(crate) fn set_scale(r: &mut Rectangle, sx: f32, sy: f32) {
    r.local_transform_mut().set_scale(Vector3::new(sx, sy, 1.0));
}

pub(crate) fn window_size(ctx: &PluginContext) -> Vector2<f32> {
    match ctx.graphics_context {
        GraphicsContext::Initialized(ref g) => {
            let s = g.window.inner_size();
            Vector2::new(s.width as f32, s.height as f32)
        }
        _ => Vector2::new(1280.0, 720.0),
    }
}

/// UI scale factor from window size (1.0 at 1280x720). All fonts and fixed-px
/// widget geometry multiply by this so the HUD reads on any monitor.
pub(crate) fn ui_scale(ctx: &PluginContext) -> f32 {
    let s = window_size(ctx);
    (s.x / 1280.0).min(s.y / 720.0).clamp(0.6, 2.0)
}

pub(crate) fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

pub(crate) fn pressed(input: &fyrox::engine::input::InputState, code: KeyCode, prev: &[KeyCode]) -> bool {
    input.is_key_down(code) && !prev.contains(&code)
}

/// Mouse for menus: hovering does NOTHING (no accidental selection);
/// a fresh left-click both selects and confirms the row under the cursor.
/// Rects are (x, y, w, h) in UI px — pass the same numbers the sync fns use.
/// `clicked` is the once-per-frame fresh-press edge (see update()).
/// NOTE: assumes UI px == window px (true at 100% display scaling).
pub(crate) fn mouse_list(
    input: &fyrox::engine::input::InputState,
    rects: &[(f32, f32, f32, f32)],
    sel: &mut usize,
    clicked: bool,
) -> Option<usize> {
    if !clicked {
        return None;
    }
    let mp = input.mouse_position();
    let (mx, my) = (mp.x, mp.y);
    for (i, &(x, y, w, h)) in rects.iter().enumerate() {
        if mx >= x && mx <= x + w && my >= y && my <= y + h {
            *sel = i;
            return Some(i);
        }
    }
    None
}
