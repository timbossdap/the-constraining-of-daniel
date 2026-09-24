//! Fyrox plugin glue: the `Game` node, its update loop, input, and states
//! (menu → playing → level-up draft).

use fyrox::{
    core::{pool::Handle, reflect::prelude::*, visitor::prelude::*},
    event::Event,
    graph::SceneGraph,
    gui::{border::Border, image::Image, message::UiMessage, nine_patch::NinePatch, text::Text, texture::TextureResource, UserInterface},
    keyboard::KeyCode,
    material::MaterialResource,
    plugin::{Plugin, PluginContext, PluginRegistrationContext, error::GameResult},
    scene::{camera::Camera, dim2::rectangle::Rectangle, Scene},
};
use crate::{
    combat::player_attack_damage,
    items::{ITEMS, random_item_of, roll_item_rarity},
    pickup::PickupKind,
    player_class::CharacterClass,
    progression::PlayerCore,
    sim::{FloatText, GameInner, Projectile},
    stats::AttrKind,
    util::{dist, pressed},
    weapon_list::{Weapon, roll_loot},
    zone::Zone,
};

#[derive(Visit, Reflect, Debug)]
#[reflect(non_cloneable)]
pub struct Game {
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) inner: GameInner,
    pub(crate) scene: Handle<Scene>,
    pub(crate) player_handle: Handle<Rectangle>,
    pub(crate) player_belly: Handle<Rectangle>,
    pub(crate) player_pupil_l: Handle<Rectangle>,
    pub(crate) player_pupil_r: Handle<Rectangle>,
    pub(crate) player_boot_l: Handle<Rectangle>,
    pub(crate) player_boot_r: Handle<Rectangle>,
    pub(crate) player_arm_l: Handle<Rectangle>,
    pub(crate) player_arm_r: Handle<Rectangle>,
    pub(crate) camera_handle: Handle<Camera>,
    pub(crate) player_outline: Handle<Rectangle>,
    pub(crate) player_shadow: Handle<Rectangle>,
    pub(crate) player_glow: Handle<Rectangle>,
    pub(crate) player_eyes: Handle<Rectangle>,
    pub(crate) player_weapon: Handle<Rectangle>,
    pub(crate) player_hp_bg: Handle<Rectangle>,
    pub(crate) player_hp_fg: Handle<Rectangle>,
    pub(crate) player_shield: Handle<Rectangle>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) enemy_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) enemy_face_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) enemy_hp_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) enemy_glow_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) enemy_outline_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) enemy_belly_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) enemy_shadow_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) enemy_pupil_l_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) enemy_pupil_r_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) pickup_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) pickup_accent_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) pickup_glow_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) proj_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) proj_glow_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) ground_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) grain_mat: MaterialResource,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) ground_mats: [MaterialResource; 3],
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) ground_zone: Zone,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) decor_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) particle_handles: Vec<Handle<Rectangle>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) slash_segs: Vec<Handle<Rectangle>>,
    pub(crate) hud_line: Handle<Text>,
    // Isaac-style HUD: heart containers, left stats, bottom bars, minimap
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) heart_full: TextureResource,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) heart_half: TextureResource,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) heart_empty: TextureResource,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) hearts: Vec<Handle<Image>>,
    pub(crate) heart_sig: String,
    // dim twins: shown over HUD panels + hearts while paused / drafting
    pub(crate) stats_dim: Handle<NinePatch>,
    pub(crate) skills_dim: Handle<NinePatch>,
    pub(crate) map_dim: Handle<NinePatch>,
    pub(crate) items_dim: Handle<NinePatch>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) heart_darks: Vec<Handle<Image>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) heart_d_full: TextureResource,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) heart_d_half: TextureResource,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) heart_d_empty: TextureResource,
    pub(crate) hud_dim_on: bool,
    pub(crate) left_stats_panel: Handle<NinePatch>,
    pub(crate) left_stats_text: Handle<Text>,
    // reference-style skill panel: class header, name + [key]/unlock rows
    pub(crate) sk_panel: Handle<NinePatch>,
    pub(crate) sk_title: Handle<Text>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) sk_names: Vec<Handle<Text>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) sk_keys: Vec<Handle<Text>>,
    pub(crate) items_panel: Handle<NinePatch>,
    pub(crate) items_text: Handle<Text>,
    pub(crate) map_panel: Handle<NinePatch>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) map_cells: Vec<Handle<Border>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) map_cache: Vec<(u8, u8, u8)>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) map_sig: String,
    pub(crate) log_h: Handle<Text>,
    pub(crate) log_bg: Handle<NinePatch>,
    pub(crate) log_dim: Handle<NinePatch>,
    pub(crate) banner_h: Handle<Text>,
    pub(crate) banner_timer: f32,
    pub(crate) banner_shown: bool,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) patch_tex: TextureResource,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) patch_tex_gold: TextureResource,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) patch_tex_dark: TextureResource,
    pub(crate) menu_title: Handle<Text>,
    pub(crate) menu_sub: Handle<Text>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) menu_rows: Vec<Handle<Text>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) menu_row_bgs: Vec<Handle<NinePatch>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) menu_row_golds: Vec<Handle<NinePatch>>,
    pub(crate) menu_help: Handle<Text>,
    pub(crate) menu_help_bg: Handle<NinePatch>,
    pub(crate) menu_footer: Handle<Text>,
    pub(crate) menu_index: usize,
    pub(crate) menu_help_on: bool,
    pub(crate) menu_t: f32,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) menu_sig: String,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) float_labels: Vec<Handle<Text>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) float_cache: Vec<String>,
    pub(crate) xp_bar_bg: Handle<Border>,
    pub(crate) xp_bar_fg: Handle<Border>,
    pub(crate) xpbar_on: bool,
    pub(crate) hud_ui_on: bool,
    pub(crate) font_scale: f32,
    pub(crate) draft_title: Handle<Text>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) draft_cards_ui: Vec<Handle<Text>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) draft_panels: Vec<Handle<NinePatch>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) draft_golds: Vec<Handle<NinePatch>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) draft_darks: Vec<Handle<NinePatch>>,
    pub(crate) draft_hint: Handle<Text>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) draft_sig: String,
    pub(crate) stats_open: bool,
    pub(crate) stats_sel: usize,
    pub(crate) stat_repeat_plus: f32,
    pub(crate) stat_repeat_minus: f32,
    pub(crate) stats_panel: Handle<NinePatch>,
    pub(crate) stats_title: Handle<Text>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) stats_rows: Vec<Handle<Text>>,
    pub(crate) stats_hint: Handle<Text>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) stats_sig: String,
    pub(crate) paused: bool,
    pub(crate) pause_sel: usize,
    pub(crate) pause_title: Handle<Text>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) pause_rows: Vec<Handle<Text>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) pause_row_bgs: Vec<Handle<NinePatch>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) pause_row_golds: Vec<Handle<NinePatch>>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) pause_darks: Vec<Handle<NinePatch>>,
    pub(crate) pause_hint: Handle<Text>,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) pause_sig: String,
    #[visit(skip)]
    #[reflect(hidden)]
    pub(crate) initialized: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            inner: GameInner::default(),
            scene: Handle::NONE,
            player_handle: Handle::NONE,
            player_belly: Handle::NONE,
            player_pupil_l: Handle::NONE,
            player_pupil_r: Handle::NONE,
            player_boot_l: Handle::NONE,
            player_boot_r: Handle::NONE,
            player_arm_l: Handle::NONE,
            player_arm_r: Handle::NONE,
            camera_handle: Handle::NONE,
            player_outline: Handle::NONE,
            player_shadow: Handle::NONE,
            player_glow: Handle::NONE,
            player_eyes: Handle::NONE,
            player_weapon: Handle::NONE,
            player_hp_bg: Handle::NONE,
            player_hp_fg: Handle::NONE,
            player_shield: Handle::NONE,
            enemy_handles: Vec::new(),
            enemy_face_handles: Vec::new(),
            enemy_hp_handles: Vec::new(),
            enemy_glow_handles: Vec::new(),
            enemy_outline_handles: Vec::new(),
            enemy_belly_handles: Vec::new(),
            enemy_shadow_handles: Vec::new(),
            enemy_pupil_l_handles: Vec::new(),
            enemy_pupil_r_handles: Vec::new(),
            pickup_handles: Vec::new(),
            pickup_glow_handles: Vec::new(),
            pickup_accent_handles: Vec::new(),
            proj_handles: Vec::new(),
            proj_glow_handles: Vec::new(),
            ground_handles: Vec::new(),
            grain_mat: MaterialResource::default(),
            ground_mats: Default::default(),
            ground_zone: Zone::Meadow,
            decor_handles: Vec::new(),
            particle_handles: Vec::new(),
            slash_segs: Vec::new(),
            hud_line: Handle::NONE,
            heart_full: TextureResource::default(),
            heart_half: TextureResource::default(),
            heart_empty: TextureResource::default(),
            hearts: Vec::new(),
            heart_sig: String::new(),
            stats_dim: Handle::NONE,
            skills_dim: Handle::NONE,
            map_dim: Handle::NONE,
            items_dim: Handle::NONE,
            heart_darks: Vec::new(),
            heart_d_full: TextureResource::default(),
            heart_d_half: TextureResource::default(),
            heart_d_empty: TextureResource::default(),
            hud_dim_on: false,
            left_stats_panel: Handle::NONE,
            left_stats_text: Handle::NONE,
            sk_panel: Handle::NONE,
            sk_title: Handle::NONE,
            sk_names: Vec::new(),
            sk_keys: Vec::new(),
            items_panel: Handle::NONE,
            items_text: Handle::NONE,
            map_panel: Handle::NONE,
            map_cells: Vec::new(),
            map_cache: Vec::new(),
            map_sig: String::new(),
            log_h: Handle::NONE,
            log_bg: Handle::NONE,
            log_dim: Handle::NONE,
            banner_h: Handle::NONE,
            banner_timer: 0.0,
            banner_shown: false,
            patch_tex: TextureResource::default(),
            patch_tex_gold: TextureResource::default(),
            patch_tex_dark: TextureResource::default(),
            menu_title: Handle::NONE,
            menu_sub: Handle::NONE,
            menu_rows: Vec::new(),
            menu_row_bgs: Vec::new(),
            menu_row_golds: Vec::new(),
            menu_help: Handle::NONE,
            menu_help_bg: Handle::NONE,
            menu_footer: Handle::NONE,
            menu_index: 0,
            menu_help_on: false,
            menu_t: 0.0,
            menu_sig: String::new(),
            float_labels: Vec::new(),
            float_cache: Vec::new(),
            xp_bar_bg: Handle::NONE,
            xp_bar_fg: Handle::NONE,
            xpbar_on: false,
            hud_ui_on: true,
            font_scale: 0.0,
            draft_title: Handle::NONE,
            draft_cards_ui: Vec::new(),
            draft_panels: Vec::new(),
            draft_golds: Vec::new(),
            draft_darks: Vec::new(),
            draft_hint: Handle::NONE,
            draft_sig: String::new(),
            stats_open: false,
            stats_sel: 0,
            stat_repeat_plus: 0.0,
            stat_repeat_minus: 0.0,
            stats_panel: Handle::NONE,
            stats_title: Handle::NONE,
            stats_rows: Vec::new(),
            stats_hint: Handle::NONE,
            stats_sig: String::new(),
            paused: false,
            pause_sel: 0,
            pause_title: Handle::NONE,
            pause_rows: Vec::new(),
            pause_row_bgs: Vec::new(),
            pause_row_golds: Vec::new(),
            pause_darks: Vec::new(),
            pause_hint: Handle::NONE,
            pause_sig: String::new(),
            initialized: false,
        }
    }
}

impl Clone for Game {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            scene: self.scene,
            player_handle: self.player_handle,
            player_belly: self.player_belly,
            player_pupil_l: self.player_pupil_l,
            player_pupil_r: self.player_pupil_r,
            player_boot_l: self.player_boot_l,
            player_boot_r: self.player_boot_r,
            player_arm_l: self.player_arm_l,
            player_arm_r: self.player_arm_r,
            camera_handle: self.camera_handle,
            player_outline: self.player_outline,
            player_shadow: self.player_shadow,
            player_glow: self.player_glow,
            player_eyes: self.player_eyes,
            player_weapon: self.player_weapon,
            player_hp_bg: self.player_hp_bg,
            player_hp_fg: self.player_hp_fg,
            player_shield: self.player_shield,
            enemy_handles: self.enemy_handles.clone(),
            enemy_face_handles: self.enemy_face_handles.clone(),
            enemy_hp_handles: self.enemy_hp_handles.clone(),
            enemy_glow_handles: self.enemy_glow_handles.clone(),
            enemy_outline_handles: self.enemy_outline_handles.clone(),
            enemy_belly_handles: self.enemy_belly_handles.clone(),
            enemy_shadow_handles: self.enemy_shadow_handles.clone(),
            enemy_pupil_l_handles: self.enemy_pupil_l_handles.clone(),
            enemy_pupil_r_handles: self.enemy_pupil_r_handles.clone(),
            pickup_handles: self.pickup_handles.clone(),
            pickup_glow_handles: self.pickup_glow_handles.clone(),
            pickup_accent_handles: self.pickup_accent_handles.clone(),
            proj_handles: self.proj_handles.clone(),
            proj_glow_handles: self.proj_glow_handles.clone(),
            ground_handles: self.ground_handles.clone(),
            grain_mat: self.grain_mat.clone(),
            ground_mats: self.ground_mats.clone(),
            ground_zone: self.ground_zone,
            decor_handles: self.decor_handles.clone(),
            particle_handles: self.particle_handles.clone(),
            slash_segs: self.slash_segs.clone(),
            hud_line: self.hud_line,
            heart_full: self.heart_full.clone(),
            heart_half: self.heart_half.clone(),
            heart_empty: self.heart_empty.clone(),
            hearts: self.hearts.clone(),
            heart_sig: self.heart_sig.clone(),
            stats_dim: self.stats_dim,
            skills_dim: self.skills_dim,
            map_dim: self.map_dim,
            items_dim: self.items_dim,
            heart_darks: self.heart_darks.clone(),
            heart_d_full: self.heart_d_full.clone(),
            heart_d_half: self.heart_d_half.clone(),
            heart_d_empty: self.heart_d_empty.clone(),
            hud_dim_on: self.hud_dim_on,
            left_stats_panel: self.left_stats_panel,
            left_stats_text: self.left_stats_text,
            sk_panel: self.sk_panel,
            sk_title: self.sk_title,
            sk_names: self.sk_names.clone(),
            sk_keys: self.sk_keys.clone(),
            items_panel: self.items_panel,
            items_text: self.items_text,
            map_panel: self.map_panel,
            map_cells: self.map_cells.clone(),
            map_cache: self.map_cache.clone(),
            map_sig: self.map_sig.clone(),
            log_h: self.log_h,
            log_bg: self.log_bg,
            log_dim: self.log_dim,
            banner_h: self.banner_h,
            banner_timer: self.banner_timer,
            banner_shown: self.banner_shown,
            patch_tex: self.patch_tex.clone(),
            patch_tex_gold: self.patch_tex_gold.clone(),
            patch_tex_dark: self.patch_tex_dark.clone(),
            menu_title: self.menu_title,
            menu_sub: self.menu_sub,
            menu_rows: self.menu_rows.clone(),
            menu_row_bgs: self.menu_row_bgs.clone(),
            menu_row_golds: self.menu_row_golds.clone(),
            menu_help: self.menu_help,
            menu_help_bg: self.menu_help_bg,
            menu_footer: self.menu_footer,
            menu_index: self.menu_index,
            menu_help_on: self.menu_help_on,
            menu_t: self.menu_t,
            menu_sig: self.menu_sig.clone(),
            float_labels: self.float_labels.clone(),
            float_cache: self.float_cache.clone(),
            xp_bar_bg: self.xp_bar_bg,
            xp_bar_fg: self.xp_bar_fg,
            xpbar_on: self.xpbar_on,
            hud_ui_on: self.hud_ui_on,
            font_scale: self.font_scale,
            draft_title: self.draft_title,
            draft_cards_ui: self.draft_cards_ui.clone(),
            draft_panels: self.draft_panels.clone(),
            draft_golds: self.draft_golds.clone(),
            draft_darks: self.draft_darks.clone(),
            draft_hint: self.draft_hint,
            draft_sig: self.draft_sig.clone(),
            stats_open: self.stats_open,
            stats_sel: self.stats_sel,
            stat_repeat_plus: self.stat_repeat_plus,
            stat_repeat_minus: self.stat_repeat_minus,
            stats_panel: self.stats_panel,
            stats_title: self.stats_title,
            stats_rows: self.stats_rows.clone(),
            stats_hint: self.stats_hint,
            stats_sig: self.stats_sig.clone(),
            paused: self.paused,
            pause_sel: self.pause_sel,
            pause_title: self.pause_title,
            pause_rows: self.pause_rows.clone(),
            pause_row_bgs: self.pause_row_bgs.clone(),
            pause_row_golds: self.pause_row_golds.clone(),
            pause_darks: self.pause_darks.clone(),
            pause_hint: self.pause_hint,
            pause_sig: self.pause_sig.clone(),
            initialized: self.initialized,
        }
    }
}

impl Game {
    /// Fresh run on the current class (base nothing again). If `started` is
    /// false the game lands back on the main menu instead.
    fn reset_run(&mut self, started: bool) {
        let c = self.inner.player.class;
        self.inner = GameInner::default();
        self.inner.player = PlayerCore::new(c);
        self.inner.started = started;
        self.paused = false;
        self.stats_open = false;
    }
}

impl Plugin for Game {
    fn register(&self, _context: PluginRegistrationContext) -> GameResult {
        Ok(())
    }

    fn init(&mut self, _scene_path: Option<&str>, mut context: PluginContext) -> GameResult {
        // Always build our 2D scene programmatically — no dependency on editor scene.
        self.build_scene(&mut context);
        Ok(())
    }

    fn update(&mut self, context: &mut PluginContext) -> GameResult {
        if !self.initialized || self.scene.is_none() {
            return Ok(());
        }
        // Lazy HUD — safe even if UI container was empty at init.
        self.ensure_hud(context);
        let dt = context.dt.min(0.05);
        let input = context.input_state;
        // snapshot keys we care about
        let keys = [
            KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD,
            KeyCode::ArrowUp, KeyCode::ArrowLeft, KeyCode::ArrowDown, KeyCode::ArrowRight,
            KeyCode::Space, KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4,
            KeyCode::KeyQ, KeyCode::KeyE, KeyCode::KeyR, KeyCode::KeyF,
            KeyCode::KeyZ, KeyCode::KeyX, KeyCode::KeyC, KeyCode::KeyV, KeyCode::KeyB,
            KeyCode::KeyT, KeyCode::KeyH, KeyCode::Enter,
            KeyCode::Tab, KeyCode::Escape, KeyCode::KeyP,
            KeyCode::Minus, KeyCode::Equal, KeyCode::NumpadAdd, KeyCode::NumpadSubtract,
            KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4,
            KeyCode::F5, KeyCode::F6, KeyCode::F7, KeyCode::F8,
        ];
        let mut down: Vec<KeyCode> = Vec::new();
        for k in keys {
            if input.is_key_down(k) {
                down.push(k);
            }
        }
        let prev = self.inner.prev_keys.clone();

        // --- menu: idle world renders behind, sim waits for START ---
        if !self.inner.started {
            // navigate: Up/Down or W/S
            if pressed(input, KeyCode::ArrowUp, &prev) || pressed(input, KeyCode::KeyW, &prev) {
                self.menu_index = (self.menu_index + 2) % 3;
            }
            if pressed(input, KeyCode::ArrowDown, &prev) || pressed(input, KeyCode::KeyS, &prev) {
                self.menu_index = (self.menu_index + 1) % 3;
            }
            // menu: everyone starts a Drifter — first real class at level 5.
            // (cycling is gone; the tree decides from here.)
            if self.inner.player.class.tier() != 0 {
                self.inner.player = PlayerCore::new(CharacterClass::Drifter);
                self.inner.held = Weapon::rusty();
                self.inner.loadout.clear();
                self.inner.auto_cd.clear();
            }
            // confirm: Enter/Space on START begins; on HELP toggles panel
            if pressed(input, KeyCode::Enter, &prev) || pressed(input, KeyCode::Space, &prev) {
                if self.menu_index == 0 {
                    self.inner.started = true;
                    self.inner.log("the constraining of daniel begins! Arrows/Space attack.".to_string());
                    self.set_menu_visible(context, false);
                } else if self.menu_index == 2 {
                    self.menu_help_on = !self.menu_help_on;
                }
            }
            self.inner.prev_keys = down;
            self.sync_ground_colors(context);
            self.ensure_visuals(context);
            self.sync_visuals(context);
            self.sync_floats(context);
            self.sync_menu(context);
            self.inner.hud_timer -= dt;
            if self.inner.hud_timer <= 0.0 {
                self.update_hud(context);
                self.inner.hud_timer = 0.12;
            }
            return Ok(());
        }

        // --- level-up draft / class evolution: frozen, pick 1 of 3 ---
        if self.inner.draft_open || self.inner.class_open {
            // direct pick
            let mut pick: Option<usize> = None;
            for (k, idx) in [(KeyCode::Digit1, 0), (KeyCode::Digit2, 1), (KeyCode::Digit3, 2)] {
                if pressed(input, k, &prev) {
                    pick = Some(idx);
                }
            }
            // highlight cycle
            if pressed(input, KeyCode::ArrowLeft, &prev) || pressed(input, KeyCode::KeyA, &prev) {
                self.inner.draft_sel = (self.inner.draft_sel + 2) % 3;
            }
            if pressed(input, KeyCode::ArrowRight, &prev) || pressed(input, KeyCode::KeyD, &prev) {
                self.inner.draft_sel = (self.inner.draft_sel + 1) % 3;
            }
            if pressed(input, KeyCode::Enter, &prev) || pressed(input, KeyCode::Space, &prev) {
                pick = Some(self.inner.draft_sel);
            }
            if let Some(i) = pick {
                if self.inner.class_open {
                    self.inner.apply_class(i);
                } else {
                    self.inner.apply_draft(i);
                }
            }
            self.inner.prev_keys = down;
            // frozen render (no sim ticks: enemies, arrows, particles all hold still)
            self.sync_ground_colors(context);
            self.ensure_visuals(context);
            self.sync_visuals(context);
            self.sync_floats(context);
            self.sync_draft(context);
            self.inner.hud_timer -= dt;
            if self.inner.hud_timer <= 0.0 {
                self.update_hud(context);
                self.inner.hud_timer = 0.12;
            }
            return Ok(());
        }

        // --- stats menu (Tab): frozen plus/minus allocation ---
        if self.stats_open {
            if pressed(input, KeyCode::Tab, &prev) || pressed(input, KeyCode::Escape, &prev) {
                self.stats_open = false;
            }
            if pressed(input, KeyCode::ArrowUp, &prev) || pressed(input, KeyCode::KeyW, &prev) {
                self.stats_sel = (self.stats_sel + 4) % 5;
            }
            if pressed(input, KeyCode::ArrowDown, &prev) || pressed(input, KeyCode::KeyS, &prev) {
                self.stats_sel = (self.stats_sel + 1) % 5;
            }
            let kind = AttrKind::all()[self.stats_sel];
            // plus: = key, numpad +, or Right/D. minus: - key, numpad -, or Left/A.
            // tap once for one point; HOLD to pour points in (0.35s delay, 14/s).
            let plus_down = down.contains(&KeyCode::Equal)
                || down.contains(&KeyCode::NumpadAdd)
                || down.contains(&KeyCode::ArrowRight)
                || down.contains(&KeyCode::KeyD);
            let minus_down = down.contains(&KeyCode::Minus)
                || down.contains(&KeyCode::NumpadSubtract)
                || down.contains(&KeyCode::ArrowLeft)
                || down.contains(&KeyCode::KeyA);
            let plus_hit = pressed(input, KeyCode::Equal, &prev)
                || pressed(input, KeyCode::NumpadAdd, &prev)
                || pressed(input, KeyCode::ArrowRight, &prev)
                || pressed(input, KeyCode::KeyD, &prev);
            let minus_hit = pressed(input, KeyCode::Minus, &prev)
                || pressed(input, KeyCode::NumpadSubtract, &prev)
                || pressed(input, KeyCode::ArrowLeft, &prev)
                || pressed(input, KeyCode::KeyA, &prev);
            if plus_hit {
                self.inner.player.allocate(kind);
                self.stat_repeat_plus = 0.35;
            } else if plus_down {
                self.stat_repeat_plus -= dt;
                while self.stat_repeat_plus <= 0.0 {
                    self.inner.player.allocate(kind);
                    self.stat_repeat_plus += 0.07;
                }
            }
            if minus_hit {
                self.inner.player.deallocate(kind);
                self.stat_repeat_minus = 0.35;
            } else if minus_down {
                self.stat_repeat_minus -= dt;
                while self.stat_repeat_minus <= 0.0 {
                    self.inner.player.deallocate(kind);
                    self.stat_repeat_minus += 0.07;
                }
            }
            self.inner.prev_keys = down;
            // frozen render (sim holds still like the draft)
            self.sync_ground_colors(context);
            self.ensure_visuals(context);
            self.sync_visuals(context);
            self.sync_floats(context);
            self.sync_stats(context);
            self.inner.hud_timer -= dt;
            if self.inner.hud_timer <= 0.0 {
                self.update_hud(context);
                self.inner.hud_timer = 0.12;
            }
            return Ok(());
        } else if pressed(input, KeyCode::Tab, &prev) && self.inner.started && !self.inner.draft_open && !self.inner.class_open {
            self.stats_open = true;
            self.stats_sel = 0;
        }

        // --- pause menu (Esc/P): frozen, resume / restart / quit ---
        if self.paused {
            if pressed(input, KeyCode::ArrowUp, &prev) || pressed(input, KeyCode::KeyW, &prev) {
                self.pause_sel = (self.pause_sel + 2) % 3;
            }
            if pressed(input, KeyCode::ArrowDown, &prev) || pressed(input, KeyCode::KeyS, &prev) {
                self.pause_sel = (self.pause_sel + 1) % 3;
            }
            let mut act: Option<usize> = None;
            for (k, idx) in [(KeyCode::Digit1, 0), (KeyCode::Digit2, 1), (KeyCode::Digit3, 2)] {
                if pressed(input, k, &prev) {
                    act = Some(idx);
                }
            }
            if pressed(input, KeyCode::Enter, &prev) || pressed(input, KeyCode::Space, &prev) {
                act = Some(self.pause_sel);
            }
            if pressed(input, KeyCode::Escape, &prev) || pressed(input, KeyCode::KeyP, &prev) {
                self.paused = false;
            } else if let Some(i) = act {
                match i {
                    0 => self.paused = false,
                    1 => {
                        // restart the run fresh on the same class
                        self.reset_run(true);
                        self.paused = false;
                        self.inner.log("Run restarted — base nothing, again.".to_string());
                    }
                    _ => {
                        // quit to the main menu (keeps the class for next run)
                        self.reset_run(false);
                        self.set_menu_visible(context, true);
                        self.menu_index = 0;
                    }
                }
            }
            self.inner.prev_keys = down;
            // frozen render + pause overlay
            self.sync_ground_colors(context);
            self.ensure_visuals(context);
            self.sync_visuals(context);
            self.sync_floats(context);
            self.sync_pause(context);
            self.inner.hud_timer -= dt;
            if self.inner.hud_timer <= 0.0 {
                self.update_hud(context);
                self.inner.hud_timer = 0.12;
            }
            return Ok(());
        } else if (pressed(input, KeyCode::Escape, &prev) || pressed(input, KeyCode::KeyP, &prev))
            && self.inner.started
            && !self.inner.draft_open
            && !self.inner.class_open
            && !self.stats_open
        {
            self.paused = true;
            self.pause_sel = 0;
        }

        // --- movement: WASD only (arrow keys shoot — see below) ---
        let dstats = self.inner.player.derived();
        let mut mx = 0.0f32;
        let mut my = 0.0f32;
        if down.contains(&KeyCode::KeyA) {
            mx += 1.0;
        }
        if down.contains(&KeyCode::KeyD) {
            mx -= 1.0;
        }
        if down.contains(&KeyCode::KeyW) {
            my += 1.0;
        }
        if down.contains(&KeyCode::KeyS) {
            my -= 1.0;
        }
        if mx != 0.0 {
            self.inner.facing = mx.signum();
        }
        let mut speed = dstats.move_speed * self.inner.player.class.passive().speed * self.inner.player.item_totals().speed_mult;
        if self.inner.dash_timer > 0.0 {
            speed = 12.0;
            mx = self.inner.dash_dir.0;
            my = self.inner.dash_dir.1;
            self.inner.dash_timer -= dt;
        }
        let l = (mx * mx + my * my).sqrt().max(1.0);
        // fixed arena 24x18 tiles *1.3 => ~15.6 x 11.7 half-extents; clamp inside
        let moving_now = (mx != 0.0 || my != 0.0) as u8 as f32;
        self.inner.move_mag = self.inner.move_mag * 0.85 + moving_now * 0.15;
        self.inner.player_pos.0 += mx / l * speed * dt;
        self.inner.player_pos.1 += my / l * speed * dt;
        self.inner.player_pos.0 = self.inner.player_pos.0.clamp(-13.5, 13.5);
        self.inner.player_pos.1 = self.inner.player_pos.1.clamp(-9.5, 9.5);
        if self.inner.dash_cd > 0.0 {
            self.inner.dash_cd -= dt;
        }
        // dash trail
        if self.inner.dash_timer > 0.0 {
            self.inner.burst(self.inner.player_pos, self.inner.player.class.tint(), 2, 1.5, 0.35, 0.28);
        } else if moving_now > 0.5 && (self.inner.time * 8.0 % 1.0) < dt * 8.0 {
            // footstep dust
            self.inner.burst((self.inner.player_pos.0, self.inner.player_pos.1 - 0.3), (200, 200, 200), 1, 1.0, 0.3, 0.14);
        }

        // --- shooting: Space/F fires ahead, arrow keys aim (twin-stick).
        // Enemies ONLY take damage from arrow contact — no instant hits. ---
        self.inner.attack_cd -= dt;
        if self.inner.swing_t > 0.0 {
            self.inner.swing_t -= dt;
        }
        // --- basic attack: sword-line classes SWING (twin-stick melee),
        // everyone else shoots. Space autotargets; arrows stay manual.
        // Shared cooldown either way. ---
        let swings = self.inner.player.class.swings();
        let want_basic = down.contains(&KeyCode::Space) || down.contains(&KeyCode::KeyF);
        if want_basic && self.inner.attack_cd <= 0.0 {
            if swings {
                let d = self.inner.melee_aim(self.inner.swing_radius());
                self.inner.swing_basic(d);
            } else {
                let d = self.inner.aim_dir();
                self.inner.fire_arrow(d);
            }
            self.inner.attack_cd = self.inner.hold_cooldown();
        }
        let mut ax = 0.0f32;
        let mut ay = 0.0f32;
        // NOTE: world +x appears on screen-LEFT through this camera (it looks
        // down +Z), so screen-left arrows map to +x. Yes, really — verified
        // against the view matrix (look_at_rh flips x).
        if down.contains(&KeyCode::ArrowLeft) {
            ax += 1.0;
        }
        if down.contains(&KeyCode::ArrowRight) {
            ax -= 1.0;
        }
        if down.contains(&KeyCode::ArrowUp) {
            ay += 1.0;
        }
        if down.contains(&KeyCode::ArrowDown) {
            ay -= 1.0;
        }
        if (ax != 0.0 || ay != 0.0) && self.inner.attack_cd <= 0.0 {
            if swings {
                self.inner.swing_basic((ax, ay));
            } else {
                self.inner.fire_arrow((ax, ay));
            }
            self.inner.attack_cd = self.inner.hold_cooldown();
        }
        // --- class skills: Z/X/C/V primary, 1-4 + QER as alternates ---
        let skill_keys = [
            (KeyCode::Digit1, 0),
            (KeyCode::KeyZ, 0),
            (KeyCode::Digit2, 1),
            (KeyCode::KeyQ, 1),
            (KeyCode::KeyX, 1),
            (KeyCode::Digit3, 2),
            (KeyCode::KeyE, 2),
            (KeyCode::KeyC, 2),
            (KeyCode::Digit4, 3),
            (KeyCode::KeyR, 3),
            (KeyCode::KeyV, 3),
        ];
        for (k, idx) in skill_keys {
            if pressed(input, k, &prev) {
                self.inner.try_cast(idx);
            }
        }

        // --- stats / potions (Z/X/C/V/B are skills now; Tab menu allocates) ---
        if pressed(input, KeyCode::KeyT, &prev) {
            self.inner.player.auto_allocate();
            self.inner.log("Auto-allocated stats.".to_string());
        }
        if pressed(input, KeyCode::KeyH, &prev) {
            let dmax = self.inner.player.derived().max_hp;
            if self.inner.player.drink_potion() {
                self.inner.log("Potion!".to_string());
                let pp = self.inner.player_pos;
                self.inner.floats.push(FloatText { pos: pp, life: 0.9, text: format!("+{:.0}", dmax * 0.45) });
            }
        }
        // NOTE: no more free class switching — classes evolve at level
        // checkpoints (5/10/15...) via the class draft.

        // --- sim tick ---
        // kill hit-stop: freeze the world a beat so impacts land (bosses longer)
        if self.inner.hitstop > 0.0 {
            self.inner.hitstop -= dt;
            self.sync_ground_colors(context);
            self.ensure_visuals(context);
            self.sync_visuals(context);
            self.sync_floats(context);
            self.inner.hud_timer -= dt;
            if self.inner.hud_timer <= 0.0 {
                self.update_hud(context);
                self.inner.hud_timer = 0.12;
            }
            self.inner.prev_keys = down;
            return Ok(());
        }
        let prev_level = self.inner.player.level;
        self.inner.time += dt;
        self.inner.player.tick(dt);
        self.inner.tick_particles(dt);
        if self.inner.hit_flash > 0.0 {
            self.inner.hit_flash -= dt;
        }
        if self.inner.warn_cd > 0.0 {
            self.inner.warn_cd -= dt;
        }
        // low-HP warning (throttled): log + red pulse so damage is readable
        {
            let dmax = self.inner.player.derived().max_hp;
            if self.inner.player.hp > 0.0
                && self.inner.player.hp < dmax * 0.3
                && self.inner.warn_cd <= 0.0
            {
                self.inner.warn_cd = 6.0;
                self.inner.log("LOW HP! Press H for potion!".to_string());
                self.inner.burst(self.inner.player_pos, (255, 60, 60), 10, 3.5, 0.5, 0.22);
            }
        }
        if self.banner_timer > 0.0 {
            self.banner_timer -= dt;
        }
        self.inner.ensure_zone();
        self.inner.spawn_timer -= dt;
        if self.inner.spawn_timer <= 0.0 {
            self.inner.spawn_wave();
            // later stages press harder: faster waves on top of more bodies
            self.inner.spawn_timer = match self.inner.zone {
                Zone::Meadow => 1.2,
                Zone::CinderCaves => 1.0,
                Zone::FrostKeep => 0.8,
            };
        }
        if self.inner.boss_dead_timer > 0.0 {
            self.inner.boss_dead_timer -= dt;
        }

        // enemies AI + DoT
        let mut dead_enemies: Vec<usize> = Vec::new();
        let mut touch_hits: Vec<usize> = Vec::new();
        for (i, e) in self.inner.enemies.iter_mut().enumerate() {
            if e.tick_dot(dt) {
                dead_enemies.push(i);
                continue;
            }
            if e.touch_cd > 0.0 {
                e.touch_cd -= dt;
            }
            if e.shot_cd > 0.0 {
                e.shot_cd -= dt;
            }
            let dx = self.inner.player_pos.0 - e.pos.0;
            let dy = self.inner.player_pos.1 - e.pos.1;
            let dd = (dx * dx + dy * dy).sqrt().max(0.001);
            let sp = e.effective_speed();
            if e.shooter && dd < 4.5 {
                // shooters kite: back away when crowded, hold range otherwise
                e.pos.0 -= dx / dd * sp * 0.7 * dt;
                e.pos.1 -= dy / dd * sp * 0.7 * dt;
            } else {
                e.pos.0 += dx / dd * sp * dt;
                e.pos.1 += dy / dd * sp * dt;
            }
            // contact: discrete chunky hit per enemy (0.8s cooldown) — always felt
            if dd < 0.7 + e.size * 0.4 && e.touch_cd <= 0.0 {
                touch_hits.push(i);
            }
            // shooters spit bullets at range
            if e.shooter && dd < 9.0 && dd > 1.5 && e.shot_cd <= 0.0 {
                e.shot_cd = 2.5;
                let l = dd.max(0.001);
                self.inner.projectiles.push(Projectile {
                    pos: e.pos,
                    vel: (dx / l * 5.5, dy / l * 5.5),
                    life: 1.6,
                    dmg: e.atk * 0.8,
                    friendly: false,
                    color: (255, 90, 90),
                    size: 0.4,
                    burn: 0.0,
                    slow: 0.0,
                    lifesteal: 0.0,
                    homing: 0.0,
                });
            }
        }
        // touch damage (shield/defense inside take_damage, 20% always gets through)
        let mut thorn_killed: Vec<usize> = Vec::new();
        for i in touch_hits {
            let atk = self.inner.enemies.get(i).map(|e| e.atk).unwrap_or(0.0);
            if atk <= 0.0 {
                continue;
            }
            if let Some(e) = self.inner.enemies.get_mut(i) {
                e.touch_cd = 0.8;
            }
            let dealt = self.inner.player.take_damage(atk * 1.5);
            if dealt > 0.5 {
                self.inner.hit_flash = 0.12;
                self.inner.cam_shake = (self.inner.cam_shake + 0.25).min(1.0);
                // blood-ish puff at player
                self.inner.burst(self.inner.player_pos, (255, 90, 90), 3, 3.0, 0.35, 0.2);
            }
            // thornmail: attackers bleed for a cut of what they dealt.
            // deaths merge into the normal kill pass below (sorted, deduped).
            let th = self.inner.player.item_totals().thorns;
            if th > 0.0 && dealt > 0.0 {
                let (died, epos) = if let Some(e) = self.inner.enemies.get_mut(i) {
                    let epos = e.pos;
                    (e.take_damage(dealt * th), epos)
                } else {
                    (false, (0.0, 0.0))
                };
                if died {
                    self.inner.burst(epos, (200, 60, 60), 4, 3.0, 0.3, 0.16);
                    thorn_killed.push(i);
                }
            }
            if self.inner.player.hp <= 0.0 {
                break;
            }
        }
        dead_enemies.extend(thorn_killed);
        dead_enemies.sort_unstable();
        dead_enemies.dedup();
        for i in dead_enemies.into_iter().rev() {
            self.inner.kill_enemy(i);
        }

        // projectiles
        let mut proj_dead: Vec<usize> = Vec::new();
        let mut enemy_hits: Vec<(usize, usize, f32, f32, f32)> = Vec::new(); // proj_idx, enemy_idx, dmg, burn, slow
        let mut player_hits: Vec<(usize, f32)> = Vec::new(); // hostile proj_idx, dmg
        for (pi, pr) in self.inner.projectiles.iter_mut().enumerate() {
            pr.life -= dt;
            pr.pos.0 += pr.vel.0 * dt;
            pr.pos.1 += pr.vel.1 * dt;
            // telepathy: friendly shots bend toward the nearest enemy
            if pr.friendly && pr.homing > 0.0 {
                let mut bx: Option<(f32, f32, f32)> = None;
                for e in self.inner.enemies.iter() {
                    let dx = e.pos.0 - pr.pos.0;
                    let dy = e.pos.1 - pr.pos.1;
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < 8.0 && (bx.is_none() || d < bx.unwrap().2) {
                        bx = Some((dx, dy, d));
                    }
                }
                if let Some((dx, dy)) = bx.map(|(x, y, _)| (x, y)) {
                    let sp = (pr.vel.0 * pr.vel.0 + pr.vel.1 * pr.vel.1).sqrt().max(0.001);
                    let a0 = pr.vel.1.atan2(pr.vel.0);
                    let a1 = dy.atan2(dx);
                    let mut diff = a1 - a0;
                    while diff > std::f32::consts::PI {
                        diff -= std::f32::consts::TAU;
                    }
                    while diff < -std::f32::consts::PI {
                        diff += std::f32::consts::TAU;
                    }
                    let turn = diff.signum() * diff.abs().min(pr.homing * dt);
                    let na = a0 + turn;
                    pr.vel.0 = na.cos() * sp;
                    pr.vel.1 = na.sin() * sp;
                }
            }
            if pr.life <= 0.0 {
                proj_dead.push(pi);
                continue;
            }
            if !pr.friendly {
                // enemy bullets hunt the player, not other enemies
                let dx = pr.pos.0 - self.inner.player_pos.0;
                let dy = pr.pos.1 - self.inner.player_pos.1;
                if (dx * dx + dy * dy).sqrt() < 0.55 {
                    player_hits.push((pi, pr.dmg));
                }
                continue;
            }
            if pr.friendly {
                for (ei, e) in self.inner.enemies.iter().enumerate() {
                    if dist(pr.pos, e.pos) < 0.5 + e.size * 0.5 {
                        enemy_hits.push((pi, ei, pr.dmg, pr.burn, pr.slow));
                        if pr.lifesteal > 0.0 {
                            let d = self.inner.player.derived();
                            self.inner.player.hp = (self.inner.player.hp + pr.dmg * pr.lifesteal).min(d.max_hp);
                        }
                        // global item lifesteal (Vampire Fang et al.)
                        let gl = self.inner.player.item_totals().lifesteal;
                        if gl > 0.0 {
                            let d = self.inner.player.derived();
                            self.inner.player.hp = (self.inner.player.hp + pr.dmg * gl).min(d.max_hp);
                        }
                        break;
                    }
                }
            }
        }
        // apply hits (collect first to avoid borrow issues)
        let mut killed: Vec<usize> = Vec::new();
        // mark projs that hit (unless big life? all one-shot except visuals with dmg 0)
        let mut hit_projs: Vec<usize> = Vec::new();
        for (pi, ei, dmg, burn, slow) in enemy_hits {
            // impact spark even before kill check
            if dmg > 0.0 {
                if let Some(e) = self.inner.enemies.get(ei) {
                    self.inner.burst(e.pos, self.inner.player.class.tint(), 3, 3.5, 0.3, 0.16);
                }
            }
            if let Some(e) = self.inner.enemies.get_mut(ei) {
                if dmg > 0.0 {
                    // arrow connected — combo now feeds off hits, not casts
                    self.inner.player.register_hit();
                }
                if dmg > 0.0 && e.take_damage(dmg) {
                    if !killed.contains(&ei) {
                        killed.push(ei);
                    }
                } else if dmg > 0.0 {
                    if burn > 0.0 {
                        e.burn_ticks = 2.5 + self.inner.player.class.passive().burn_bonus;
                        e.burn_dps = burn;
                    }
                    if slow > 0.0 {
                        e.slow_timer = 2.0;
                    }
                    self.inner.floats.push(FloatText { pos: e.pos, life: 0.6, text: format!("{}", dmg as u32) });
                }
            }
            if !hit_projs.contains(&pi) {
                // visuals with dmg 0 pass through
                if self.inner.projectiles.get(pi).map(|p| p.dmg > 0.0).unwrap_or(false) {
                    hit_projs.push(pi);
                }
            }
        }
        killed.sort_unstable();
        killed.dedup();
        for i in killed.into_iter().rev() {
            self.inner.kill_enemy(i);
        }
        // hostile bullets bite the player (shield/defense/dodge apply)
        for (pi, dmg) in player_hits {
            let dealt = self.inner.player.take_damage(dmg);
            if dealt > 0.5 {
                self.inner.hit_flash = 0.1;
                self.inner.cam_shake = (self.inner.cam_shake + 0.15).min(0.8);
                self.inner.burst(self.inner.player_pos, (255, 90, 90), 2, 2.5, 0.3, 0.16);
            }
            if !proj_dead.contains(&pi) {
                proj_dead.push(pi);
            }
            if self.inner.player.hp <= 0.0 {
                break;
            }
        }
        proj_dead.extend(hit_projs);
        proj_dead.sort_unstable();
        proj_dead.dedup();
        for i in proj_dead.into_iter().rev() {
            if i < self.inner.projectiles.len() {
                self.inner.projectiles.remove(i);
            }
        }

        // pickups (magnet charms pull + widen the grab radius)
        let magnet_r = self.inner.player.item_totals().magnet.max(0.8);
        if magnet_r > 0.8 {
            let pp = self.inner.player_pos;
            for pk in self.inner.pickups.iter_mut() {
                let dx = pp.0 - pk.pos.0;
                let dy = pp.1 - pk.pos.1;
                let d = (dx * dx + dy * dy).sqrt();
                if d < magnet_r && d > 0.05 {
                    pk.pos.0 += dx / d * 6.0 * dt;
                    pk.pos.1 += dy / d * 6.0 * dt;
                }
            }
        }
        // auto battery: banked weapons fire on their own timers
        self.inner.tick_auto_weapons(dt);
        let mut got: Vec<usize> = Vec::new();
        for (i, pk) in self.inner.pickups.iter().enumerate() {
            if dist(pk.pos, self.inner.player_pos) < magnet_r {
                got.push(i);
            }
        }
        for i in got.into_iter().rev() {
            if i >= self.inner.pickups.len() {
                continue;
            }
            let pk = self.inner.pickups.remove(i);
            let pk_pos = pk.pos;
            match pk.kind {
                PickupKind::Heart => {
                    let d = self.inner.player.derived();
                    let amt = d.max_hp * 0.25;
                    self.inner.player.hp = (self.inner.player.hp + amt).min(d.max_hp);
                    self.inner.log("Heart +25% HP".to_string());
                    self.inner.floats.push(FloatText { pos: pk_pos, life: 0.9, text: format!("+{:.0}", amt) });
                    self.inner.burst(pk_pos, (255, 100, 110), 8, 3.0, 0.5, 0.2);
                }
                PickupKind::Mana => {
                    let d = self.inner.player.derived();
                    self.inner.player.mp = (self.inner.player.mp + d.max_mp * 0.5).min(d.max_mp);
                    self.inner.log("Mana surge!".to_string());
                    self.inner.burst(pk_pos, (100, 160, 255), 8, 3.0, 0.5, 0.2);
                }
                PickupKind::Gold => {
                    let g = ((10 + self.inner.player.level * 2) as f32 * self.inner.player.item_totals().gold_mult) as u32;
                    self.inner.player.gold += g;
                    self.inner.burst(pk_pos, (255, 215, 100), 6, 3.5, 0.4, 0.16);
                    if self.inner.player.gold % 3 == 0 {
                        self.inner.player.potions = (self.inner.player.potions + 1).min(9);
                        self.inner.log("Gold + bonus potion!".to_string());
                    }
                }
                PickupKind::Bomb => {
                    // bomb bursts into 12 live arrows — still only arrows hit
                    let bseed = self.inner.next_seed();
                    let (mut bdmg, _) = player_attack_damage(&self.inner.player, 0, bseed);
                    bdmg += self.inner.held.bonus_atk * self.inner.held.rarity.multiplier() + 25.0 + self.inner.player.level as f32 * 4.0;
                    for k in 0..12 {
                        let a = k as f32 * std::f32::consts::TAU / 12.0;
                        self.inner.projectiles.push(Projectile {
                            pos: pk_pos,
                            vel: (a.cos() * 8.0, a.sin() * 8.0),
                            life: 0.55 * self.inner.player.item_totals().range_mult,
                            dmg: bdmg,
                            friendly: true,
                            color: (255, 150, 60),
                            size: 0.5,
                            burn: 0.0,
                            slow: 0.0,
                            lifesteal: 0.0,
                            homing: self.inner.player.item_totals().homing,
                        });
                    }
                    self.inner.log("BOOM!".to_string());
                    self.inner.burst(pk_pos, (255, 150, 60), 18, 7.0, 0.6, 0.28);
                    self.inner.cam_shake = (self.inner.cam_shake + 0.6).min(1.2);
                }
                PickupKind::Chest => {
                    // chests are the ONLY enemy-drop gear path: 5% weapon,
                    // 5% item, otherwise a fat gold cache. Open by touching.
                    let s = self.inner.next_seed();
                    let r = (s % 100) as u32;
                    if r < 5 {
                        if let Some(w) = roll_loot(self.inner.player.kills + 20, self.inner.player.base_attrs.luck + 5, s) {
                            let label = self.inner.equip_weapon(w);
                            self.inner.log(format!("WEAPON: {} joins! (auto-fires)", label));
                        } else {
                            self.inner.player.gold += 40;
                            self.inner.log("Chest: dusty... +40g".to_string());
                        }
                    } else if r < 10 {
                        let idx = random_item_of(roll_item_rarity(s), s / 7 + 3);
                        let label = self.inner.player.grant_item(idx);
                        let def = &ITEMS[idx];
                        self.inner.log(format!("ITEM: {} {} — {}", def.rarity.name(), label, def.desc));
                    } else {
                        self.inner.player.gold += 30 + self.inner.player.level * 2;
                        self.inner.player.potions = (self.inner.player.potions + 1).min(9);
                        self.inner.log("Chest: gold + potion!".to_string());
                    }
                    self.inner.burst(pk_pos, (255, 220, 130), 12, 4.0, 0.6, 0.22);
                }
                PickupKind::Shrine => {
                    self.inner.player.shield_hp = 35.0 + self.inner.player.level as f32 * 3.0;
                    self.inner.player.shield_timer = 20.0;
                    self.inner.player.mp = self.inner.player.derived().max_mp;
                    self.inner.log("Shrine blessing: shield + full mana!".to_string());
                    self.inner.burst(pk_pos, (140, 255, 190), 14, 3.5, 0.8, 0.24);
                }
            }
        }

        // floats decay (+ cap: matches the 24-label UI pool)
        for f in self.inner.floats.iter_mut() {
            f.life -= dt;
            f.pos.1 += dt * 1.2;
        }
        self.inner.floats.retain(|f| f.life > 0.0);
        if self.inner.floats.len() > 24 {
            let excess = self.inner.floats.len() - 24;
            self.inner.floats.drain(..excess);
        }

        // level-up beam + banner
        if self.inner.player.level != prev_level {
            let pp = self.inner.player_pos;
            self.inner.burst(pp, (150, 255, 150), 22, 5.0, 0.9, 0.26);
            self.inner.burst(pp, (255, 240, 150), 12, 3.5, 0.7, 0.2);
            self.inner.cam_shake = (self.inner.cam_shake + 0.3).min(1.0);
            self.banner_timer = 3.5;
        }

        // death / respawn — forgiving, fun
        if self.inner.player.hp <= 0.0 {
            self.inner.log(format!("You fell... respawning with {}g lost. Kills: {}", (self.inner.player.gold / 4).min(50), self.inner.player.kills));
            self.inner.player.gold = self.inner.player.gold.saturating_sub((self.inner.player.gold / 4).min(50));
            let d = self.inner.player.derived();
            self.inner.player.hp = d.max_hp;
            self.inner.player.mp = d.max_mp;
            self.inner.player_pos = (0.0, 0.0);
            self.inner.enemies.retain(|e| e.boss); // clear trash, keep boss drama
            self.inner.player.combo_count = 0;
        }

        // Enter: if boss dead recently, jump to next zone feel (gain bonus level progress)
        if pressed(input, KeyCode::Enter, &prev) && self.inner.boss_dead_timer > 0.0 {
            let bonus = 50 + self.inner.player.level * 10;
            let ups = self.inner.player.add_xp(bonus);
            self.inner.handle_level_ups(ups);
            self.inner.log(format!("Portal! +{} xp{}", bonus, if ups > 0 { " LEVEL UP!" } else { "" }));
            self.inner.boss_dead_timer = 0.0;
        }

        self.inner.prev_keys = down;

        // debug: prove sim + transforms are alive (visible in cargo run console)
        self.inner.frame += 1;
        if self.inner.frame % 180 == 0 {
            // read back actual renderer state for the player node
            let (pinfo, ginfo, cinfo) = if let Ok(scene) = context.scenes.try_get_mut(self.scene) {
                let p = scene.graph.try_get_mut(self.player_handle)
                    .map(|r| {
                        let pos = **r.local_transform().position();
                        let sc = **r.local_transform().scale();
                        format!("pos=({:.2},{:.2},{:.2}) scale=({:.2},{:.2}) vis={} en={} color={:?}", pos.x, pos.y, pos.z, sc.x, sc.y, r.visibility(), r.is_enabled(), r.color())
                    })
                    .unwrap_or_else(|_| "MISSING/INVALID HANDLE".to_string());
                let g = self.ground_handles.first()
                    .map(|h| scene.graph.try_get_mut(*h)
                        .map(|r| {
                            let pos = **r.local_transform().position();
                            format!("ground0=({:.2},{:.2},{:.2})", pos.x, pos.y, pos.z)
                        })
                        .unwrap_or_else(|_| "ground INVALID".to_string()))
                    .unwrap_or("no ground".to_string());
                let c = scene.graph.try_get_mut(self.camera_handle)
                    .map(|cam| {
                        let pos = **cam.local_transform().position();
                        format!("cam=({:.2},{:.2},{:.2})", pos.x, pos.y, pos.z)
                    })
                    .unwrap_or_else(|_| "cam INVALID".to_string());
                (p, g, c)
            } else {
                ("no scene".to_string(), "no scene".to_string(), "no scene".to_string())
            };
            let ui_count = context.user_interfaces.iter().count();
            eprintln!(
                "[tcond] frame={} player=({:.1},{:.1}) cam=({:.1},{:.1}) enemies={} projectiles={} hp={:.0} ui_count={} hud_none={} | {} | {} | {}",
                self.inner.frame,
                self.inner.player_pos.0,
                self.inner.player_pos.1,
                self.inner.cam_pos.0,
                self.inner.cam_pos.1,
                self.inner.enemies.len(),
                self.inner.projectiles.len(),
                self.inner.player.hp,
                ui_count,
                self.hearts.is_empty(),
                pinfo,
                ginfo,
                cinfo,
            );
        }

        // visuals
        self.sync_ground_colors(context);
        self.ensure_visuals(context);
        self.sync_visuals(context);
        self.sync_floats(context);
        self.inner.hud_timer -= dt;
        if self.inner.hud_timer <= 0.0 {
            self.update_hud(context);
            self.inner.hud_timer = 0.12;
        }
        Ok(())
    }

    fn on_os_event(&mut self, _event: &Event<()>, _context: PluginContext) -> GameResult {
        Ok(())
    }

    fn on_ui_message(&mut self, _context: &mut PluginContext, _message: &UiMessage, _ui_handle: Handle<UserInterface>) -> GameResult {
        Ok(())
    }
}
