//! Stitched leather-patch panels: the burnt-orange dialog look, generated
//! pixel-by-pixel at runtime (no image assets) and scaled to any panel size
//! with NinePatch so corners + stitches never distort.

use fyrox::{
    asset::untyped::ResourceKind,
    core::{math::Rect, pool::Handle, uuid::Uuid},
    gui::{
        BuildContext,
        nine_patch::{NinePatch, NinePatchBuilder},
        texture::{
            Texture, TextureKind, TextureMagnificationFilter, TexturePixelKind,
            TextureResource,
        },
        widget::WidgetBuilder,
    },
};

/// Burnt-orange patch fill (sampled from the reference).
pub(crate) const PATCH_ORANGE: (u8, u8, u8) = (198, 116, 42);
/// Darker stitch dashes.
pub(crate) const PATCH_STITCH: (u8, u8, u8) = (146, 74, 22);
/// Gold stitches for the selected card / row.
pub(crate) const PATCH_GOLD: (u8, u8, u8) = (255, 215, 90);
/// Dark chocolate text on orange.
pub(crate) const PATCH_INK: (u8, u8, u8) = (58, 30, 16);
/// Cream text for selected entries on orange.
pub(crate) const PATCH_CREAM: (u8, u8, u8) = (255, 238, 205);
/// Heavily-dimmed variant (upgrade screen): dark fill, dark stitches.
/// Kept readable, not pitch — the unselected-card twins were too black.
pub(crate) const PATCH_DIM_FILL: (u8, u8, u8) = (105, 62, 30);
pub(crate) const PATCH_DIM_STITCH: (u8, u8, u8) = (75, 45, 22);

const SIZE: i32 = 128;
const RADIUS: i32 = 24;
const INSET: i32 = 11;
const DASH: i32 = 10;
const GAP: i32 = 8;
const STITCH_R: i32 = 2;

fn put(px: &mut [u8], x: i32, y: i32, c: (u8, u8, u8)) {
    if x >= 0 && y >= 0 && x < SIZE && y < SIZE {
        let i = ((y * SIZE + x) * 4) as usize;
        px[i..i + 4].copy_from_slice(&[c.0, c.1, c.2, 255]);
    }
}

fn dash_h(px: &mut [u8], x0: i32, y: i32, stitch: (u8, u8, u8)) {
    for x in x0..x0 + DASH {
        for oy in -STITCH_R..=STITCH_R {
            put(px, x, y + oy, stitch);
        }
    }
}

fn dash_v(px: &mut [u8], x: i32, y0: i32, stitch: (u8, u8, u8)) {
    for y in y0..y0 + DASH {
        for ox in -STITCH_R..=STITCH_R {
            put(px, x + ox, y, stitch);
        }
    }
}

fn dot(px: &mut [u8], x: i32, y: i32, stitch: (u8, u8, u8)) {
    for oy in -STITCH_R..=STITCH_R {
        for ox in -STITCH_R..=STITCH_R {
            put(px, x + ox, y + oy, stitch);
        }
    }
}

/// Draws the patch (fill + dashed stitch rounded path) and wraps it as an
/// embedded texture resource. Both fill and stitch are parameters so dimmed
/// and gold-selected variants share the same stitching.
pub(crate) fn patch_texture(fill: (u8, u8, u8), stitch: (u8, u8, u8)) -> TextureResource {
    let mut px = vec![0u8; (SIZE * SIZE * 4) as usize];
    // rounded-rect fill
    for y in 0..SIZE {
        for x in 0..SIZE {
            let cx = x.clamp(RADIUS, SIZE - 1 - RADIUS);
            let cy = y.clamp(RADIUS, SIZE - 1 - RADIUS);
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= RADIUS * RADIUS {
                put(&mut px, x, y, fill);
            }
        }
    }
    // straight stitch runs
    let mut x = RADIUS + 4;
    while x < SIZE - RADIUS - 4 - DASH {
        dash_h(&mut px, x, INSET, stitch);
        dash_h(&mut px, x, SIZE - 1 - INSET, stitch);
        x += DASH + GAP;
    }
    let mut y = RADIUS + 4;
    while y < SIZE - RADIUS - 4 - DASH {
        dash_v(&mut px, INSET, y, stitch);
        dash_v(&mut px, SIZE - 1 - INSET, y, stitch);
        y += DASH + GAP;
    }
    // corner arcs (axis-aligned dots along the inset curve)
    let r = (RADIUS - INSET) as f32;
    for (ccx, ccy, a0) in [
        (RADIUS, RADIUS, 180.0),
        (SIZE - 1 - RADIUS, RADIUS, 270.0),
        (SIZE - 1 - RADIUS, SIZE - 1 - RADIUS, 0.0),
        (RADIUS, SIZE - 1 - RADIUS, 90.0),
    ] {
        let mut a: f32 = a0;
        while a < a0 + 90.0 {
            let rad = a.to_radians();
            dot(
                &mut px,
                ccx + (r * rad.cos()) as i32,
                ccy + (r * rad.sin()) as i32,
                stitch,
            );
            a += 15.0;
        }
    }
    let tex = Texture::from_bytes(
        TextureKind::Rectangle {
            width: SIZE as u32,
            height: SIZE as u32,
        },
        TexturePixelKind::RGBA8,
        px,
    )
    .expect("patch texture byte size");
    TextureResource::new_ok(Uuid::new_v4(), ResourceKind::Embedded, tex)
}

/// Builds a NinePatch panel showing `tex`, sized `w`×`h` px.
pub(crate) fn patch_panel(
    ctx: &mut BuildContext,
    tex: &TextureResource,
    w: f32,
    h: f32,
) -> Handle<NinePatch> {
    NinePatchBuilder::new(WidgetBuilder::new().with_width(w).with_height(h))
        .with_texture(tex.clone())
        // our texture is 128x128 — the builder defaults to a 200x200 region,
        // which sampled out of bounds and smeared the stitches into streaks
        .with_texture_region(Rect::new(0, 0, SIZE as u32, SIZE as u32))
        .with_top_margin(RADIUS as u32)
        .with_bottom_margin(RADIUS as u32)
        .with_left_margin(RADIUS as u32)
        .with_right_margin(RADIUS as u32)
        .with_draw_center(true)
        .build(ctx)
}

/// Pixel-art heart containers (Isaac-style HP): full, half, empty.
/// 7x6 pattern, doubled to 14x12, crisp nearest-neighbor sampling.
const HEART_ROWS: [&str; 6] = [
    ".XX.XX.",
    "XXXXXXX",
    "XXXXXXX",
    ".XXXXX.",
    "..XXX..",
    "...X...",
];
const HEART_FULL: (u8, u8, u8) = (220, 40, 50);
const HEART_EMPTY: (u8, u8, u8) = (62, 24, 30);
const HEART_EDGE: (u8, u8, u8) = (120, 20, 28);
const HW: i32 = 14;
const HH: i32 = 12;

fn heart_texture(mode: u8) -> TextureResource {
    heart_texture_shaded(mode, false)
}

/// Dark twin of a heart sprite for the dimmed HUD (pause/draft overlays).
fn dark_heart_texture(mode: u8) -> TextureResource {
    heart_texture_shaded(mode, true)
}

fn shade(c: (u8, u8, u8)) -> (u8, u8, u8) {
    ((c.0 as f32 * 0.45) as u8, (c.1 as f32 * 0.45) as u8, (c.2 as f32 * 0.45) as u8)
}

fn heart_texture_shaded(mode: u8, dark: bool) -> TextureResource {
    // mode: 0 = full, 1 = half (left red), 2 = empty
    let mut px = vec![0u8; (HW * HH * 4) as usize];
    let mut shape = [[false; HW as usize]; HH as usize];
    let mut full = [[false; HW as usize]; HH as usize];
    for (y, row) in HEART_ROWS.iter().enumerate() {
        for (x, ch) in row.bytes().enumerate() {
            if ch == b'X' {
                for oy in 0..2 {
                    for ox in 0..2 {
                        let gx = (x * 2 + ox) as i32;
                        let gy = (y * 2 + oy) as i32;
                        if gx < HW && gy < HH {
                            shape[gy as usize][gx as usize] = true;
                            full[gy as usize][gx as usize] = gx < HW / 2;
                        }
                    }
                }
            }
        }
    }
    for y in 0..HH {
        for x in 0..HW {
            if !shape[y as usize][x as usize] {
                continue;
            }
            // edge pass: any transparent 4-neighbor -> outline color
            let mut edge = false;
            for (ox, oy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = x + ox;
                let ny = y + oy;
                if nx < 0 || ny < 0 || nx >= HW || ny >= HH || !shape[ny as usize][nx as usize] {
                    edge = true;
                    break;
                }
            }
            let mut c = if edge {
                HEART_EDGE
            } else {
                match mode {
                    0 => HEART_FULL,
                    1 => {
                        if full[y as usize][x as usize] {
                            HEART_FULL
                        } else {
                            HEART_EMPTY
                        }
                    }
                    _ => HEART_EMPTY,
                }
            };
            if dark {
                c = shade(c);
            }
            let i = ((y * HW + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&[c.0, c.1, c.2, 255]);
        }
    }
    let mut tex = Texture::from_bytes(
        TextureKind::Rectangle {
            width: HW as u32,
            height: HH as u32,
        },
        TexturePixelKind::RGBA8,
        px,
    )
    .expect("heart texture byte size");
    tex.set_magnification_filter(TextureMagnificationFilter::Nearest);
    TextureResource::new_ok(Uuid::new_v4(), ResourceKind::Embedded, tex)
}

/// Returns (full, half, empty) heart textures.
pub(crate) fn heart_textures() -> (TextureResource, TextureResource, TextureResource) {
    (heart_texture(0), heart_texture(1), heart_texture(2))
}

/// Dark twins for the dimmed HUD.
pub(crate) fn dark_heart_textures() -> (TextureResource, TextureResource, TextureResource) {
    (
        dark_heart_texture(0),
        dark_heart_texture(1),
        dark_heart_texture(2),
    )
}
