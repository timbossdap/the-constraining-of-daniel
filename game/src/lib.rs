//! the constraining of daniel — 2D Action RPG in Fyrox.
//!
//! 8 classes, 50 levels, 5 stats + allocation, 3 zones, elites/bosses,
//! combos/loot/shrines for fun > grind, procedural 2D visuals + HUD.
//!
//! Module map:
//! - `sim` — headless simulation (player, enemies, arrows, XP, loot)
//! - `draft` — level-up 3-card upgrade picks
//! - `render` — scene building + per-frame world sync
//! - `hud` — HUD, menu, draft cards, floating text
//! - `gfx` — procedural world textures + materials
//! - `panel` — generated stitched-patch backgrounds
//! - `plugin` — Fyrox `Game` plugin + update loop
//! - `player_class` — 8 classes × 4 skills
//! - `weapon_list` — loot rarities + 48 named weapons
//! - `combat` — damage, crit, hit math
//! - `progression` — XP curve, levels, stat allocation
//! - `zone` / `enemy` / `pickup` — world content
//! - `stats` — attributes + derived stats
//! - `util` — colors, rect builders, small helpers
//!
//! Controls: WASD move · Arrows/Space shoot (only arrows hurt) · 1-4/QER skills
//! Z/X/C/V/B stats · T auto · H potion · F1-F8 class · Enter portal/menu confirm.

pub mod combat;
pub mod draft;
pub mod enemy;
pub mod gfx;
pub mod hud;
pub mod gu;
pub mod panel;
pub mod pickup;
pub mod player_class;
pub mod plugin;
pub mod progression;
pub mod render;
pub mod sim;
pub mod stats;
pub mod story;
pub mod synergy;
pub mod util;
pub mod weapon_list;
pub mod zone;

pub use fyrox;
pub use plugin::Game;

