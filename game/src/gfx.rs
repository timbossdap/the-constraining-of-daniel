//! Procedural world textures: everything is generated pixel-by-pixel at
//! runtime (no image assets). The standard 2D shader multiplies vertex color
//! by the diffuse texture, so bright grayscale detail lets every tint shine
//! through while killing the flat-vector look.

use fyrox::{
    asset::untyped::ResourceKind,
    core::uuid::Uuid,
    gui::texture::{Texture, TextureKind, TexturePixelKind, TextureResource},
    material::{Material, MaterialResource},
};

use crate::zone::Zone;

const TEX: i32 = 128;

/// Tiny deterministic RNG (xorshift) so textures are stable per boot.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        if x == 0 {
            x = 0x9E3779B97F4A7C15;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: i32) -> i32 {
        (self.next() % n as u64) as i32
    }
}

fn put(px: &mut [u8], size: i32, x: i32, y: i32, v: u8) {
    if x >= 0 && y >= 0 && x < size && y < size {
        let i = ((y * size + x) * 4) as usize;
        px[i] = v;
        px[i + 1] = v;
        px[i + 2] = v;
        px[i + 3] = 255;
    }
}

fn to_texture(size: i32, px: Vec<u8>) -> TextureResource {
    let tex = Texture::from_bytes(
        TextureKind::Rectangle {
            width: size as u32,
            height: size as u32,
        },
        TexturePixelKind::RGBA8,
        px,
    )
    .expect("gfx texture byte size");
    TextureResource::new_ok(Uuid::new_v4(), ResourceKind::Embedded, tex)
}

/// Neutral film grain shared by every entity: white base, per-pixel noise,
/// sparse dark speckles. Multiplied by vertex color it reads as surface.
pub(crate) fn grain_texture() -> TextureResource {
    let mut rng = Rng(0xC0FFEE);
    let mut px = vec![255u8; (TEX * TEX * 4) as usize];
    for y in 0..TEX {
        for x in 0..TEX {
            let n = 228 + rng.below(28) as u8 - 14;
            let i = ((y * TEX + x) * 4) as usize;
            px[i] = n;
            px[i + 1] = n;
            px[i + 2] = n;
            if rng.below(220) == 0 {
                let d = 150 + rng.below(40) as u8;
                px[i] = d;
                px[i + 1] = d;
                px[i + 2] = d;
            }
        }
    }
    to_texture(TEX, px)
}

/// Ground detail per zone, kept near-white so the checker palette dominates:
/// meadow = grass blades, cinder = cracks, frost = crystal veins.
pub(crate) fn ground_texture(zone: Zone) -> TextureResource {
    let mut rng = Rng(0x5EED + zone as u64 * 7919);
    let mut px = vec![255u8; (TEX * TEX * 4) as usize];
    // soft blotches so tiles don't read as flat color
    for _ in 0..26 {
        let cx = rng.below(TEX);
        let cy = rng.below(TEX);
        let r = 6 + rng.below(14);
        let v = 225 + rng.below(20) as u8;
        for oy in -r..=r {
            for ox in -r..=r {
                if ox * ox + oy * oy <= r * r && rng.below(3) > 0 {
                    put(&mut px, TEX, cx + ox, cy + oy, v);
                }
            }
        }
    }
    match zone {
        Zone::Meadow => {
            // grass blades: short darker vertical strokes
            for _ in 0..130 {
                let x = rng.below(TEX);
                let y = rng.below(TEX);
                let len = 3 + rng.below(4);
                let v = 195 + rng.below(30) as u8;
                for k in 0..len {
                    put(&mut px, TEX, x + if k > 2 { 1 } else { 0 }, y + k, v);
                }
            }
        }
        Zone::CinderCaves => {
            // cracks: dark random walks
            for _ in 0..14 {
                let mut x = rng.below(TEX);
                let mut y = rng.below(TEX);
                for _ in 0..22 {
                    put(&mut px, TEX, x, y, 165);
                    put(&mut px, TEX, x + 1, y, 185);
                    x += rng.below(3) - 1;
                    y += rng.below(3) - 1;
                }
            }
            // embers: a few bright dots
            for _ in 0..24 {
                put(&mut px, TEX, rng.below(TEX), rng.below(TEX), 255);
            }
        }
        Zone::FrostKeep => {
            // crystal veins: pale diagonal strokes + sparkle dots
            for _ in 0..40 {
                let x = rng.below(TEX);
                let y = rng.below(TEX);
                let len = 4 + rng.below(6);
                for k in 0..len {
                    put(&mut px, TEX, x + k, y + k / 2, 210);
                }
            }
            for _ in 0..60 {
                put(&mut px, TEX, rng.below(TEX), rng.below(TEX), 245);
            }
        }
    }
    to_texture(TEX, px)
}

/// Zone -> ground material slot (Meadow 0, Cinder 1, Frost 2).
pub(crate) fn zone_index(zone: Zone) -> usize {
    match zone {
        Zone::Meadow => 0,
        Zone::CinderCaves => 1,
        Zone::FrostKeep => 2,
    }
}

pub(crate) fn material_for(tex: &TextureResource) -> MaterialResource {
    let mut m = Material::standard_2d();
    // the standard 2D shader declares a diffuseTexture sampler binding;
    // point it at our generated texture (vertex color still multiplies).
    if let Some(binding) = m.texture_mut("diffuseTexture") {
        binding.value = Some(tex.clone());
    }
    MaterialResource::new_ok(Uuid::new_v4(), ResourceKind::Embedded, m)
}
