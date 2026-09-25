//! Scene building + per-frame world sync (ground, entities, camera).

use fyrox::{
    core::{algebra::Vector3, color::Color, pool::Handle},
    graph::SceneGraph,
    plugin::PluginContext,
    material::MaterialResource,
    scene::{
        base::BaseBuilder,
        camera::{CameraBuilder, OrthographicProjection, Projection},
        dim2::rectangle::Rectangle,
        transform::TransformBuilder,
        Scene,
    },
};
use crate::{
    gfx::{grain_texture, ground_texture, material_for, zone_index},
    pickup::PickupKind,
    plugin::Game,
    util::{col, darken, lighten, make_rect_mat, make_rect_z, set_pos, set_rot, set_scale},
    zone::Zone,
};

impl Game {

    pub(crate) fn build_scene(&mut self, ctx: &mut PluginContext) {
        let mut scene = Scene::new();
        // Camera at z=-10 with default orientation (faces +Z, toward the world).
        // Depth testing handles layering: nearer the camera = more negative z.
        // Ground 0 (farthest) / decor -1 / pickup -2 / enemy -3 / player -5 /
        // projectiles -6 / particles -7 (nearest). Opaque rects, no blending.
        let cam_t = TransformBuilder::new()
            .with_local_position(Vector3::new(0.0, 0.0, -10.0))
            .build();
        let cam = CameraBuilder::new(BaseBuilder::new().with_name("Cam").with_local_transform(cam_t))
            .with_projection(Projection::Orthographic(OrthographicProjection {
                vertical_size: 9.0,
                z_near: -20.0,
                z_far: 20.0,
            }))
            .build(&mut scene.graph);
        self.camera_handle = cam;

        // Black void behind everything (z=+1 sits behind ground z=0 from the
        // camera at z=-10): fills tile gaps and the area past the arena edge
        // so you never see the default clear color ("skybox").
        make_rect_z(&mut scene.graph, "Backdrop", (0.0, 0.0), (220.0, 220.0), Color::BLACK, 1.0);

        // --- Ground after backdrop so the arena draws on top of the void ---
        // Materials are generated once: grain for entities, one ground texture
        // per zone (swapped live when the zone changes).
        self.grain_mat = material_for(&grain_texture());
        self.ground_mats = [
            material_for(&ground_texture(Zone::Meadow)),
            material_for(&ground_texture(Zone::CinderCaves)),
            material_for(&ground_texture(Zone::FrostKeep)),
        ];
        self.ground_zone = self.inner.zone;
        let ground_mat = &self.ground_mats[zone_index(self.inner.zone)].clone();
        self.ground_handles.clear();
        const GW: i32 = 24;
        const GH: i32 = 18;
        const SP: f32 = 1.3;
        let (c1, c2) = self.inner.zone.palette();
        for gy in 0..GH {
            for gx in 0..GW {
                let wx = (gx - GW / 2) as f32 * SP;
                let wy = (gy - GH / 2) as f32 * SP;
                let checker = ((gx + gy) & 1) == 0;
                // dark border around arena so bounds are readable
                let edge = gx == 0 || gy == 0 || gx == GW - 1 || gy == GH - 1;
                let mut c = if checker { c1 } else { c2 };
                if edge {
                    c = darken(c, 0.55);
                }
                let g = make_rect_mat(&mut scene.graph, "Ground", (wx, wy), (SP * 0.96, SP * 0.96), col(c), 0.0, ground_mat);
                self.ground_handles.push(g);
            }
        }
        // vignette frame: dark trim just outside the arena edge
        {
            let hw = GW as f32 * SP * 0.5 + 0.7;
            let hh = GH as f32 * SP * 0.5 + 0.7;
            let fc = col((16, 26, 18));
            make_rect_z(&mut scene.graph, "Frame", (0.0, hh + 0.6), (hw * 2.0 + 2.4, 1.2), fc, 0.5);
            make_rect_z(&mut scene.graph, "Frame", (0.0, -hh - 0.6), (hw * 2.0 + 2.4, 1.2), fc, 0.5);
            make_rect_z(&mut scene.graph, "Frame", (hw + 0.6, 0.0), (1.2, hh * 2.0), fc, 0.5);
            make_rect_z(&mut scene.graph, "Frame", (-hw - 0.6, 0.0), (1.2, hh * 2.0), fc, 0.5);
        }
        // decor — fixed scattered positions, opaque
        self.decor_handles.clear();
        let mut hseed: u32 = 1234567;
        let mut rnd = || {
            hseed = hseed.wrapping_mul(1664525).wrapping_add(1013904223);
            (hseed >> 8) as f32 / 16777216.0
        };
        for _ in 0..90 {
            let wx = (rnd() - 0.5) * (GW as f32 * SP - 2.0);
            let wy = (rnd() - 0.5) * (GH as f32 * SP - 2.0);
            let kind = (rnd() * 4.0) as u32;
            let (dc, sx, sy) = match self.inner.zone {
                Zone::Meadow => match kind {
                    0 => ((38, 95, 52), 0.28, 0.42),
                    1 => ((215, 130, 170), 0.22, 0.22),
                    2 => ((235, 225, 140), 0.20, 0.20),
                    _ => ((105, 105, 110), 0.34, 0.26),
                },
                Zone::CinderCaves => match kind {
                    0 => ((255, 140, 50), 0.24, 0.24),
                    1 => ((55, 32, 30), 0.42, 0.32),
                    2 => ((170, 55, 32), 0.50, 0.16),
                    _ => ((115, 48, 38), 0.16, 0.46),
                },
                Zone::FrostKeep => match kind {
                    0 => ((235, 245, 255), 0.36, 0.28),
                    1 => ((150, 200, 240), 0.20, 0.38),
                    2 => ((65, 105, 125), 0.32, 0.42),
                    _ => ((205, 225, 245), 0.24, 0.24),
                },
            };
            let d = make_rect_z(&mut scene.graph, "Decor", (wx, wy), (sx, sy), col(dc), -1.0);
            self.decor_handles.push(d);
        }
        // particle pool — hidden by scale 0 until used, opaque
        self.particle_handles.clear();
        for _ in 0..80 {
            let pr = make_rect_z(&mut scene.graph, "Particle", (0.0, 0.0), (0.001, 0.001), col((255, 255, 255)), -7.0);
            self.particle_handles.push(pr);
        }
        // swoosh pool: 7 rotated segments form the blade-arc VFX
        self.slash_segs.clear();
        for _ in 0..7 {
            let sg = make_rect_z(&mut scene.graph, "SlashSeg", (0.0, 0.0), (0.001, 0.001), col((255, 255, 255)), -6.5);
            self.slash_segs.push(sg);
        }

        // --- Player LAST (on top), bigger for readability ---
        let tint = self.inner.player.class.tint();
        let grain = &self.grain_mat.clone();
        // white outline behind body so you pop against green tiles
        self.player_outline = make_rect_mat(&mut scene.graph, "PlayerOutline", (0.0, 0.0), (1.35, 1.50), col((245, 245, 250)), -4.8, grain);
        self.player_shadow = make_rect_z(&mut scene.graph, "PlayerShadow", (0.0, -0.55), (1.0, 0.32), col((18, 20, 30)), -4.7);
        // reused as bouncing YOU arrow (gold), always visible above you
        self.player_glow = make_rect_z(&mut scene.graph, "YouArrow", (0.0, 1.6), (0.55, 0.35), col((255, 215, 90)), -5.6);
        self.player_handle = make_rect_mat(&mut scene.graph, "Player", (0.0, 0.0), (1.05, 1.05), col(tint), -5.0, grain);
        self.player_belly = make_rect_mat(&mut scene.graph, "PlayerBelly", (0.0, -0.3), (0.9, 0.6), col(darken(tint, 0.7)), -5.05, grain);
        self.player_eyes = make_rect_z(&mut scene.graph, "PlayerEyes", (0.15, 0.28), (0.55, 0.20), Color::WHITE, -5.1);
        self.player_pupil_l = make_rect_z(&mut scene.graph, "PlayerPupilL", (0.05, 0.28), (0.12, 0.12), col((20, 20, 26)), -5.15);
        self.player_pupil_r = make_rect_z(&mut scene.graph, "PlayerPupilR", (0.28, 0.28), (0.12, 0.12), col((20, 20, 26)), -5.15);
        self.player_boot_l = make_rect_mat(&mut scene.graph, "PlayerBootL", (-0.3, -0.75), (0.34, 0.28), col((30, 28, 36)), -4.9, grain);
        self.player_boot_r = make_rect_mat(&mut scene.graph, "PlayerBootR", (0.3, -0.75), (0.34, 0.28), col((30, 28, 36)), -4.9, grain);
        self.player_arm_l = make_rect_mat(&mut scene.graph, "PlayerArmL", (-0.75, 0.0), (0.26, 0.6), col(darken(tint, 0.8)), -4.9, grain);
        self.player_arm_r = make_rect_mat(&mut scene.graph, "PlayerArmR", (0.75, 0.0), (0.26, 0.6), col(darken(tint, 0.8)), -4.9, grain);
        self.player_weapon = make_rect_mat(&mut scene.graph, "PlayerWeapon", (0.7, 0.0), (0.65, 0.22), col(lighten(tint, 70)), -5.2, grain);
        self.player_hp_bg = make_rect_z(&mut scene.graph, "PlayerHpBg", (0.0, 0.95), (1.2, 0.16), col((15, 15, 20)), -5.3);
        self.player_hp_fg = make_rect_z(&mut scene.graph, "PlayerHpFg", (0.0, 0.95), (1.1, 0.10), col((90, 230, 120)), -5.4);
        self.player_shield = make_rect_z(&mut scene.graph, "PlayerShield", (0.0, 0.0), (0.001, 0.001), col((120, 200, 255)), -5.5);

        // Pre-warm camera
        self.inner.cam_pos = self.inner.player_pos;

        self.scene = ctx.scenes.add(scene);

        // NOTE: Do NOT touch UI here — at init time the UiContainer can be empty
        // (caused "must have at least one user interface" panic on macOS).
        // HUD is created lazily in ensure_hud() during update().
        self.initialized = true;
    }

    pub(crate) fn sync_ground_colors(&mut self, ctx: &mut PluginContext) {
        if self.scene.is_none() {
            return;
        }
        // Static arena: full tiles, plain checker + dark border. Characters live
        // at negative z (nearer the camera at z=-10), so depth testing draws them
        // on top of the grid — no punch-through holes, no dimming ring.
        // (Menu mode darkens the whole arena so the title art pops.)
        let (c1, c2) = self.inner.zone.palette();
        let menu_dim: f32 = if !self.inner.started {
            0.30
        } else if self.inner.draft_open || self.paused || self.story_open {
            0.35
        } else {
            1.0
        };
        if let Ok(scene) = ctx.scenes.try_get_mut(self.scene) {
            // zone change: swap every tile to the new ground texture
            if self.inner.zone != self.ground_zone {
                let mat = self.ground_mats[zone_index(self.inner.zone)].clone();
                for h in self.ground_handles.iter() {
                    if let Ok(r) = scene.graph.try_get_mut(*h) {
                        r.material_mut().set_value_and_mark_modified(mat.clone());
                    }
                }
                self.ground_zone = self.inner.zone;
            }
            const GW: i32 = 24;
            const GH: i32 = 18;
            const SP: f32 = 1.3;
            for (i, h) in self.ground_handles.iter().enumerate() {
                let gx = (i as i32) % GW;
                let gy = (i as i32) / GW;
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_scale(r, SP * 0.96, SP * 0.96);
                    let checker = ((gx + gy) & 1) == 0;
                    let edge = gx == 0 || gy == 0 || gx == GW - 1 || gy == GH - 1;
                    let mut c = if checker { c1 } else { c2 };
                    if edge {
                        c = darken(c, 0.55);
                    }
                    c = (
                        (c.0 as f32 * menu_dim) as u8,
                        (c.1 as f32 * menu_dim) as u8,
                        (c.2 as f32 * menu_dim) as u8,
                    );
                    r.set_color(col(c));
                }
            }
            // Decor keeps its build-time size/position forever — nothing here
            // hides or rescales it, so nothing can poke through the character
            // by popping back in, and nothing vanishes around the player.
        }
    }

    pub(crate) fn ensure_visuals(&mut self, ctx: &mut PluginContext) {
        if self.scene.is_none() {
            return;
        }
        let Ok(scene) = ctx.scenes.try_get_mut(self.scene) else {
            return;
        };
        // self-healing singletons: if a stored handle went stale (bad gen /
        // wrong scene / removed), rebuild it instead of silently skipping.
        fn heal(handle: &mut Handle<Rectangle>, scene: &mut Scene, name: &str, z: f32, color: Color, scale: f32) {
            let bad = handle.is_none() || scene.graph.try_get(*handle).is_err();
            if bad {
                *handle = make_rect_z(&mut scene.graph, name, (0.0, -100.0), (scale, scale), color, z);
            }
        }
        fn heal_mat(handle: &mut Handle<Rectangle>, scene: &mut Scene, name: &str, z: f32, color: Color, scale: f32, mat: &MaterialResource) {
            let bad = handle.is_none() || scene.graph.try_get(*handle).is_err();
            if bad {
                *handle = make_rect_mat(&mut scene.graph, name, (0.0, -100.0), (scale, scale), color, z, mat);
            }
        }
        fn sync_pool(pool: &mut Vec<Handle<Rectangle>>, want: usize, scene: &mut Scene, name: &str, z: f32, color: Color, scale: f32, mat: Option<&MaterialResource>) {
            while pool.len() < want {
                let h = match mat {
                    Some(m) => make_rect_mat(&mut scene.graph, name, (0.0, -100.0), (scale, scale), color, z, m),
                    None => make_rect_z(&mut scene.graph, name, (0.0, -100.0), (scale, scale), color, z),
                };
                pool.push(h);
            }
            while pool.len() > want {
                if let Some(h) = pool.pop() {
                    scene.graph.remove_node(h);
                }
            }
        }
        let grain = self.grain_mat.clone();
        let gm = Some(&grain);
        let flat: Option<&MaterialResource> = None;
        heal_mat(&mut self.player_outline, scene, "PlayerOutline", -4.8, col((245, 245, 250)), 1.6, &grain);
        heal(&mut self.player_shadow, scene, "PlayerShadow", -4.7, col((18, 20, 30)), 1.0);
        heal(&mut self.player_glow, scene, "YouArrow", -5.6, col((255, 215, 90)), 0.55);
        heal_mat(&mut self.player_handle, scene, "Player", -5.0, col(self.inner.player.class.tint()), 1.3, &grain);
        heal_mat(&mut self.player_belly, scene, "PlayerBelly", -5.05, col(darken(self.inner.player.class.tint(), 0.7)), 0.9, &grain);
        heal(&mut self.player_eyes, scene, "PlayerEyes", -5.1, Color::WHITE, 0.5);
        heal_mat(&mut self.player_pupil_l, scene, "PlayerPupilL", -5.15, col((20, 20, 26)), 0.12, &grain);
        heal_mat(&mut self.player_pupil_r, scene, "PlayerPupilR", -5.15, col((20, 20, 26)), 0.12, &grain);
        heal_mat(&mut self.player_boot_l, scene, "PlayerBootL", -4.9, col((30, 28, 36)), 0.34, &grain);
        heal_mat(&mut self.player_boot_r, scene, "PlayerBootR", -4.9, col((30, 28, 36)), 0.34, &grain);
        heal_mat(&mut self.player_arm_l, scene, "PlayerArmL", -4.9, col(darken(self.inner.player.class.tint(), 0.8)), 0.4, &grain);
        heal_mat(&mut self.player_arm_r, scene, "PlayerArmR", -4.9, col(darken(self.inner.player.class.tint(), 0.8)), 0.4, &grain);
        heal_mat(&mut self.player_weapon, scene, "PlayerWeapon", -5.2, Color::WHITE, 0.5, &grain);
        heal(&mut self.player_hp_bg, scene, "PlayerHpBg", -5.3, col((15, 15, 20)), 1.0);
        heal(&mut self.player_hp_fg, scene, "PlayerHpFg", -5.4, col((90, 230, 120)), 1.0);
        heal(&mut self.player_shield, scene, "PlayerShield", -5.5, col((120, 200, 255)), 1.9);
        let n_en = self.inner.enemies.len();
        sync_pool(&mut self.enemy_handles, n_en, scene, "Enemy", -3.0, Color::RED, 1.0, gm);
        sync_pool(&mut self.enemy_outline_handles, n_en, scene, "EnemyOutline", -2.95, col((25, 22, 28)), 1.1, gm);
        sync_pool(&mut self.enemy_belly_handles, n_en, scene, "EnemyBelly", -3.05, col((200, 200, 200)), 0.8, gm);
        sync_pool(&mut self.enemy_shadow_handles, n_en, scene, "EnemyShadow", -2.85, col((15, 15, 18)), 0.9, flat);
        sync_pool(&mut self.enemy_face_handles, n_en, scene, "EnemyFace", -3.1, Color::WHITE, 0.5, gm);
        sync_pool(&mut self.enemy_pupil_l_handles, n_en, scene, "EnemyPupilL", -3.15, col((20, 16, 20)), 0.12, flat);
        sync_pool(&mut self.enemy_pupil_r_handles, n_en, scene, "EnemyPupilR", -3.15, col((20, 16, 20)), 0.12, flat);
        sync_pool(&mut self.enemy_hp_handles, n_en, scene, "EnemyHp", -3.2, Color::GREEN, 0.9, flat);
        sync_pool(&mut self.enemy_glow_handles, n_en, scene, "EnemyGlow", -2.9, col((255, 100, 100)), 0.001, flat);
        let n_pk = self.inner.pickups.len();
        sync_pool(&mut self.pickup_handles, n_pk, scene, "Pickup", -2.0, Color::GOLD, 0.5, gm);
        sync_pool(&mut self.pickup_accent_handles, n_pk, scene, "PickupAccent", -2.1, Color::WHITE, 0.2, flat);
        sync_pool(&mut self.pickup_glow_handles, n_pk, scene, "PickupGlow", -1.9, col((255, 240, 180)), 0.001, flat);
        let n_pr = self.inner.projectiles.len();
        sync_pool(&mut self.proj_handles, n_pr, scene, "Proj", -6.0, Color::WHITE, 0.6, gm);
        sync_pool(&mut self.proj_glow_handles, n_pr, scene, "ProjGlow", -5.9, col((255, 255, 255)), 0.001, flat);
        // particles use fixed pool — hidden by scale until used
        while self.slash_segs.len() < 7 {
            let sg = make_rect_z(&mut scene.graph, "SlashSeg", (0.0, -100.0), (0.001, 0.001), col((255, 255, 255)), -6.5);
            self.slash_segs.push(sg);
        }
    }

    pub(crate) fn sync_visuals(&mut self, ctx: &mut PluginContext) {
        if self.scene.is_none() {
            return;
        }
        let Ok(scene) = ctx.scenes.try_get_mut(self.scene) else {
            return;
        };
        let t = self.inner.time;
        // draft / class / menu / pause / story dim the WHOLE world
        let dim: f32 = if self.inner.draft_open || !self.inner.started || self.paused || self.story_open { 0.35 } else { 1.0 };
        let dc = |c: (u8, u8, u8)| col(darken(c, dim));
        let (ppx, ppy) = self.inner.player_pos;
        let facing = self.inner.facing;
        let tint = self.inner.player.class.tint();
        let wcol = lighten(self.inner.held.rarity.color(), 30);
        let moving = self.inner.move_mag;
        let bob = (t * 9.0).sin() * 0.06 * moving.min(1.0);

        // --- player: shadow / body / belly / eyes+pupils / boots / arms / weapon ---
        // attack lunge + dash stretch + hit pop reshape the silhouette
        let lunge = if self.inner.swing_t > 0.0 { facing * 0.18 } else { 0.0 };
        let dashing = self.inner.dash_timer > 0.0;
        let pop = if self.inner.hit_flash > 0.0 { 1.1 } else { 1.0 };
        if let Ok(r) = scene.graph.try_get_mut(self.player_outline) {
            set_pos(r, ppx + lunge, ppy + bob, -4.8);
            if dashing {
                set_scale(r, 1.60 * 1.2, 1.85 * 0.82);
            } else {
                set_scale(r, 1.60 * pop, 1.85 * pop);
            }
            // flash outline too so hits read clearly
            if self.inner.hit_flash > 0.0 {
                r.set_color(dc((255, 255, 255)));
            } else {
                r.set_color(dc((245, 245, 250)));
            }
        }
        if let Ok(r) = scene.graph.try_get_mut(self.player_shadow) {
            set_pos(r, ppx, ppy - 0.72, -4.7);
            set_scale(r, 1.05, 0.30);
            r.set_color(dc((18, 20, 30)));
        }
        // YOU arrow — bouncing gold marker, impossible to lose yourself
        if let Ok(r) = scene.graph.try_get_mut(self.player_glow) {
            let bounce = (t * 4.0).sin() * 0.12;
            set_pos(r, ppx, ppy + 1.65 + bounce, -5.6);
            // arrow-ish: wide top, pulse with time
            let pulse = 1.0 + (t * 4.0).cos() * 0.10;
            set_scale(r, 0.55 * pulse, 0.35 / pulse.max(0.7));
            r.set_color(dc((255, 215, 90)));
        }
        if let Ok(r) = scene.graph.try_get_mut(self.player_handle) {
            set_pos(r, ppx + lunge, ppy + bob, -5.0);
            // tall humanoid torso (not a tile-square) — deliberately large for readability
            if dashing {
                set_scale(r, 1.30 * 1.25, 1.55 * 0.8);
            } else {
                set_scale(r, 1.30 * pop, 1.55 * pop);
            }
            if self.inner.hit_flash > 0.0 {
                r.set_color(dc((255, 255, 255)));
            } else {
                r.set_color(dc(tint));
            }
        }
        if let Ok(r) = scene.graph.try_get_mut(self.player_belly) {
            set_pos(r, ppx + lunge, ppy - 0.3 + bob * 0.5, -5.05);
            set_scale(r, 0.9 * pop, 0.6 * pop);
            r.set_color(dc(darken(tint, 0.7)));
        }
        if let Ok(r) = scene.graph.try_get_mut(self.player_eyes) {
            set_pos(r, ppx + facing * 0.18 + lunge, ppy + 0.32 + bob, -5.1);
            set_scale(r, 0.62, 0.22);
            r.set_color(dc((255, 255, 255)));
        }
        // pupils track facing so the knight actually looks somewhere
        if let Ok(r) = scene.graph.try_get_mut(self.player_pupil_l) {
            set_pos(r, ppx + facing * 0.18 - 0.14 + facing * 0.05 + lunge, ppy + 0.32 + bob, -5.15);
            set_scale(r, 0.12, 0.12);
            r.set_color(dc((20, 20, 26)));
        }
        if let Ok(r) = scene.graph.try_get_mut(self.player_pupil_r) {
            set_pos(r, ppx + facing * 0.18 + 0.14 + facing * 0.05 + lunge, ppy + 0.32 + bob, -5.15);
            set_scale(r, 0.12, 0.12);
            r.set_color(dc((20, 20, 26)));
        }
        // boots alternate while moving; arms counter-swing
        {
            let step = (t * 9.0).sin() * 0.08 * moving.min(1.0);
            if let Ok(r) = scene.graph.try_get_mut(self.player_boot_l) {
                set_pos(r, ppx - 0.3 + lunge, ppy - 0.75 + step, -4.9);
                set_scale(r, 0.34, 0.28);
                r.set_color(dc((30, 28, 36)));
            }
            if let Ok(r) = scene.graph.try_get_mut(self.player_boot_r) {
                set_pos(r, ppx + 0.3 + lunge, ppy - 0.75 - step, -4.9);
                set_scale(r, 0.34, 0.28);
                r.set_color(dc((30, 28, 36)));
            }
            if let Ok(r) = scene.graph.try_get_mut(self.player_arm_l) {
                set_pos(r, ppx - 0.78 + lunge, ppy - step * 1.2, -4.9);
                set_scale(r, 0.26, 0.6);
                r.set_color(dc(darken(tint, 0.8)));
            }
            if let Ok(r) = scene.graph.try_get_mut(self.player_arm_r) {
                set_pos(r, ppx + 0.78 + lunge, ppy + step * 1.2, -4.9);
                set_scale(r, 0.26, 0.6);
                r.set_color(dc(darken(tint, 0.8)));
            }
        }
        if let Ok(r) = scene.graph.try_get_mut(self.player_weapon) {
            // idle: held at the side; mid-swing: sweeps +-63° around the
            // recorded swing angle so the weapon follows vertical swings too
            let (wx, wy, rot) = if self.inner.swing_t > 0.0 {
                let ph = (1.0 - self.inner.swing_t / 0.22).clamp(0.0, 1.0);
                let a = self.inner.swing_ang - 1.1 + 2.2 * ph;
                (ppx + a.cos() * 0.95, ppy + a.sin() * 0.95, a)
            } else {
                (ppx + facing * 0.85, ppy, facing * 0.45)
            };
            set_pos(r, wx, wy, -5.2);
            set_scale(r, 0.70, 0.24);
            r.set_color(dc(wcol));
            set_rot(r, rot);
        }
        // swoosh: pooled segments trace the blade arc, shrinking as they die
        for (i, h) in self.slash_segs.iter().enumerate() {
            if let Ok(r) = scene.graph.try_get_mut(*h) {
                if let Some(sg) = self.inner.arc_fx.get(i) {
                    let f = (sg.life / sg.max_life).clamp(0.0, 1.0);
                    set_pos(r, sg.pos.0, sg.pos.1, -6.5);
                    set_scale(r, (sg.len * (0.4 + f * 0.6)).max(0.08), sg.thick);
                    r.set_color(dc(sg.color));
                    set_rot(r, sg.angle);
                } else {
                    set_scale(r, 0.001, 0.001);
                }
            }
        }
        // Player HP lives in the heart row now (Isaac-style) — keep the
        // world-space bar nodes parked invisible.
        if let Ok(r) = scene.graph.try_get_mut(self.player_hp_bg) {
            set_scale(r, 0.001, 0.001);
        }
        if let Ok(r) = scene.graph.try_get_mut(self.player_hp_fg) {
            set_scale(r, 0.001, 0.001);
        }
        let p = &self.inner.player;
        if let Ok(r) = scene.graph.try_get_mut(self.player_shield) {
            if p.shield_hp > 0.0 {
                set_pos(r, ppx, ppy, -5.5);
                set_scale(r, 1.9, 1.9);
                r.set_color(dc((120, 200, 255)));
            } else {
                set_scale(r, 0.001, 0.001);
            }
        }
        // camera: tight follow so character stays centered, world scrolls underneath
        if let Ok(c) = scene.graph.try_get_mut(self.camera_handle) {
            let (cx, cy) = self.inner.cam_pos;
            let nx = cx + (ppx - cx) * 0.25;
            let ny = cy + (ppy - cy) * 0.25;
            self.inner.cam_pos = (nx, ny);
            // SHAKE toggle zeroes the trauma at render time (sim still tracks)
            let sh = if self.shake_on { self.inner.cam_shake } else { 0.0 };
            let shx = if sh > 0.0 { (t * 70.0).sin() * sh * 0.25 } else { 0.0 };
            let shy = if sh > 0.0 { (t * 55.0).cos() * sh * 0.25 } else { 0.0 };
            c.local_transform_mut().set_position(Vector3::new(nx + shx, ny + shy, -10.0));
        }
        // enemies — outline + shaded body + belly + shadow + tracking pupils + hp
        for (i, e) in self.inner.enemies.iter().enumerate() {
            let wob = (t * 5.0 + i as f32 * 1.7).sin() * 0.05;
            let esz = e.size + 0.25;
            let epop = if e.flash > 0.0 { 1.12 } else { 1.0 };
            let look = (ppx - e.pos.0).signum();
            if let Some(h) = self.enemy_outline_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_pos(r, e.pos.0, e.pos.1 + wob * 0.5, -2.95);
                    set_scale(r, (esz + 0.18) * epop, (esz + 0.18) * epop);
                    r.set_color(dc((25, 22, 28)));
                }
            }
            if let Some(h) = self.enemy_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_pos(r, e.pos.0, e.pos.1 + wob * 0.5, -3.0);
                    set_scale(r, esz * epop, esz * epop);
                    if e.flash > 0.0 {
                        r.set_color(dc((255, 255, 255)));
                    } else if e.boss {
                        r.set_color(dc((200, 45, 55)));
                    } else if e.elite {
                        r.set_color(dc((235, 200, 90)));
                    } else if e.burn_ticks > 0.0 {
                        r.set_color(dc((255, 130, 60)));
                    } else if e.slow_timer > 0.0 {
                        r.set_color(dc((150, 200, 255)));
                    } else {
                        r.set_color(dc(e.color));
                    }
                }
            }
            if let Some(h) = self.enemy_belly_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_pos(r, e.pos.0, e.pos.1 - esz * 0.18 + wob * 0.5, -3.05);
                    set_scale(r, esz * 0.8 * epop, esz * 0.5 * epop);
                    r.set_color(dc(darken(e.color, 0.6)));
                }
            }
            if let Some(h) = self.enemy_shadow_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_pos(r, e.pos.0, e.pos.1 - esz * 0.55, -2.85);
                    set_scale(r, esz * 0.95, 0.2);
                    r.set_color(dc((15, 15, 18)));
                }
            }
            if let Some(h) = self.enemy_face_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    let dx = (ppx - e.pos.0).signum() * 0.1;
                    set_pos(r, e.pos.0 + dx, e.pos.1 + 0.22, -3.1);
                    set_scale(r, 0.5, 0.16);
                    if e.boss || e.elite {
                        r.set_color(dc((255, 60, 60)));
                    } else {
                        r.set_color(dc((255, 255, 255)));
                    }
                }
            }
            // pupils lean toward you — they see you coming
            if let Some(h) = self.enemy_pupil_l_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_pos(r, e.pos.0 - 0.12 + look * 0.08, e.pos.1 + 0.22, -3.15);
                    set_scale(r, 0.11, 0.11);
                    r.set_color(dc((20, 16, 20)));
                }
            }
            if let Some(h) = self.enemy_pupil_r_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_pos(r, e.pos.0 + 0.12 + look * 0.08, e.pos.1 + 0.22, -3.15);
                    set_scale(r, 0.11, 0.11);
                    r.set_color(dc((20, 16, 20)));
                }
            }
            if let Some(h) = self.enemy_hp_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    // always visible — read enemy health at a glance
                    let f = (e.hp / e.max_hp).clamp(0.0, 1.0);
                    let w = 0.9 * f;
                    set_pos(r, e.pos.0 - (0.9 - w) * 0.5, e.pos.1 + 0.65, -3.2);
                    set_scale(r, w.max(0.001), 0.10);
                        r.set_color(dc(if f > 0.5 { (110, 230, 120) } else if f > 0.25 { (240, 200, 80) } else { (240, 90, 90) }));
                }
            }
            if let Some(h) = self.enemy_glow_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_scale(r, 0.001, 0.001);
                }
            }
        }
        // pickups — body + accent icon (shine, lid, fuse, core). glow hidden.
        for (i, pk) in self.inner.pickups.iter().enumerate() {
            let bobp = (t * 3.0 + i as f32 * 0.9).sin() * 0.12;
            let (c, sx, sy, ac, aox, aoy, asx, asy) = match pk.kind {
                PickupKind::Heart => ((255, 80, 90), 0.55, 0.50, (255, 235, 235), -0.10, 0.10, 0.16, 0.14),
                PickupKind::Essence => ((90, 150, 255), 0.48, 0.55, (200, 230, 255), 0.0, 0.14, 0.30, 0.12),
                PickupKind::Gold => ((255, 210, 90), 0.42, 0.42, (255, 255, 240), 0.09, 0.09, 0.12, 0.12),
                PickupKind::Bomb => ((60, 60, 70), 0.55, 0.55, (255, 150, 60), 0.0, 0.34, 0.14, 0.14),
                PickupKind::Chest => ((190, 130, 70), 0.85, 0.65, (255, 220, 130), 0.0, 0.20, 0.70, 0.14),
                PickupKind::Shrine => ((140, 255, 190), 0.70, 0.78, (240, 255, 245), 0.0, 0.0, 0.30, 0.30),
            };
            if let Some(h) = self.pickup_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_pos(r, pk.pos.0, pk.pos.1 + bobp, -2.0);
                    set_scale(r, sx, sy);
                    r.set_color(dc(c));
                }
            }
            if let Some(h) = self.pickup_accent_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_pos(r, pk.pos.0 + aox, pk.pos.1 + bobp + aoy, -2.1);
                    set_scale(r, asx, asy);
                    r.set_color(dc(ac));
                }
            }
            if let Some(h) = self.pickup_glow_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_scale(r, 0.001, 0.001);
                }
            }
        }
        // projectiles — opaque bright cores
        for (i, pr) in self.inner.projectiles.iter().enumerate() {
            if let Some(h) = self.proj_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_pos(r, pr.pos.0, pr.pos.1, -6.0);
                    set_scale(r, pr.size + 0.25, pr.size + 0.15);
                    r.set_color(dc(pr.color));
                }
            }
            if let Some(h) = self.proj_glow_handles.get(i) {
                if let Ok(r) = scene.graph.try_get_mut(*h) {
                    set_scale(r, 0.001, 0.001);
                }
            }
        }
        // particles — opaque small, hidden when dead
        for (i, h) in self.particle_handles.iter().enumerate() {
            if let Ok(r) = scene.graph.try_get_mut(*h) {
                if let Some(pt) = self.inner.particles.get(i) {
                    let f = (pt.life / pt.max_life).clamp(0.0, 1.0);
                    set_pos(r, pt.pos.0, pt.pos.1, -7.0);
                    let s = (pt.size * (0.5 + f * 0.5)).max(0.08);
                    set_scale(r, s, s);
                    r.set_color(dc(pt.color));
                } else {
                    set_scale(r, 0.001, 0.001);
                }
            }
        }
    }

}
