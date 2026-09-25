//! All UI: HUD, log, banner, floating text, Isaac-style menu, draft cards.

use fyrox::{
    core::{
        algebra::{Vector2, Vector3},
        pool::Handle,
    },
    engine::GraphicsContext,
    graph::SceneGraph,
    gui::{
        HorizontalAlignment, VerticalAlignment,
        border::{Border, BorderBuilder},
        brush::Brush,
        formatted_text::WrapMode,
        image::{ImageBuilder, ImageMessage},
        text::{Text, TextBuilder, TextMessage},
        widget::{WidgetBuilder, WidgetMessage},
        Thickness,
        UserInterface,
    },
    plugin::PluginContext,
};
use crate::{
    draft::DraftKind,
    gu::GU,
    panel::{PATCH_CREAM, PATCH_DIM_FILL, PATCH_DIM_STITCH, PATCH_GOLD, PATCH_INK, PATCH_ORANGE, PATCH_STITCH, DARK_BODY, DARK_DIM, DARK_GOLD, DARK_PAPER, DARK_RED, dark_heart_textures, dark_panel, heart_textures, hsep, patch_panel, patch_texture},
    plugin::Game,
    progression::MAX_LEVEL,
    stats::{AttrKind, DerivedStats},
    util::{col, darken, ui_scale, window_size},
};

// Shared story-hub geometry: identical rects for sync (draw) and the update
// branch (mouse hit-testing), so clicks always land on text.
pub(crate) struct StoryLayout {
    pub back_r: (f32, f32, f32, f32),
    pub sleep_bg_r: (f32, f32, f32, f32),
    pub title_r: (f32, f32, f32, f32),
    pub sep_title_r: (f32, f32, f32, f32),
    pub left_r: (f32, f32, f32, f32),
    pub body_r: (f32, f32, f32, f32),
    pub body_bg_r: (f32, f32, f32, f32),
    pub sub_rs: [(f32, f32, f32, f32); 3],
    pub col_rs: [(f32, f32, f32, f32); 3],
    pub row_rs: Vec<(f32, f32, f32, f32)>,
    pub sep_mid_r: (f32, f32, f32, f32),
    pub log_r: (f32, f32, f32, f32),
    pub status_r: (f32, f32, f32, f32),
    pub reinc_r: (f32, f32, f32, f32),
}

/// Shared story-hub layout: identical rects for sync (draw) and the update
/// branch (mouse hit-testing), so clicks always land on text.
pub(crate) fn story_layout(w: f32, h: f32, s: f32, counts: [usize; 3]) -> StoryLayout {
    let cx = w * 0.5;
    let col_w = (w - 372.0 * s) / 3.0;
    let mut col_x = [0.0f32; 3];
    for i in 0..3 {
        col_x[i] = 332.0 * s + i as f32 * (col_w + 12.0 * s);
    }
    let rows_y = 304.0 * s;
    let mut row_rs = Vec::new();
    for c in 0..3 {
        for k in 0..counts[c].min(5) {
            row_rs.push((col_x[c], rows_y + k as f32 * 34.0 * s, col_w, 30.0 * s));
        }
    }
    let log_w = (w - 48.0 * s) * 0.58;
    StoryLayout {
        back_r: (0.0, 0.0, w, h),
        sleep_bg_r: (16.0 * s, 318.0 * s, 300.0 * s, 14.0 * s),
        title_r: (cx - 400.0 * s, 30.0 * s, 800.0 * s, 40.0 * s),
        sep_title_r: (16.0 * s, 73.0 * s, w - 32.0 * s, 2.0),
        left_r: (16.0 * s, 80.0 * s, 300.0 * s, 230.0 * s),
        body_r: (344.0 * s, 84.0 * s, w - 344.0 * s - 218.0 * s, 182.0 * s),
        body_bg_r: (332.0 * s, 80.0 * s, w - 332.0 * s - 16.0 * s, 190.0 * s),
        sub_rs: [
            (col_x[0], 276.0 * s, col_w, 26.0 * s),
            (col_x[1], 276.0 * s, col_w, 26.0 * s),
            (col_x[2], 276.0 * s, col_w, 26.0 * s),
        ],
        col_rs: [
            (col_x[0] - 8.0 * s, 268.0 * s, col_w + 16.0 * s, 216.0 * s),
            (col_x[1] - 8.0 * s, 268.0 * s, col_w + 16.0 * s, 216.0 * s),
            (col_x[2] - 8.0 * s, 268.0 * s, col_w + 16.0 * s, 216.0 * s),
        ],
        row_rs,
        sep_mid_r: (16.0 * s, h - 158.0 * s, w - 32.0 * s, 2.0),
        log_r: (16.0 * s, h - 150.0 * s, log_w, 134.0 * s),
        status_r: (16.0 * s + log_w + 16.0 * s, h - 150.0 * s, w - 16.0 * s - (16.0 * s + log_w + 16.0 * s), 134.0 * s),
        reinc_r: (w - 190.0 * s, 80.0 * s, 174.0 * s, 40.0 * s),
    }
}

// Isaac-style minimap: 16x12 cells over x∈[-16,16], y∈[-12,12].
pub(crate) const MAP_W: usize = 16;
pub(crate) const MAP_H: usize = 12;
pub(crate) const MAP_CELL: f32 = 18.0;
pub(crate) const MAP_EMPTY: (u8, u8, u8) = (18, 26, 18);
pub(crate) const MAP_PLAYER: (u8, u8, u8) = (255, 255, 255);
pub(crate) const MAP_BOSS: (u8, u8, u8) = (255, 90, 40);
pub(crate) const MAP_ENEMY: (u8, u8, u8) = (220, 60, 60);
pub(crate) const MAP_PICKUP: (u8, u8, u8) = (255, 210, 90);

impl Game {

    pub(crate) fn ensure_hud(&mut self, ctx: &mut PluginContext) {
        // Invisible-HUD root cause: the executor can hand us an EMPTY UI
        // container (init-time `first()` even panicked on it). No UI means
        // every widget build below silently goes nowhere. Create our own.
        if ctx.user_interfaces.iter().count() == 0 {
            let s = window_size(ctx);
            ctx.user_interfaces.add(UserInterface::new(s));
        }
        // OS window title (the executor hardcodes "Fyrox Game")
        if let GraphicsContext::Initialized(ref g) = ctx.graphics_context {
            g.window.set_title("the constraining of daniel");
        }
        if !self.hearts.is_empty() {
            return;
        }
        // stitched-patch textures (generated once, shared by every panel)
        self.patch_tex = patch_texture(PATCH_ORANGE, PATCH_STITCH);
        self.patch_tex_gold = patch_texture(PATCH_ORANGE, PATCH_GOLD);
        self.patch_tex_dark = patch_texture(PATCH_DIM_FILL, PATCH_DIM_STITCH);
        let (full, half, empty) = heart_textures();
        self.heart_full = full;
        self.heart_half = half;
        self.heart_empty = empty;
        let (dfull, dhalf, dempty) = dark_heart_textures();
        self.heart_d_full = dfull;
        self.heart_d_half = dhalf;
        self.heart_d_empty = dempty;
        // --- heart row: 10 Isaac-style containers, top-left under the XP bar ---
        self.hearts.clear();
        self.heart_darks.clear();
        for i in 0..10 {
            let Some(uih) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let heart = ImageBuilder::new(
                WidgetBuilder::new()
                    .with_width(36.0)
                    .with_height(30.0)
                    .with_desired_position(Vector2::new(16.0 + i as f32 * 44.0, 40.0)),
            )
            .with_texture(self.heart_full.clone())
            .build(&mut uih.build_ctx());
            self.hearts.push(heart);
            // dark twin for the dimmed HUD (same slot, hidden unless dimmed)
            let Some(uid) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let dark = ImageBuilder::new(
                WidgetBuilder::new()
                    .with_width(36.0)
                    .with_height(30.0)
                    .with_desired_position(Vector2::new(16.0 + i as f32 * 44.0, 40.0))
                    .with_visibility(false),
            )
            .with_texture(self.heart_d_full.clone())
            .build(&mut uid.build_ctx());
            self.heart_darks.push(dark);
        }
        // level line under the hearts (bare ink text, Isaac-style)
        let Some(uihl) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.hud_line = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(500.0)
                .with_height(48.0)
                .with_desired_position(Vector2::new(16.0, 76.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
        )
        .with_text("Lv1")
        .with_font_size(40.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Left)
        .with_vertical_text_alignment(VerticalAlignment::Top)
        .build(&mut uihl.build_ctx());
        // --- left stats column on a slim patch ---
        let Some(uisp) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.left_stats_panel = patch_panel(&mut uisp.build_ctx(), &self.patch_tex, 156.0, 260.0);
        // dark twin for the dimmed HUD
        let Some(uispd) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.stats_dim = patch_panel(&mut uispd.build_ctx(), &self.patch_tex_dark, 156.0, 260.0);
        let Some(uispv) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uispv.send(self.stats_dim, WidgetMessage::Visibility(false));
        let Some(uisz) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uisz.send(self.left_stats_panel, WidgetMessage::DesiredPosition(Vector2::new(16.0, 130.0)));
        uisz.send(self.left_stats_panel, WidgetMessage::ZIndex(0));
        let Some(uist) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.left_stats_text = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(132.0)
                .with_height(236.0)
                .with_desired_position(Vector2::new(28.0, 142.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
        )
        .with_text("...")
        .with_font_size(30.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Left)
        .with_vertical_text_alignment(VerticalAlignment::Top)
        .build(&mut uist.build_ctx());
        let Some(uisz2) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uisz2.send(self.left_stats_text, WidgetMessage::ZIndex(1));
        // --- reference-style skill panel: class header, name + [key] rows ---
        let Some(uisk) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.sk_panel = patch_panel(&mut uisk.build_ctx(), &self.patch_tex, 248.0, 312.0);
        let Some(uiskd) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.skills_dim = patch_panel(&mut uiskd.build_ctx(), &self.patch_tex_dark, 248.0, 312.0);
        let Some(uiskv) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uiskv.send(self.skills_dim, WidgetMessage::Visibility(false));
        let Some(uiskz) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uiskz.send(self.sk_panel, WidgetMessage::DesiredPosition(Vector2::new(16.0, 400.0)));
        uiskz.send(self.sk_panel, WidgetMessage::ZIndex(0));
        let Some(uiskt) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.sk_title = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(224.0)
                .with_height(44.0)
                .with_desired_position(Vector2::new(28.0, 410.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
        )
        .with_text("")
        .with_font_size(26.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uiskt.build_ctx());
        self.sk_names.clear();
        self.sk_keys.clear();
        for i in 0..4 {
            let Some(uisn) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let nm = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(140.0)
                    .with_height(56.0)
                    .with_desired_position(Vector2::new(28.0, 462.0 + i as f32 * 62.0))
                    .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
            )
            .with_text("")
            .with_font_size(20.0_f32.into())
            .with_horizontal_text_alignment(HorizontalAlignment::Left)
            .with_vertical_text_alignment(VerticalAlignment::Center)
            .build(&mut uisn.build_ctx());
            self.sk_names.push(nm);
            let Some(uisk2) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let kb = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(76.0)
                    .with_height(56.0)
                    .with_desired_position(Vector2::new(172.0, 462.0 + i as f32 * 62.0))
                    .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
            )
            .with_text("")
            .with_font_size(18.0_f32.into())
            .with_horizontal_text_alignment(HorizontalAlignment::Right)
            .with_vertical_text_alignment(VerticalAlignment::Center)
            .build(&mut uisk2.build_ctx());
            self.sk_keys.push(kb);
        }
        // --- bottom-right ITEMS panel (positioned per tick: corner-anchored) ---
        let Some(uiit) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.items_panel = patch_panel(&mut uiit.build_ctx(), &self.patch_tex, 232.0, 214.0);
        let Some(uiitd) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.items_dim = patch_panel(&mut uiitd.build_ctx(), &self.patch_tex_dark, 232.0, 214.0);
        let Some(uiitv) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uiitv.send(self.items_dim, WidgetMessage::Visibility(false));
        let Some(uiitt) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.items_text = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(208.0)
                .with_height(190.0)
                .with_desired_position(Vector2::new(900.0, 500.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
        )
        .with_text("")
        .with_font_size(18.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Left)
        .with_vertical_text_alignment(VerticalAlignment::Top)
        .build(&mut uiitt.build_ctx());
        // --- top-right minimap: patch + 16x12 cell grid ---
        let Some(uimp) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.map_panel = patch_panel(&mut uimp.build_ctx(), &self.patch_tex, 312.0, 240.0);
        let Some(uimpd) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.map_dim = patch_panel(&mut uimpd.build_ctx(), &self.patch_tex_dark, 312.0, 240.0);
        let Some(uimpv) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uimpv.send(self.map_dim, WidgetMessage::Visibility(false));
        self.map_cells.clear();
        self.map_cache = vec![(0, 0, 0); MAP_W * MAP_H];
        for _cy in 0..MAP_H {
            for _cx in 0..MAP_W {
                let Some(uimc) = ctx.user_interfaces.iter_mut().next() else {
                    return;
                };
                let cell = BorderBuilder::new(
                    WidgetBuilder::new()
                        .with_width(MAP_CELL)
                        .with_height(MAP_CELL)
                        .with_desired_position(Vector2::new(900.0, 60.0))
                        .with_background(Brush::Solid(col(MAP_EMPTY)).into()),
                )
                .build(&mut uimc.build_ctx());
                self.map_cells.push(cell);
            }
        }
        // patch behind the log
        let Some(uip1) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.log_bg = patch_panel(&mut uip1.build_ctx(), &self.patch_tex, 1024.0, 234.0);
        let Some(uiz2) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uiz2.send(self.log_bg, WidgetMessage::DesiredPosition(Vector2::new(4.0, 308.0)));
        uiz2.send(self.log_bg, WidgetMessage::ZIndex(0));
        // second widget needs fresh borrow
        let Some(ui2) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        let ctx3 = &mut ui2.build_ctx();
        self.log_h = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(1000.0)
                .with_height(210.0)
                .with_desired_position(Vector2::new(16.0, 320.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
        )
        .with_text("...")
        .with_font_size(30.0_f32.into())
        .with_wrap(WrapMode::Word)
        .with_horizontal_text_alignment(HorizontalAlignment::Left)
        .with_vertical_text_alignment(VerticalAlignment::Top)
        .build(ctx3);
        let Some(uiz3) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uiz3.send(self.log_h, WidgetMessage::ZIndex(2));
        // dark twin over the log panel for the draft dim (visibility-toggled)
        let Some(uidim) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.log_dim = patch_panel(&mut uidim.build_ctx(), &self.patch_tex_dark, 1024.0, 234.0);
        let Some(uidimz) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uidimz.send(self.log_dim, WidgetMessage::Visibility(false));
        uidimz.send(self.log_dim, WidgetMessage::ZIndex(1));
        // level-up banner (center screen, shown briefly on level-up)
        let Some(ui3) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        let ctx4 = &mut ui3.build_ctx();
        self.banner_h = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(1000.0)
                .with_height(96.0)
                .with_desired_position(Vector2::new(140.0, 330.0)),
        )
        .with_text("")
        .with_font_size(64.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(ctx4);
        // floating combat text pool (damage numbers, +XP) — positioned per-frame
        self.float_labels.clear();
        self.float_cache.clear();
        for _ in 0..24 {
            let Some(uif) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let lbl = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(220.0)
                    .with_height(50.0)
                    .with_desired_position(Vector2::new(-260.0, -260.0)),
            )
            .with_text("")
            .with_font_size(40.0_f32.into())
            .with_horizontal_text_alignment(HorizontalAlignment::Center)
            .with_vertical_text_alignment(VerticalAlignment::Center)
            .build(&mut uif.build_ctx());
            self.float_labels.push(lbl);
            self.float_cache.push(String::new());
        }
        // --- Isaac-style menu widgets (centered per-frame in sync_menu) ---
        let Some(uim) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        let mctx = &mut uim.build_ctx();
        // dripping-blood title
        self.menu_title = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(1100.0)
                .with_height(100.0)
                .with_desired_position(Vector2::new(90.0, 100.0))
                .with_foreground(Brush::Solid(col((186, 26, 26))).into()),
        )
        .with_text("the constraining of daniel")
        .with_font_size(64.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(mctx);
        let Some(uim2) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.menu_sub = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(900.0)
                .with_height(40.0)
                .with_desired_position(Vector2::new(190.0, 190.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
        )
        .with_text("a tiny basement rogue-like")
        .with_font_size(28.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uim2.build_ctx());
        // selectable rows, each on its own stitched patch (refreshed in sync_menu)
        self.menu_rows.clear();
        self.menu_row_bgs.clear();
        self.menu_row_golds.clear();
        for _ in 0..3 {
            let Some(uir) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let bg = patch_panel(&mut uir.build_ctx(), &self.patch_tex, 1024.0, 88.0);
            self.menu_row_bgs.push(bg);
            // gold-stitched twin shown only on the selected row (visibility is
            // the only reliable layer switch; texture swaps don't stick)
            let Some(uirg) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let gold = patch_panel(&mut uirg.build_ctx(), &self.patch_tex_gold, 1024.0, 88.0);
            self.menu_row_golds.push(gold);
            let Some(uirv) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            uirv.send(gold, WidgetMessage::Visibility(false));
            let Some(uir2) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let row = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(1000.0)
                    .with_height(64.0)
                    .with_desired_position(Vector2::new(140.0, 300.0))
                    .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
            )
            .with_text("")
            .with_font_size(44.0_f32.into())
            .with_horizontal_text_alignment(HorizontalAlignment::Center)
            .with_vertical_text_alignment(VerticalAlignment::Center)
            .build(&mut uir2.build_ctx());
            self.menu_rows.push(row);
        }
        let Some(uimh) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.menu_help_bg = patch_panel(&mut uimh.build_ctx(), &self.patch_tex, 1124.0, 244.0);
        let Some(uimh2) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.menu_help = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(1100.0)
                .with_height(220.0)
                .with_desired_position(Vector2::new(90.0, 440.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
        )
        .with_text("")
        .with_font_size(26.0_f32.into())
        .with_wrap(WrapMode::Word)
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Top)
        .build(&mut uimh2.build_ctx());
        let Some(uimf) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.menu_footer = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(1100.0)
                .with_height(36.0)
                .with_desired_position(Vector2::new(90.0, 660.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
        )
        .with_text("")
        .with_font_size(26.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uimf.build_ctx());
        // --- level-up draft widgets (shown frozen over the game) ---
        let Some(uid) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.draft_title = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(900.0)
                .with_height(72.0)
                .with_desired_position(Vector2::new(190.0, 100.0))
                .with_foreground(Brush::Solid(col((255, 215, 90))).into()),
        )
        .with_text("")
        .with_font_size(54.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uid.build_ctx());
        // three-line cards: rarity (colored) / name / small desc, one widget
        // each so every line gets its own size + color.
        self.draft_rarity.clear();
        self.draft_names.clear();
        self.draft_descs.clear();
        for _ in 0..3 {
            let Some(uic) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let rar = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(460.0)
                    .with_height(30.0)
                    .with_desired_position(Vector2::new(190.0, 220.0))
                    .with_foreground(Brush::Solid(col(PATCH_CREAM)).into()),
            )
            .with_text("")
            .with_font_size(22.0_f32.into())
            .with_horizontal_text_alignment(HorizontalAlignment::Center)
            .with_vertical_text_alignment(VerticalAlignment::Center)
            .build(&mut uic.build_ctx());
            self.draft_rarity.push(rar);
            let Some(uin) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let nm = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(460.0)
                    .with_height(48.0)
                    .with_desired_position(Vector2::new(190.0, 254.0))
                    .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
            )
            .with_text("")
            .with_font_size(32.0_f32.into())
            .with_horizontal_text_alignment(HorizontalAlignment::Center)
            .with_vertical_text_alignment(VerticalAlignment::Center)
            .build(&mut uin.build_ctx());
            self.draft_names.push(nm);
            let Some(uid) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let ds = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(460.0)
                    .with_height(110.0)
                    .with_desired_position(Vector2::new(190.0, 306.0))
                    .with_foreground(Brush::Solid(col(PATCH_INK)).into()),
            )
            .with_text("")
            .with_font_size(20.0_f32.into())
            .with_wrap(WrapMode::Word)
            .with_horizontal_text_alignment(HorizontalAlignment::Center)
            .with_vertical_text_alignment(VerticalAlignment::Top)
            .build(&mut uid.build_ctx());
            self.draft_descs.push(ds);
        }
        // stitched patch behind each card + gold twin for the selection
        self.draft_panels.clear();
        self.draft_golds.clear();
        self.draft_darks.clear();
        for _ in 0..3 {
            let Some(uip) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let panel = patch_panel(&mut uip.build_ctx(), &self.patch_tex, 484.0, 264.0);
            self.draft_panels.push(panel);
            let Some(uipg) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let gold = patch_panel(&mut uipg.build_ctx(), &self.patch_tex_gold, 484.0, 264.0);
            self.draft_golds.push(gold);
            // dark twin: shown on UNSELECTED cards so the pick pops
            let Some(uipd) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let dark = patch_panel(&mut uipd.build_ctx(), &self.patch_tex_dark, 484.0, 264.0);
            self.draft_darks.push(dark);
            let Some(uipv) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            uipv.send(panel, WidgetMessage::Visibility(false));
            uipv.send(gold, WidgetMessage::Visibility(false));
            uipv.send(dark, WidgetMessage::Visibility(false));
            // card text must sit above ALL panel layers (same-z siblings
            // render in build order, so text gets its own higher layer)
            uipv.send(panel, WidgetMessage::ZIndex(2));
            uipv.send(gold, WidgetMessage::ZIndex(2));
            uipv.send(dark, WidgetMessage::ZIndex(2));
        }
        let Some(uidh) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.draft_hint = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(900.0)
                .with_height(36.0)
                .with_desired_position(Vector2::new(190.0, 500.0))
                .with_foreground(Brush::Solid(col((130, 125, 120))).into()),
        )
        .with_text("")
        .with_font_size(26.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uidh.build_ctx());
        // --- XP bar: two flat borders pinned to the very top (widths per tick) ---
        let Some(uix) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.xp_bar_bg = BorderBuilder::new(
            WidgetBuilder::new()
                .with_width(1280.0)
                .with_height(26.0)
                .with_desired_position(Vector2::new(0.0, 0.0))
                .with_background(Brush::Solid(col((10, 10, 14))).into())
                .with_visibility(false),
        )
        .build(&mut uix.build_ctx());
        let Some(uix2) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.xp_bar_fg = BorderBuilder::new(
            WidgetBuilder::new()
                .with_width(4.0)
                .with_height(18.0)
                .with_desired_position(Vector2::new(4.0, 4.0))
                .with_background(Brush::Solid(col((255, 210, 90))).into())
                .with_visibility(false),
        )
        .build(&mut uix2.build_ctx());
        // --- stats menu (Tab): plus/minus allocation on a stitched patch ---
        let Some(uis) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.stats_panel = patch_panel(&mut uis.build_ctx(), &self.patch_tex, 568.0, 384.0);
        let Some(uisv) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uisv.send(self.stats_panel, WidgetMessage::Visibility(false));
        let Some(uist) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.stats_title = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(520.0)
                .with_height(44.0)
                .with_desired_position(Vector2::new(200.0, 200.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into())
                .with_visibility(false),
        )
        .with_text("")
        .with_font_size(34.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uist.build_ctx());
        self.stats_rows.clear();
        for _ in 0..5 {
            let Some(uisr) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let row = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(520.0)
                    .with_height(40.0)
                    .with_desired_position(Vector2::new(200.0, 260.0))
                    .with_foreground(Brush::Solid(col(PATCH_INK)).into())
                    .with_visibility(false),
            )
            .with_text("")
            .with_font_size(28.0_f32.into())
            .with_horizontal_text_alignment(HorizontalAlignment::Center)
            .with_vertical_text_alignment(VerticalAlignment::Center)
            .build(&mut uisr.build_ctx());
            self.stats_rows.push(row);
        }
        let Some(uish) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.stats_hint = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(520.0)
                .with_height(30.0)
                .with_desired_position(Vector2::new(200.0, 480.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into())
                .with_visibility(false),
        )
        .with_text("Up/Down pick · +/- adjust · Tab/Esc close")
        .with_font_size(20.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uish.build_ctx());
        // --- pause menu (Esc/P): resume / restart / quit, frozen game ---
        let Some(uipz) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.pause_title = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(700.0)
                .with_height(72.0)
                .with_desired_position(Vector2::new(290.0, 200.0))
                .with_foreground(Brush::Solid(col((255, 215, 90))).into())
                .with_visibility(false),
        )
        .with_text("PAUSED")
        .with_font_size(60.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uipz.build_ctx());
        self.pause_rows.clear();
        self.pause_row_bgs.clear();
        self.pause_row_golds.clear();
        self.pause_darks.clear();
        for _ in 0..5 {
            let Some(uipr) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let bg = patch_panel(&mut uipr.build_ctx(), &self.patch_tex, 724.0, 88.0);
            self.pause_row_bgs.push(bg);
            let Some(uiprg) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let gold = patch_panel(&mut uiprg.build_ctx(), &self.patch_tex_gold, 724.0, 88.0);
            self.pause_row_golds.push(gold);
            let Some(uiprd) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let dark = patch_panel(&mut uiprd.build_ctx(), &self.patch_tex_dark, 724.0, 88.0);
            self.pause_darks.push(dark);
            let Some(uiprv) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            uiprv.send(bg, WidgetMessage::Visibility(false));
            uiprv.send(gold, WidgetMessage::Visibility(false));
            uiprv.send(dark, WidgetMessage::Visibility(false));
            let Some(uiprt) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let row = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(700.0)
                    .with_height(64.0)
                    .with_desired_position(Vector2::new(290.0, 300.0))
                    .with_foreground(Brush::Solid(col(PATCH_INK)).into())
                    .with_visibility(false),
            )
            .with_text("")
            .with_font_size(40.0_f32.into())
            .with_horizontal_text_alignment(HorizontalAlignment::Center)
            .with_vertical_text_alignment(VerticalAlignment::Center)
            .build(&mut uiprt.build_ctx());
            self.pause_rows.push(row);
        }
        let Some(uiph) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.pause_hint = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(700.0)
                .with_height(30.0)
                .with_desired_position(Vector2::new(290.0, 600.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into())
                .with_visibility(false),
        )
        .with_text("Up/Down choose · ENTER confirm · Esc resume")
        .with_font_size(22.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uiph.build_ctx());
        // --- settings overlay (from pause): log / floaters / shake toggles ---
        let Some(uist) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.settings_title = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(700.0)
                .with_height(72.0)
                .with_desired_position(Vector2::new(290.0, 200.0))
                .with_foreground(Brush::Solid(col((255, 215, 90))).into())
                .with_visibility(false),
        )
        .with_text("SETTINGS")
        .with_font_size(60.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uist.build_ctx());
        self.settings_rows.clear();
        self.settings_row_bgs.clear();
        for _ in 0..4 {
            let Some(uisr) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let bg = patch_panel(&mut uisr.build_ctx(), &self.patch_tex, 724.0, 88.0);
            self.settings_row_bgs.push(bg);
            let Some(uisrv) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            uisrv.send(bg, WidgetMessage::Visibility(false));
            let Some(uisrt) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let row = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(700.0)
                    .with_height(64.0)
                    .with_desired_position(Vector2::new(290.0, 300.0))
                    .with_foreground(Brush::Solid(col(PATCH_INK)).into())
                    .with_visibility(false),
            )
            .with_text("")
            .with_font_size(40.0_f32.into())
            .with_horizontal_text_alignment(HorizontalAlignment::Center)
            .with_vertical_text_alignment(VerticalAlignment::Center)
            .build(&mut uisrt.build_ctx());
            self.settings_rows.push(row);
        }
        let Some(uish) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.settings_hint = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(700.0)
                .with_height(30.0)
                .with_desired_position(Vector2::new(290.0, 620.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into())
                .with_visibility(false),
        )
        .with_text("ENTER toggle · Esc back")
        .with_font_size(22.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uish.build_ctx());
        // --- path select: twelve Gu paths, one soul ---
        let Some(uiph) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.path_title = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(700.0)
                .with_height(64.0)
                .with_desired_position(Vector2::new(290.0, 60.0))
                .with_foreground(Brush::Solid(col((255, 215, 90))).into())
                .with_visibility(false),
        )
        .with_text("CHOOSE YOUR PATH")
        .with_font_size(48.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uiph.build_ctx());
        self.path_rows.clear();
        for _ in 0..12 {
            let Some(uipr2) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let row = TextBuilder::new(
                WidgetBuilder::new()
                    .with_width(640.0)
                    .with_height(36.0)
                    .with_desired_position(Vector2::new(320.0, 150.0))
                    .with_foreground(Brush::Solid(col(PATCH_INK)).into())
                    .with_visibility(false),
            )
            .with_text("")
            .with_font_size(20.0_f32.into())
            .with_horizontal_text_alignment(HorizontalAlignment::Center)
            .with_vertical_text_alignment(VerticalAlignment::Center)
            .build(&mut uipr2.build_ctx());
            self.path_rows.push(row);
        }
        let Some(uiph2) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.path_hint = TextBuilder::new(
            WidgetBuilder::new()
                .with_width(700.0)
                .with_height(30.0)
                .with_desired_position(Vector2::new(290.0, 650.0))
                .with_foreground(Brush::Solid(col(PATCH_INK)).into())
                .with_visibility(false),
        )
        .with_text("")
        .with_font_size(20.0_f32.into())
        .with_horizontal_text_alignment(HorizontalAlignment::Center)
        .with_vertical_text_alignment(VerticalAlignment::Center)
        .build(&mut uiph2.build_ctx());
        // --- story hub: black panels + separators (reference look), light text ---
        macro_rules! stext {
            ($w:expr, $h:expr, $fs:expr, $align:expr) => {{
                let ui_opt = ctx.user_interfaces.iter_mut().next();
                let Some(uisb_) = ui_opt else {
                    return;
                };
                TextBuilder::new(
                    WidgetBuilder::new()
                        .with_width($w)
                        .with_height($h)
                        .with_desired_position(Vector2::new(0.0, 0.0))
                        .with_foreground(Brush::Solid(col(DARK_BODY)).into())
                        .with_visibility(false),
                )
                .with_text("")
                .with_font_size($fs.into())
                .with_horizontal_text_alignment($align)
                .with_vertical_text_alignment(VerticalAlignment::Top)
                .build(&mut uisb_.build_ctx())
            }};
        }
        let Some(uisb) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        self.story_left_bg = dark_panel(&mut uisb.build_ctx(), 324.0, 254.0);
        self.story_body_bg = dark_panel(&mut uisb.build_ctx(), 800.0, 190.0);
        self.story_backdrop = dark_panel(&mut uisb.build_ctx(), 1280.0, 720.0);
        let Some(uisv0) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uisv0.send(self.story_left_bg, WidgetMessage::Visibility(false));
        uisv0.send(self.story_body_bg, WidgetMessage::Visibility(false));
        uisv0.send(self.story_backdrop, WidgetMessage::Visibility(false));
        // sleep bar: dark trough + green fill, sized per-frame in sync
        self.story_sleep_bg = BorderBuilder::new(
            WidgetBuilder::new()
                .with_width(300.0)
                .with_height(14.0)
                .with_desired_position(Vector2::new(0.0, 0.0))
                .with_background(Brush::Solid(col((28, 30, 38))).into())
                .with_foreground(Brush::Solid(col((150, 155, 172))).into())
                .with_visibility(false),
        )
        .with_stroke_thickness(Thickness::uniform(1.0).into())
        .build(&mut uisv0.build_ctx());
        self.story_sleep_fg = BorderBuilder::new(
            WidgetBuilder::new()
                .with_width(4.0)
                .with_height(10.0)
                .with_desired_position(Vector2::new(0.0, 0.0))
                .with_background(Brush::Solid(col((110, 200, 130))).into())
                .with_visibility(false),
        )
        .build(&mut uisv0.build_ctx());
        self.story_seps.clear();
        for _ in 0..2 {
            let Some(uisep) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            self.story_seps.push(hsep(&mut uisep.build_ctx(), 800.0));
        }
        let Some(uisv0b) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        for sp in self.story_seps.iter() {
            uisv0b.send(*sp, WidgetMessage::Visibility(false));
        }
        // starfield: 90 deterministic specks across the screen, two tiers —
        // positioned per-frame in sync_story, parked everywhere else
        self.story_stars.clear();
        for i in 0..90 {
            let Some(uist) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            let big = i % 3 == 0;
            let px = if big { 3.0 } else { 2.0 };
            let tint = if big { (200, 210, 230) } else { (120, 130, 160) };
            let star = BorderBuilder::new(
                WidgetBuilder::new()
                    .with_width(px)
                    .with_height(px)
                    .with_desired_position(Vector2::new(0.0, 0.0))
                    .with_background(Brush::Solid(col(tint)).into())
                    .with_visibility(false),
            )
            .build(&mut uist.build_ctx());
            self.story_stars.push(star);
        }
        self.story_title = stext!(800.0, 44.0, 34.0_f32, HorizontalAlignment::Center);
        self.story_body = stext!(800.0, 190.0, 19.0_f32, HorizontalAlignment::Left);
        self.story_cols_bg.clear();
        self.story_subs.clear();
        for _ in 0..3 {
            let Some(uisbg) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            self.story_cols_bg.push(dark_panel(&mut uisbg.build_ctx(), 300.0, 300.0));
            let Some(uisub) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            self.story_subs.push(
                TextBuilder::new(
                    WidgetBuilder::new()
                        .with_width(300.0)
                        .with_height(30.0)
                        .with_desired_position(Vector2::new(0.0, 0.0))
                        .with_foreground(Brush::Solid(col(DARK_PAPER)).into())
                        .with_visibility(false),
                )
                .with_text("")
                .with_font_size(20.0_f32.into())
                .with_horizontal_text_alignment(HorizontalAlignment::Center)
                .with_vertical_text_alignment(VerticalAlignment::Center)
                .build(&mut uisub.build_ctx()),
            );
        }
        let Some(uisv1) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        for bg in self.story_cols_bg.iter() {
            uisv1.send(*bg, WidgetMessage::Visibility(false));
        }
        self.story_left_title = stext!(300.0, 36.0, 26.0_f32, HorizontalAlignment::Center);
        self.story_left_body = stext!(300.0, 200.0, 20.0_f32, HorizontalAlignment::Left);
        self.story_log = stext!(600.0, 170.0, 16.0_f32, HorizontalAlignment::Left);
        self.story_status = stext!(300.0, 170.0, 18.0_f32, HorizontalAlignment::Left);
        self.story_reinc = stext!(180.0, 40.0, 20.0_f32, HorizontalAlignment::Center);
        self.story_rows.clear();
        for _ in 0..15 {
            let Some(uisr) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            self.story_rows.push(
                TextBuilder::new(
                    WidgetBuilder::new()
                        .with_width(300.0)
                        .with_height(42.0)
                        .with_desired_position(Vector2::new(0.0, 0.0))
                        .with_foreground(Brush::Solid(col(DARK_BODY)).into())
                        .with_visibility(false),
                )
                .with_text("")
                .with_font_size(18.0_f32.into())
                .with_horizontal_text_alignment(HorizontalAlignment::Center)
                .with_vertical_text_alignment(VerticalAlignment::Center)
                .build(&mut uisr.build_ctx()),
            );
        }
        self.story_log_bg = {
            let Some(uilb) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            dark_panel(&mut uilb.build_ctx(), 600.0, 190.0)
        };
        self.story_status_bg = {
            let Some(uisb2) = ctx.user_interfaces.iter_mut().next() else {
                return;
            };
            dark_panel(&mut uisb2.build_ctx(), 300.0, 190.0)
        };
        let Some(uisv2) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        uisv2.send(self.story_log_bg, WidgetMessage::Visibility(false));
        uisv2.send(self.story_status_bg, WidgetMessage::Visibility(false));
    }

    /// Shows/hides every in-game HUD widget in one sweep (menu hides all).
    fn set_play_ui_visible_ui(&mut self, ui: &mut UserInterface, vis: bool) {
        // saga sigs always reset here so reopened overlays fully repaint
        self.path_sig.clear();
        self.story_sig.clear();
        for h in self.hearts.iter() {
            ui.send(*h, WidgetMessage::Visibility(vis));
        }
        // dark hearts + dim twins are owned by the dim helper, never the sweep
        for h in self.heart_darks.iter() {
            ui.send(*h, WidgetMessage::Visibility(false));
        }
        // dim twins only ever show via the dim helper; the sweep hides them
        for twin in [self.stats_dim, self.skills_dim, self.map_dim, self.items_dim] {
            ui.send(twin, WidgetMessage::Visibility(false));
        }
        ui.send(self.hud_line, WidgetMessage::Visibility(vis));
        ui.send(self.left_stats_panel, WidgetMessage::Visibility(vis));
        ui.send(self.left_stats_text, WidgetMessage::Visibility(vis));
        ui.send(self.sk_panel, WidgetMessage::Visibility(vis));
        ui.send(self.sk_title, WidgetMessage::Visibility(vis));
        for h in self.sk_names.iter().chain(self.sk_keys.iter()) {
            ui.send(*h, WidgetMessage::Visibility(vis));
        }
        ui.send(self.items_panel, WidgetMessage::Visibility(vis));
        ui.send(self.items_text, WidgetMessage::Visibility(vis));
        ui.send(self.map_panel, WidgetMessage::Visibility(vis));
        for c in self.map_cells.iter() {
            ui.send(*c, WidgetMessage::Visibility(vis));
        }
        ui.send(self.log_h, WidgetMessage::Visibility(vis && self.log_on));
        ui.send(self.log_bg, WidgetMessage::Visibility(vis && self.log_on));
        ui.send(self.log_dim, WidgetMessage::Visibility(false));
        // saga overlays own their own show/hide; the sweep parks them all
        ui.send(self.path_title, WidgetMessage::Visibility(false));
        for h in self.path_rows.iter() {
            ui.send(*h, WidgetMessage::Visibility(false));
        }
        ui.send(self.path_hint, WidgetMessage::Visibility(false));
        ui.send(self.story_title, WidgetMessage::Visibility(false));
        ui.send(self.story_body, WidgetMessage::Visibility(false));
        for h in self.story_subs.iter() {
            ui.send(*h, WidgetMessage::Visibility(false));
        }
        for h in self.story_rows.iter() {
            ui.send(*h, WidgetMessage::Visibility(false));
        }
        ui.send(self.story_left_title, WidgetMessage::Visibility(false));
        ui.send(self.story_left_body, WidgetMessage::Visibility(false));
        ui.send(self.story_log, WidgetMessage::Visibility(false));
        ui.send(self.story_status, WidgetMessage::Visibility(false));
        ui.send(self.story_reinc, WidgetMessage::Visibility(false));
        ui.send(self.story_left_bg, WidgetMessage::Visibility(false));
        ui.send(self.story_body_bg, WidgetMessage::Visibility(false));
        for bg in self.story_cols_bg.iter() {
            ui.send(*bg, WidgetMessage::Visibility(false));
        }
        for sp in self.story_seps.iter() {
            ui.send(*sp, WidgetMessage::Visibility(false));
        }
        for sh in self.story_stars.iter() {
            ui.send(*sh, WidgetMessage::Visibility(false));
        }
        ui.send(self.story_log_bg, WidgetMessage::Visibility(false));
        ui.send(self.story_status_bg, WidgetMessage::Visibility(false));
    }

    pub(crate)     fn sync_floats(&mut self, ctx: &mut PluginContext) {
        if self.float_labels.is_empty() || !self.floaters_on {
            return;
        }
        // Window size for world->screen projection (skip if graphics suspended).
        let screen_size = match ctx.graphics_context {
            GraphicsContext::Initialized(ref g) => {
                let s = g.window.inner_size();
                Vector2::new(s.width as f32, s.height as f32)
            }
            _ => return,
        };
        // Project while the scene is borrowed, then release before touching UI.
        let mut items: Vec<(f32, f32, String)> = Vec::new();
        if let Ok(scene) = ctx.scenes.try_get_mut(self.scene) {
            if let Ok(cam) = scene.graph.try_get(self.camera_handle) {
                for f in self.inner.floats.iter().take(self.float_labels.len()) {
                    if let Some(p) = cam.project(Vector3::new(f.pos.0, f.pos.1, -3.0), screen_size) {
                        items.push((p.x, p.y, f.text.clone()));
                    }
                }
            }
        }
        let s = ui_scale(ctx);
        let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        for (i, h) in self.float_labels.iter().enumerate() {
            match items.get(i) {
                Some((x, y, txt)) => {
                    ui.send(*h, WidgetMessage::DesiredPosition(Vector2::new(x - 110.0 * s, y - 100.0 * s)));
                    ui.send(*h, WidgetMessage::Width(220.0 * s));
                    ui.send(*h, WidgetMessage::Height(50.0 * s));
                    // only resend text when it changes — avoids per-frame layout churn
                    if self.float_cache.get(i).map(|c| c != txt).unwrap_or(true) {
                        ui.send(*h, TextMessage::Text(txt.clone()));
                        if let Some(c) = self.float_cache.get_mut(i) {
                            *c = txt.clone();
                        }
                    }
                }
                None => {
                    if self.float_cache.get(i).map(|c| !c.is_empty()).unwrap_or(false) {
                        ui.send(*h, TextMessage::Text(String::new()));
                        if let Some(c) = self.float_cache.get_mut(i) {
                            c.clear();
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn set_menu_visible(&mut self, ctx: &mut PluginContext, vis: bool) {
        let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        ui.send(self.menu_title, WidgetMessage::Visibility(vis));
        ui.send(self.menu_sub, WidgetMessage::Visibility(vis));
        for h in self.menu_rows.iter() {
            ui.send(*h, WidgetMessage::Visibility(vis));
        }
        for bg in self.menu_row_bgs.iter() {
            ui.send(*bg, WidgetMessage::Visibility(vis && !self.menu_help_on));
        }
        for (i, gold) in self.menu_row_golds.iter().enumerate() {
            ui.send(*gold, WidgetMessage::Visibility(vis && !self.menu_help_on && i == self.menu_index));
        }
        ui.send(self.menu_help, WidgetMessage::Visibility(vis && self.menu_help_on));
        ui.send(self.menu_help_bg, WidgetMessage::Visibility(vis && self.menu_help_on));
        ui.send(self.menu_footer, WidgetMessage::Visibility(vis));
    }

    /// Isaac-style title menu: blood-red logo, gold selector, class cycling.
    /// Positions re-resolve on resize; texts only resend on state change.
    pub(crate) fn sync_menu(&mut self, ctx: &mut PluginContext) {
        if self.menu_rows.len() < 3 || self.menu_title.is_none() {
            return;
        }
        let size = window_size(ctx);
        let (w, h) = (size.x, size.y);
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        // authoritative visibility: the menu owns these widgets — the moment
        // a run/path/saga takes over they park, no matter what repainted last
        let active = !self.inner.started && !self.path_open && !self.story_open;
        if !active {
            if !self.menu_sig.is_empty() {
                let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
                    return;
                };
                ui.send(self.menu_title, WidgetMessage::Visibility(false));
                ui.send(self.menu_sub, WidgetMessage::Visibility(false));
                for h in self.menu_rows.iter() {
                    ui.send(*h, WidgetMessage::Visibility(false));
                }
                for bg in self.menu_row_bgs.iter().chain(self.menu_row_golds.iter()) {
                    ui.send(*bg, WidgetMessage::Visibility(false));
                }
                ui.send(self.menu_help, WidgetMessage::Visibility(false));
                ui.send(self.menu_help_bg, WidgetMessage::Visibility(false));
                ui.send(self.menu_footer, WidgetMessage::Visibility(false));
                self.menu_sig.clear();
            }
            return;
        }
        let has_saga = self.inner.path.is_some();
        let sig = format!(
            "{:.0}x{:.0}|{}|{}|{}",
            w, h, self.menu_index, self.menu_help_on as u8, has_saga as u8
        );
        if sig == self.menu_sig {
            return;
        }
        self.menu_sig = sig;
        let s = ui_scale(ctx);
        let cx = w * 0.5;
        let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        ui.send(self.menu_title, WidgetMessage::DesiredPosition(Vector2::new(cx - 550.0 * s, h * 0.10)));
        ui.send(self.menu_title, WidgetMessage::Width(1100.0 * s));
        ui.send(self.menu_title, WidgetMessage::Height(100.0 * s));
        ui.send(self.menu_sub, WidgetMessage::DesiredPosition(Vector2::new(cx - 450.0 * s, h * 0.10 + 128.0 * s)));
        ui.send(self.menu_sub, WidgetMessage::Width(900.0 * s));
        ui.send(self.menu_sub, WidgetMessage::Height(40.0 * s));
        // rows on stitched patches, or the help patch modally in their place
        let rows_y = h * 0.34;
        for (i, rh) in self.menu_rows.iter().enumerate() {
            let ry = rows_y + i as f32 * 100.0 * s;
            let shown = !self.menu_help_on;
            let sel = i == self.menu_index;
            ui.send(*rh, WidgetMessage::DesiredPosition(Vector2::new(cx - 500.0 * s, ry)));
            ui.send(*rh, WidgetMessage::Width(1000.0 * s));
            ui.send(*rh, WidgetMessage::Height(64.0 * s));
            ui.send(*rh, WidgetMessage::Visibility(shown));
            ui.send(*rh, WidgetMessage::ZIndex(1));
            if let Some(bg) = self.menu_row_bgs.get(i) {
                ui.send(*bg, WidgetMessage::DesiredPosition(Vector2::new(cx - 512.0 * s, ry - 12.0 * s)));
                ui.send(*bg, WidgetMessage::Width(1024.0 * s));
                ui.send(*bg, WidgetMessage::Height(88.0 * s));
                ui.send(*bg, WidgetMessage::Visibility(shown));
                ui.send(*bg, WidgetMessage::ZIndex(0));
            }
            // gold-stitched twin on the selected row (visibility, not texture swap)
            if let Some(gold) = self.menu_row_golds.get(i) {
                ui.send(*gold, WidgetMessage::DesiredPosition(Vector2::new(cx - 512.0 * s, ry - 12.0 * s)));
                ui.send(*gold, WidgetMessage::Width(1024.0 * s));
                ui.send(*gold, WidgetMessage::Height(88.0 * s));
                ui.send(*gold, WidgetMessage::Visibility(shown && sel));
                ui.send(*gold, WidgetMessage::ZIndex(0));
            }
        }
        ui.send(self.menu_help, WidgetMessage::DesiredPosition(Vector2::new(cx - 550.0 * s, rows_y)));
        ui.send(self.menu_help, WidgetMessage::Width(1100.0 * s));
        ui.send(self.menu_help, WidgetMessage::Height(220.0 * s));
        ui.send(self.menu_help_bg, WidgetMessage::DesiredPosition(Vector2::new(cx - 562.0 * s, rows_y - 12.0 * s)));
        ui.send(self.menu_help_bg, WidgetMessage::Width(1124.0 * s));
        ui.send(self.menu_help_bg, WidgetMessage::Height(244.0 * s));
        ui.send(self.menu_help_bg, WidgetMessage::ZIndex(0));
        ui.send(self.menu_footer, WidgetMessage::DesiredPosition(Vector2::new(cx - 550.0 * s, h - 56.0 * s)));
        ui.send(self.menu_footer, WidgetMessage::Width(1100.0 * s));
        ui.send(self.menu_footer, WidgetMessage::Height(36.0 * s));
        // row texts + gold highlight on the selected row
        let labels = [
            "START NEW RUN".to_string(),
            if has_saga {
                "CONTINUE".to_string()
            } else {
                "CONTINUE (no saga)".to_string()
            },
            "HOW TO PLAY".to_string(),
        ];
        for (i, rh) in self.menu_rows.iter().enumerate() {
            let sel = i == self.menu_index;
            let label = if sel {
                format!("> {} <", labels[i])
            } else {
                format!("  {}  ", labels[i])
            };
            ui.send(*rh, TextMessage::Text(label));
            // cream when selected, dim when CONTINUE has nothing to resume,
            // dark ink otherwise
            let fg = if sel {
                PATCH_CREAM
            } else if i == 1 && !has_saga {
                PATCH_DIM_STITCH
            } else {
                PATCH_INK
            };
            ui.send(*rh, WidgetMessage::Foreground(Brush::Solid(col(fg)).into()));
        }
        ui.send(
            self.menu_help,
            TextMessage::Text(
                "HOW TO PLAY\nWASD move · ARROWS / SPACE attack — swing or shoot by worm\nZ/X/C/V essence arts (Strike/Barrage/Ward/Mend) · H potion · T auto-spend · Tab +/- menu\nSlay foes for XP + essence · each level drafts 1 of 3 · milestones grant worm boons\nLv 5/10/15/20/25: choose a rank 2-6 Gu · refine 3 worms upward at the Hall\nSpar for purses · Slay bosses · Chests hide rare worms · don't die".to_string(),
            ),
        );
        ui.send(self.menu_help, WidgetMessage::Visibility(self.menu_help_on));
        ui.send(self.menu_help, WidgetMessage::ZIndex(1));
        ui.send(self.menu_help_bg, WidgetMessage::Visibility(self.menu_help_on));
        ui.send(
            self.menu_footer,
            TextMessage::Text("Up/Down choose · ENTER confirm · v0.1 basement build".to_string()),
        );
    }

    /// Frozen 3-card draft overlay. Authoritative on visibility: hides itself
    /// the moment the draft closes, shows on first open (sig starts empty).
    pub(crate) fn sync_draft(&mut self, ctx: &mut PluginContext) {
        if !self.inner.draft_open {
            if !self.draft_sig.is_empty() {
                let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
                    return;
                };
                ui.send(self.draft_title, WidgetMessage::Visibility(false));
                for h in self.draft_rarity.iter().chain(self.draft_names.iter()).chain(self.draft_descs.iter()) {
                    ui.send(*h, WidgetMessage::Visibility(false));
                }
                for bp in self.draft_panels.iter().chain(self.draft_golds.iter()).chain(self.draft_darks.iter()) {
                    ui.send(*bp, WidgetMessage::Visibility(false));
                }
                ui.send(self.draft_hint, WidgetMessage::Visibility(false));
                self.draft_sig.clear();
            }
            return;
        }
        if self.draft_rarity.len() < 3 || self.draft_title.is_none() {
            return;
        }
        let size = window_size(ctx);
        let (w, h) = (size.x, size.y);
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        // drafts reopen per level; every open bumps the id.
        // The 1s input lock is part of the signature so the "GET READY"
        // state repaints exactly once when it lifts.
        let locked = self.inner.draft_lock > 0.0;
        let boon = self.inner.draft_cards.iter().any(|c| matches!(c.kind, DraftKind::GuWorm(_)));
        let sig = format!(
            "{:.0}x{:.0}|{}|{}|{}|{}",
            w, h, boon as u8, self.inner.draft_sel, self.inner.draft_id, locked as u8
        );
        if sig == self.draft_sig {
            return;
        }
        self.draft_sig = sig;
        let s = ui_scale(ctx);
        let cx = w * 0.5;
        let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        ui.send(self.draft_title, WidgetMessage::DesiredPosition(Vector2::new(cx - 450.0 * s, h * 0.07)));
        ui.send(self.draft_title, WidgetMessage::Width(900.0 * s));
        ui.send(self.draft_title, WidgetMessage::Height(72.0 * s));
        ui.send(self.draft_title, WidgetMessage::Visibility(true));
        ui.send(self.draft_title, WidgetMessage::ZIndex(3));
        ui.send(
            self.draft_title,
            TextMessage::Text(if locked {
                "GET READY...".to_string()
            } else if boon {
                "* MILESTONE! CHOOSE A WORM *".to_string()
            } else {
                "* LEVEL UP! PICK ONE *".to_string()
            }),
        );
        // cards float above the dimmed HUD (z=3 over panels at z=2).
        // (log + panel dimming is owned by the update_hud dim helper.)
        // three cards side by side: left · middle · right, each on a panel
        let cw: f32 = (460.0f32).min((w - 96.0) / 3.0).max(200.0);
        let gap = 24.0 * s;
        let panel_h = 264.0 * s;
        let cards_y = h * 0.19;
        let x0 = cx - (3.0 * cw + 2.0 * gap) * 0.5;
        for i in 0..3 {
            let px = x0 + i as f32 * (cw + gap);
            let sel = i == self.inner.draft_sel;
            // brown patch + gold twin (gold only on the selection)
            if let Some(bp) = self.draft_panels.get(i) {
                ui.send(*bp, WidgetMessage::DesiredPosition(Vector2::new(px - 12.0 * s, cards_y - 12.0 * s)));
                ui.send(*bp, WidgetMessage::Width(cw + 24.0 * s));
                ui.send(*bp, WidgetMessage::Height(panel_h));
                ui.send(*bp, WidgetMessage::Visibility(true));
                ui.send(*bp, WidgetMessage::ZIndex(2));
            }
            if let Some(gold) = self.draft_golds.get(i) {
                ui.send(*gold, WidgetMessage::DesiredPosition(Vector2::new(px - 12.0 * s, cards_y - 12.0 * s)));
                ui.send(*gold, WidgetMessage::Width(cw + 24.0 * s));
                ui.send(*gold, WidgetMessage::Height(panel_h));
                ui.send(*gold, WidgetMessage::Visibility(sel && !locked));
                ui.send(*gold, WidgetMessage::ZIndex(2));
            }
            // dark twin on every UNSELECTED card so the pick pops — plus all
            // cards while the 1s input lock is up
            if let Some(dark) = self.draft_darks.get(i) {
                ui.send(*dark, WidgetMessage::DesiredPosition(Vector2::new(px - 12.0 * s, cards_y - 12.0 * s)));
                ui.send(*dark, WidgetMessage::Width(cw + 24.0 * s));
                ui.send(*dark, WidgetMessage::Height(panel_h));
                ui.send(*dark, WidgetMessage::Visibility(!sel || locked));
                ui.send(*dark, WidgetMessage::ZIndex(2));
            }
            // three-line card: colored rarity / name / small desc, stacked
            if let (Some(rh), Some(nm), Some(ds)) = (self.draft_rarity.get(i), self.draft_names.get(i), self.draft_descs.get(i)) {
                ui.send(*rh, WidgetMessage::DesiredPosition(Vector2::new(px, cards_y + 8.0 * s)));
                ui.send(*rh, WidgetMessage::Width(cw));
                ui.send(*rh, WidgetMessage::Height(30.0 * s));
                ui.send(*rh, WidgetMessage::Visibility(true));
                ui.send(*rh, WidgetMessage::ZIndex(3));
                ui.send(*nm, WidgetMessage::DesiredPosition(Vector2::new(px, cards_y + 44.0 * s)));
                ui.send(*nm, WidgetMessage::Width(cw));
                ui.send(*nm, WidgetMessage::Height(48.0 * s));
                ui.send(*nm, WidgetMessage::Visibility(true));
                ui.send(*nm, WidgetMessage::ZIndex(3));
                ui.send(*ds, WidgetMessage::DesiredPosition(Vector2::new(px, cards_y + 100.0 * s)));
                ui.send(*ds, WidgetMessage::Width(cw));
                ui.send(*ds, WidgetMessage::Height(120.0 * s));
                ui.send(*ds, WidgetMessage::Visibility(true));
                ui.send(*ds, WidgetMessage::ZIndex(3));
            }
            // milestone worms show their rank color; weapons theirs; rest talent
            let (rar_txt, rar_col, title, desc, has) = if let Some(card) = self.inner.draft_cards.get(i) {
                let (rt, rc) = match &card.kind {
                    DraftKind::Weapon(w) => (w.rarity.name().to_lowercase(), w.rarity.color()),
                    DraftKind::GuWorm(idx) => {
                        let r = crate::gu::GU.get(*idx).map(|d| d.rank).unwrap_or(1);
                        (format!("rank {r} worm"), crate::gu::rank_rarity(r).color())
                    }
                    _ => ("talent".to_string(), PATCH_CREAM),
                };
                (rt, rc, card.title.clone(), card.desc.clone(), true)
            } else {
                (String::new(), PATCH_CREAM, String::new(), String::new(), false)
            };
            if has {
                if let Some(rh) = self.draft_rarity.get(i) {
                    ui.send(*rh, TextMessage::Text(rar_txt));
                    ui.send(*rh, WidgetMessage::Foreground(Brush::Solid(col(rar_col)).into()));
                }
                if let Some(nm) = self.draft_names.get(i) {
                    let txt = if sel {
                        format!("> {} <", title)
                    } else {
                        title.clone()
                    };
                    ui.send(*nm, TextMessage::Text(txt));
                    let fg = if sel { PATCH_CREAM } else { PATCH_INK };
                    ui.send(*nm, WidgetMessage::Foreground(Brush::Solid(col(fg)).into()));
                }
                if let Some(ds) = self.draft_descs.get(i) {
                    ui.send(*ds, TextMessage::Text(desc));
                    ui.send(*ds, WidgetMessage::Foreground(Brush::Solid(col(PATCH_INK)).into()));
                }
            } else {
                for h in [self.draft_rarity.get(i), self.draft_names.get(i), self.draft_descs.get(i)]
                    .into_iter()
                    .flatten()
                {
                    ui.send(*h, TextMessage::Text(String::new()));
                }
            }
        }
        ui.send(
            self.draft_hint,
            WidgetMessage::DesiredPosition(Vector2::new(cx - 450.0 * s, cards_y + 240.0 * s + 20.0 * s)),
        );
        ui.send(self.draft_hint, WidgetMessage::Width(900.0 * s));
        ui.send(self.draft_hint, WidgetMessage::Height(36.0 * s));
        ui.send(self.draft_hint, WidgetMessage::Visibility(true));
        ui.send(self.draft_hint, WidgetMessage::ZIndex(3));
        ui.send(
            self.draft_hint,
            TextMessage::Text("1 / 2 / 3 pick · Left/Right select · ENTER confirm".to_string()),
        );
    }

    /// Pause menu: resume / restart / quit. Same patch language as the rest.
    pub(crate) fn sync_pause(&mut self, ctx: &mut PluginContext) {
        if !self.paused {
            if !self.pause_sig.is_empty() {
                let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
                    return;
                };
                ui.send(self.pause_title, WidgetMessage::Visibility(false));
                for h in self.pause_rows.iter() {
                    ui.send(*h, WidgetMessage::Visibility(false));
                }
                for bg in self.pause_row_bgs.iter().chain(self.pause_row_golds.iter()).chain(self.pause_darks.iter()) {
                    ui.send(*bg, WidgetMessage::Visibility(false));
                }
                ui.send(self.pause_hint, WidgetMessage::Visibility(false));
                self.pause_sig.clear();
            }
            return;
        }
        if self.pause_rows.len() < 5 || self.pause_title.is_none() {
            return;
        }
        let size = window_size(ctx);
        let (w, h) = (size.x, size.y);
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let sig = format!("{:.0}x{:.0}|{}", w, h, self.pause_sel);
        if sig == self.pause_sig {
            return;
        }
        self.pause_sig = sig;
        let s = ui_scale(ctx);
        let cx = w * 0.5;
        let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        ui.send(self.pause_title, WidgetMessage::DesiredPosition(Vector2::new(cx - 350.0 * s, h * 0.24)));
        ui.send(self.pause_title, WidgetMessage::Width(700.0 * s));
        ui.send(self.pause_title, WidgetMessage::Height(72.0 * s));
        ui.send(self.pause_title, WidgetMessage::Visibility(true));
        ui.send(self.pause_title, WidgetMessage::ZIndex(3));
        let rows_y = h * 0.32;
        let labels = ["RESUME".to_string(), "STORY".to_string(), "SETTINGS".to_string(), "RESTART RUN".to_string(), "QUIT TO MENU".to_string()];
        for (i, rh) in self.pause_rows.iter().enumerate() {
            let ry = rows_y + i as f32 * 84.0 * s;
            let sel = i == self.pause_sel;
            ui.send(*rh, WidgetMessage::DesiredPosition(Vector2::new(cx - 350.0 * s, ry)));
            ui.send(*rh, WidgetMessage::Width(700.0 * s));
            ui.send(*rh, WidgetMessage::Height(64.0 * s));
            ui.send(*rh, WidgetMessage::Visibility(true));
            ui.send(*rh, WidgetMessage::ZIndex(3));
            ui.send(*rh, TextMessage::Text(if sel {
                format!("> {} <", labels[i])
            } else {
                format!("  {}  ", labels[i])
            }));
            let fg = if sel { PATCH_CREAM } else { PATCH_INK };
            ui.send(*rh, WidgetMessage::Foreground(Brush::Solid(col(fg)).into()));
            if let Some(bg) = self.pause_row_bgs.get(i) {
                ui.send(*bg, WidgetMessage::DesiredPosition(Vector2::new(cx - 362.0 * s, ry - 12.0 * s)));
                ui.send(*bg, WidgetMessage::Width(724.0 * s));
                ui.send(*bg, WidgetMessage::Height(88.0 * s));
                ui.send(*bg, WidgetMessage::Visibility(true));
                ui.send(*bg, WidgetMessage::ZIndex(2));
            }
            if let Some(gold) = self.pause_row_golds.get(i) {
                ui.send(*gold, WidgetMessage::DesiredPosition(Vector2::new(cx - 362.0 * s, ry - 12.0 * s)));
                ui.send(*gold, WidgetMessage::Width(724.0 * s));
                ui.send(*gold, WidgetMessage::Height(88.0 * s));
                ui.send(*gold, WidgetMessage::Visibility(sel));
                ui.send(*gold, WidgetMessage::ZIndex(2));
            }
            // dark twin on every UNSELECTED row so the pick pops
            if let Some(dark) = self.pause_darks.get(i) {
                ui.send(*dark, WidgetMessage::DesiredPosition(Vector2::new(cx - 362.0 * s, ry - 12.0 * s)));
                ui.send(*dark, WidgetMessage::Width(724.0 * s));
                ui.send(*dark, WidgetMessage::Height(88.0 * s));
                ui.send(*dark, WidgetMessage::Visibility(!sel));
                ui.send(*dark, WidgetMessage::ZIndex(2));
            }
        }
        ui.send(self.pause_hint, WidgetMessage::DesiredPosition(Vector2::new(cx - 350.0 * s, rows_y + 5.0 * 84.0 * s + 16.0 * s)));
        ui.send(self.pause_hint, WidgetMessage::Width(700.0 * s));
        ui.send(self.pause_hint, WidgetMessage::Height(30.0 * s));
        ui.send(self.pause_hint, WidgetMessage::Visibility(true));
        ui.send(self.pause_hint, WidgetMessage::ZIndex(3));
    }

    /// Applies a settings toggle immediately (visibility follows at once).
    pub(crate) fn flip_setting(&mut self, ctx: &mut PluginContext, i: usize) {
        match i {
            0 => {
                self.log_on = !self.log_on;
                if let Some(ui) = ctx.user_interfaces.iter_mut().next() {
                    let vis = self.log_on && self.hud_ui_on;
                    ui.send(self.log_h, WidgetMessage::Visibility(vis));
                    ui.send(self.log_bg, WidgetMessage::Visibility(vis));
                    ui.send(self.log_dim, WidgetMessage::Visibility(false));
                }
            }
            1 => {
                self.floaters_on = !self.floaters_on;
                if let Some(ui) = ctx.user_interfaces.iter_mut().next() {
                    // hidden until the next sync repositions them
                    for h in self.float_labels.iter() {
                        ui.send(*h, WidgetMessage::Visibility(self.floaters_on && self.hud_ui_on));
                    }
                }
            }
            _ => {
                self.shake_on = !self.shake_on;
            }
        }
    }

    /// Settings overlay: LOG / FLOATERS / SHAKE toggles + BACK.
    /// Same patch language, text highlight only (no gold twins here).
    pub(crate) fn sync_settings(&mut self, ctx: &mut PluginContext) {
        if !self.settings_open {
            if !self.settings_sig.is_empty() {
                let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
                    return;
                };
                ui.send(self.settings_title, WidgetMessage::Visibility(false));
                for h in self.settings_rows.iter() {
                    ui.send(*h, WidgetMessage::Visibility(false));
                }
                for bg in self.settings_row_bgs.iter() {
                    ui.send(*bg, WidgetMessage::Visibility(false));
                }
                ui.send(self.settings_hint, WidgetMessage::Visibility(false));
                self.settings_sig.clear();
            }
            return;
        }
        if self.settings_rows.len() < 4 || self.settings_title.is_none() {
            return;
        }
        let size = window_size(ctx);
        let (w, h) = (size.x, size.y);
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let sig = format!(
            "{:.0}x{:.0}|{}|{}|{}|{}",
            w, h, self.settings_sel, self.log_on as u8, self.floaters_on as u8, self.shake_on as u8
        );
        if sig == self.settings_sig {
            return;
        }
        self.settings_sig = sig;
        // pause widgets hide behind the overlay; pause_sig is cleared by the
        // branch so they repaint on return
        let s = ui_scale(ctx);
        let cx = w * 0.5;
        let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        ui.send(self.pause_title, WidgetMessage::Visibility(false));
        for h in self.pause_rows.iter() {
            ui.send(*h, WidgetMessage::Visibility(false));
        }
        for bg in self.pause_row_bgs.iter().chain(self.pause_row_golds.iter()).chain(self.pause_darks.iter()) {
            ui.send(*bg, WidgetMessage::Visibility(false));
        }
        ui.send(self.pause_hint, WidgetMessage::Visibility(false));
        ui.send(self.settings_title, WidgetMessage::DesiredPosition(Vector2::new(cx - 350.0 * s, h * 0.24)));
        ui.send(self.settings_title, WidgetMessage::Width(700.0 * s));
        ui.send(self.settings_title, WidgetMessage::Height(72.0 * s));
        ui.send(self.settings_title, WidgetMessage::Visibility(true));
        ui.send(self.settings_title, WidgetMessage::ZIndex(3));
        let rows_y = h * 0.38;
        let onoff = |b: bool| if b { "ON" } else { "OFF" };
        let labels = [
            format!("LOG: {}", onoff(self.log_on)),
            format!("FLOATERS: {}", onoff(self.floaters_on)),
            format!("SHAKE: {}", onoff(self.shake_on)),
            "BACK".to_string(),
        ];
        for (i, rh) in self.settings_rows.iter().enumerate() {
            let ry = rows_y + i as f32 * 100.0 * s;
            let sel = i == self.settings_sel;
            ui.send(*rh, WidgetMessage::DesiredPosition(Vector2::new(cx - 350.0 * s, ry)));
            ui.send(*rh, WidgetMessage::Width(700.0 * s));
            ui.send(*rh, WidgetMessage::Height(64.0 * s));
            ui.send(*rh, WidgetMessage::Visibility(true));
            ui.send(*rh, WidgetMessage::ZIndex(3));
            ui.send(*rh, TextMessage::Text(if sel {
                format!("> {} <", labels[i])
            } else {
                format!("  {}  ", labels[i])
            }));
            let fg = if sel { PATCH_CREAM } else { PATCH_INK };
            ui.send(*rh, WidgetMessage::Foreground(Brush::Solid(col(fg)).into()));
            if let Some(bg) = self.settings_row_bgs.get(i) {
                ui.send(*bg, WidgetMessage::DesiredPosition(Vector2::new(cx - 362.0 * s, ry - 12.0 * s)));
                ui.send(*bg, WidgetMessage::Width(724.0 * s));
                ui.send(*bg, WidgetMessage::Height(88.0 * s));
                ui.send(*bg, WidgetMessage::Visibility(true));
                ui.send(*bg, WidgetMessage::ZIndex(2));
            }
        }
        ui.send(self.settings_hint, WidgetMessage::DesiredPosition(Vector2::new(cx - 350.0 * s, rows_y + 4.0 * 100.0 * s + 16.0 * s)));
        ui.send(self.settings_hint, WidgetMessage::Width(700.0 * s));
        ui.send(self.settings_hint, WidgetMessage::Height(30.0 * s));
        ui.send(self.settings_hint, WidgetMessage::Visibility(true));
        ui.send(self.settings_hint, WidgetMessage::ZIndex(3));
    }

    /// Shared story-hub layout helper (see free function below).
    /// Path select: twelve Gu paths in one column, desc of the hovered path
    /// in the footer line.
    pub(crate) fn sync_path(&mut self, ctx: &mut PluginContext) {
        if !self.path_open {
            if !self.path_sig.is_empty() {
                let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
                    return;
                };
                ui.send(self.path_title, WidgetMessage::Visibility(false));
                for h in self.path_rows.iter() {
                    ui.send(*h, WidgetMessage::Visibility(false));
                }
                ui.send(self.path_hint, WidgetMessage::Visibility(false));
                self.path_sig.clear();
            }
            return;
        }
        if self.path_rows.len() < 12 || self.path_title.is_none() {
            return;
        }
        let size = window_size(ctx);
        let (w, h) = (size.x, size.y);
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let sig = format!("{:.0}x{:.0}|{}", w, h, self.path_sel);
        if sig == self.path_sig {
            return;
        }
        self.path_sig = sig;
        let s = ui_scale(ctx);
        let cx = w * 0.5;
        let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        ui.send(self.path_title, WidgetMessage::DesiredPosition(Vector2::new(cx - 350.0 * s, h * 0.06)));
        ui.send(self.path_title, WidgetMessage::Width(700.0 * s));
        ui.send(self.path_title, WidgetMessage::Height(64.0 * s));
        ui.send(self.path_title, WidgetMessage::Visibility(true));
        ui.send(self.path_title, WidgetMessage::ZIndex(3));
        let rows_y = h * 0.18;
        for (i, rh) in self.path_rows.iter().enumerate() {
            let p = &crate::gu::PATHS[i];
            let sel = i == self.path_sel;
            ui.send(*rh, WidgetMessage::DesiredPosition(Vector2::new(cx - 320.0 * s, rows_y + i as f32 * 40.0 * s)));
            ui.send(*rh, WidgetMessage::Width(640.0 * s));
            ui.send(*rh, WidgetMessage::Height(36.0 * s));
            ui.send(*rh, WidgetMessage::Visibility(true));
            ui.send(*rh, WidgetMessage::ZIndex(3));
            ui.send(*rh, TextMessage::Text(if sel {
                format!("> {}. {} <", i + 1, p.name)
            } else {
                format!("{}. {}", i + 1, p.name)
            }));
            let fg = if sel { PATCH_CREAM } else { PATCH_INK };
            ui.send(*rh, WidgetMessage::Foreground(Brush::Solid(col(fg)).into()));
        }
        let desc = crate::gu::PATHS[self.path_sel.min(11)].desc;
        ui.send(self.path_hint, WidgetMessage::DesiredPosition(Vector2::new(cx - 350.0 * s, rows_y + 12.0 * 40.0 * s + 8.0 * s)));
        ui.send(self.path_hint, WidgetMessage::Width(700.0 * s));
        ui.send(self.path_hint, WidgetMessage::Height(60.0 * s));
        ui.send(self.path_hint, WidgetMessage::Visibility(true));
        ui.send(self.path_hint, WidgetMessage::ZIndex(3));
        ui.send(self.path_hint, TextMessage::Wrap(WrapMode::Word));
        ui.send(self.path_hint, TextMessage::Text(format!("{} — 1-9 / Up/Down · ENTER to kowtow", desc)));
    }

    /// Story hub: chronicle rail, narrative, three option columns, logs.
    /// Hidden unless `story_open` and no draft/class overlay owns the screen.
    pub(crate) fn sync_story(&mut self, ctx: &mut PluginContext) {
        let show = self.story_open && !self.inner.draft_open;
        if !show {
            if !self.story_sig.is_empty() {
                let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
                    return;
                };
                ui.send(self.story_title, WidgetMessage::Visibility(false));
                ui.send(self.story_body, WidgetMessage::Visibility(false));
                for h in self.story_subs.iter() {
                    ui.send(*h, WidgetMessage::Visibility(false));
                }
                for h in self.story_rows.iter() {
                    ui.send(*h, WidgetMessage::Visibility(false));
                }
                ui.send(self.story_left_title, WidgetMessage::Visibility(false));
                ui.send(self.story_left_body, WidgetMessage::Visibility(false));
                ui.send(self.story_log, WidgetMessage::Visibility(false));
                ui.send(self.story_status, WidgetMessage::Visibility(false));
                ui.send(self.story_reinc, WidgetMessage::Visibility(false));
                ui.send(self.story_left_bg, WidgetMessage::Visibility(false));
                ui.send(self.story_body_bg, WidgetMessage::Visibility(false));
                for bg in self.story_cols_bg.iter() {
                    ui.send(*bg, WidgetMessage::Visibility(false));
                }
        ui.send(self.story_backdrop, WidgetMessage::Visibility(false));
        ui.send(self.story_sleep_bg, WidgetMessage::Visibility(false));
        ui.send(self.story_sleep_fg, WidgetMessage::Visibility(false));
        for sp in self.story_seps.iter() {
                    ui.send(*sp, WidgetMessage::Visibility(false));
                }
                ui.send(self.story_backdrop, WidgetMessage::Visibility(false));
                ui.send(self.story_sleep_bg, WidgetMessage::Visibility(false));
                ui.send(self.story_sleep_fg, WidgetMessage::Visibility(false));
                for sh in self.story_stars.iter() {
                    ui.send(*sh, WidgetMessage::Visibility(false));
                }
                ui.send(self.story_log_bg, WidgetMessage::Visibility(false));
                ui.send(self.story_status_bg, WidgetMessage::Visibility(false));
                self.story_sig.clear();
            }
            return;
        }
        if self.story_rows.len() < 15 || self.story_title.is_none() {
            return;
        }
        let size = window_size(ctx);
        let (w, h) = (size.x, size.y);
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let p = &self.inner.player;
        let st = &self.inner.story;
        let mut best_rank: u8 = 0;
        for &gi in p.inventory.iter() {
            if let Some(g) = GU.get(gi) {
                best_rank = best_rank.max(g.rank);
            }
        }
        let sect_view = st.sect.as_deref();
        let nodes = crate::story::story_nodes();
        let Some(node) = crate::story::find_node(&nodes, &st.node) else {
            return;
        };
        let items = crate::story::visible_choices(
            node, 0, p.level, self.inner.path, sect_view, &st.flags, &st.counters,
            st.stamina, p.gold, p.kills, best_rank,
        );
        let mut counts = [0usize; 3];
        let flat = crate::story::flat_choices(&items);
        for &fi in flat.iter() {
            counts[items[fi].col] += 1;
        }
        let sig = format!(
            "{:.0}x{:.0}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            w, h, st.node, self.story_sel, st.stamina as u32, p.gold, p.level,
            p.kills, st.contrib(), best_rank,
            st.flags.len(), self.inner.path.unwrap_or(99), st.sleeping as u8,
            p.xp, p.inventory.len(),
        );
        if sig == self.story_sig {
            return;
        }
        self.story_sig = sig;
        let s = ui_scale(ctx);
        let lay = story_layout(w, h, s, counts);
        let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        let place = |ui: &mut UserInterface, h: Handle<Text>, r: (f32, f32, f32, f32), z: usize| {
            ui.send(h, WidgetMessage::DesiredPosition(Vector2::new(r.0, r.1)));
            ui.send(h, WidgetMessage::Width(r.2));
            ui.send(h, WidgetMessage::Height(r.3));
            ui.send(h, WidgetMessage::Visibility(true));
            ui.send(h, WidgetMessage::ZIndex(z));
        };
        let place_bg = |ui: &mut UserInterface, h: Handle<Border>, r: (f32, f32, f32, f32)| {
            ui.send(h, WidgetMessage::DesiredPosition(Vector2::new(r.0, r.1)));
            ui.send(h, WidgetMessage::Width(r.2));
            ui.send(h, WidgetMessage::Height(r.3));
            ui.send(h, WidgetMessage::Visibility(true));
            ui.send(h, WidgetMessage::ZIndex(2));
        };
        let place_sep = |ui: &mut UserInterface, h: Handle<Border>, r: (f32, f32, f32, f32)| {
            ui.send(h, WidgetMessage::DesiredPosition(Vector2::new(r.0, r.1)));
            ui.send(h, WidgetMessage::Width(r.2));
            ui.send(h, WidgetMessage::Height(r.3.max(2.0)));
            ui.send(h, WidgetMessage::Visibility(true));
            ui.send(h, WidgetMessage::ZIndex(3));
        };
        if self.story_seps.len() < 2 {
            return;
        }
        // full-screen black first (z0), then stars (z1), then panels (z2)
        ui.send(self.story_backdrop, WidgetMessage::DesiredPosition(Vector2::new(lay.back_r.0, lay.back_r.1)));
        ui.send(self.story_backdrop, WidgetMessage::Width(lay.back_r.2));
        ui.send(self.story_backdrop, WidgetMessage::Height(lay.back_r.3));
        ui.send(self.story_backdrop, WidgetMessage::Visibility(true));
        ui.send(self.story_backdrop, WidgetMessage::ZIndex(0));
        place_sep(&mut *ui, self.story_seps[0], lay.sep_title_r);
        place_sep(&mut *ui, self.story_seps[1], lay.sep_mid_r);
        // starfield behind everything (deterministic scatter, screen space)
        for (i, sh) in self.story_stars.iter().enumerate() {
            let fi = i as f32 + 0.5;
            let sx = (fi * 0.6180339).fract() * w;
            let sy = (fi * 0.381966).fract() * h;
            ui.send(*sh, WidgetMessage::DesiredPosition(Vector2::new(sx, sy)));
            ui.send(*sh, WidgetMessage::Visibility(true));
            ui.send(*sh, WidgetMessage::ZIndex(1));
        }
        place_bg(&mut *ui, self.story_left_bg, lay.left_r);
        ui.send(self.story_left_title, TextMessage::Text("CHRONICLE".to_string()));
        ui.send(self.story_left_title, WidgetMessage::Foreground(Brush::Solid(col(DARK_PAPER)).into()));
        place(&mut *ui, self.story_left_title, (lay.left_r.0 + 12.0 * s, lay.left_r.1 + 8.0 * s, lay.left_r.2 - 24.0 * s, 32.0 * s), 3);
        ui.send(
            self.story_left_body,
            TextMessage::Text(format!(
                "Spirit Stones: {}\nSTAMINA: {:.0}/{}{}\nCurrently: {} · Lv {}",
                p.gold,
                st.stamina,
                st.max_stamina,
                if st.sleeping { " (asleep…)" } else { "" },
                st.sect.as_deref().unwrap_or("Rogue"),
                p.level
            )),
        );
        place(&mut *ui, self.story_left_body, (lay.left_r.0 + 12.0 * s, lay.left_r.1 + 46.0 * s, lay.left_r.2 - 24.0 * s, lay.left_r.3 - 58.0 * s), 3);
        ui.send(self.story_left_body, WidgetMessage::Foreground(Brush::Solid(col(DARK_BODY)).into()));
        // sleep bar under the chronicle: trough + stamina fill
        {
            let (bx, by, bw, bh) = lay.sleep_bg_r;
            let frac = (st.stamina / st.max_stamina.max(1) as f32).clamp(0.0, 1.0);
            ui.send(self.story_sleep_bg, WidgetMessage::DesiredPosition(Vector2::new(bx, by)));
            ui.send(self.story_sleep_bg, WidgetMessage::Width(bw));
            ui.send(self.story_sleep_bg, WidgetMessage::Height(bh));
            ui.send(self.story_sleep_bg, WidgetMessage::Visibility(true));
            ui.send(self.story_sleep_bg, WidgetMessage::ZIndex(2));
            let iw = ((bw - 6.0 * s) * frac).max(if frac > 0.0 { 3.0 } else { 0.0 });
            ui.send(self.story_sleep_fg, WidgetMessage::DesiredPosition(Vector2::new(bx + 3.0 * s, by + 3.0 * s)));
            ui.send(self.story_sleep_fg, WidgetMessage::Width(iw));
            ui.send(self.story_sleep_fg, WidgetMessage::Height((bh - 6.0 * s).max(2.0)));
            ui.send(self.story_sleep_fg, WidgetMessage::Visibility(true));
            ui.send(self.story_sleep_fg, WidgetMessage::ZIndex(3));
        }
        ui.send(self.story_title, TextMessage::Text(node.title.to_string()));
        ui.send(self.story_title, WidgetMessage::Foreground(Brush::Solid(col(DARK_PAPER)).into()));
        place(&mut *ui, self.story_title, lay.title_r, 3);
        place_bg(&mut *ui, self.story_body_bg, lay.body_bg_r);
        ui.send(self.story_body, TextMessage::Text(node.body.to_string()));
        ui.send(self.story_body, WidgetMessage::Foreground(Brush::Solid(col(DARK_BODY)).into()));
        place(&mut *ui, self.story_body, (lay.body_r.0, lay.body_r.1, lay.body_r.2, lay.body_r.3), 3);
        let sub_names = ["Special Options", "Repeatable Options", "Incomplete Options"];
        for c in 0..3 {
            ui.send(self.story_subs[c], TextMessage::Text(sub_names[c].to_string()));
            ui.send(self.story_subs[c], WidgetMessage::Foreground(Brush::Solid(col(DARK_PAPER)).into()));
            place(&mut *ui, self.story_subs[c], lay.sub_rs[c], 3);
            place_bg(&mut *ui, self.story_cols_bg[c], lay.col_rs[c]);
        }
        // rows follow the shared flat order 1:1 with layout row rects
        for (ri, h) in self.story_rows.iter().enumerate() {
            if let Some(&fi) = flat.get(ri) {
                let r = lay.row_rs[ri];
                let it = &items[fi];
                ui.send(*h, WidgetMessage::DesiredPosition(Vector2::new(r.0, r.1)));
                ui.send(*h, WidgetMessage::Width(r.2));
                ui.send(*h, WidgetMessage::Height(r.3));
                ui.send(*h, WidgetMessage::Visibility(true));
                ui.send(*h, WidgetMessage::ZIndex(3));
                let sel = ri == self.story_sel;
                let mut txt = it.label.clone();
                if !it.note.is_empty() {
                    txt.push_str(&format!(" ({})", it.note));
                }
                if sel && it.col != 2 {
                    txt = format!("> {}", txt);
                }
                ui.send(*h, TextMessage::Text(txt));
                let fg = if it.col == 2 {
                    DARK_DIM
                } else if sel {
                    DARK_GOLD
                } else {
                    DARK_BODY
                };
                ui.send(*h, WidgetMessage::Foreground(Brush::Solid(col(fg)).into()));
            } else {
                ui.send(*h, WidgetMessage::Visibility(false));
                ui.send(*h, TextMessage::Text(String::new()));
            }
        }
        place_bg(&mut *ui, self.story_log_bg, lay.log_r);
        let tail: Vec<&str> = st.log.iter().rev().take(8).map(|s| s.as_str()).collect();
        let log_txt = tail.into_iter().rev().collect::<Vec<_>>().join("\n");
        ui.send(self.story_log, TextMessage::Text(log_txt));
        ui.send(self.story_log, WidgetMessage::Foreground(Brush::Solid(col(DARK_BODY)).into()));
        place(&mut *ui, self.story_log, (lay.log_r.0 + 12.0 * s, lay.log_r.1 + 8.0 * s, lay.log_r.2 - 24.0 * s, lay.log_r.3 - 16.0 * s), 3);
        let path_name = self.inner.path.map(|p| crate::gu::PATHS[p].name).unwrap_or("?");
        let syn = crate::gu::synergy_summary(&p.inventory);
        let sets = self.inner.seen_syn.len();
        ui.send(
            self.story_status,
            TextMessage::Text(format!(
                "PATH: {}\nSECT: {} · Lv {} (XP {}/{})\nGU: {} (best rank {}) · ESS {:.0}\nSYNERGY: {} · {} sets\nENDINGS: {}",
                path_name,
                st.sect.as_deref().unwrap_or("Rogue"),
                p.level,
                p.xp,
                p.xp_needed(),
                p.inventory.len(),
                best_rank,
                p.essence,
                syn,
                sets,
                p.endings
            )),
        );
        place(&mut *ui, self.story_status, (lay.status_r.0 + 12.0 * s, lay.status_r.1 + 8.0 * s, lay.status_r.2 - 24.0 * s, lay.status_r.3 - 16.0 * s), 3);
        ui.send(self.story_status, WidgetMessage::Foreground(Brush::Solid(col(DARK_BODY)).into()));
        place_bg(&mut *ui, self.story_status_bg, lay.status_r);
        ui.send(self.story_reinc, TextMessage::Text("REINCARNATE".to_string()));
        ui.send(self.story_reinc, WidgetMessage::Foreground(Brush::Solid(col(DARK_RED)).into()));
        place(&mut *ui, self.story_reinc, lay.reinc_r, 3);
    }

    /// Stats menu: 5 rows with [-]/[+] affordances, Tab/Esc to close.
    /// Authoritative on visibility like the draft overlay.
    pub(crate) fn sync_stats(&mut self, ctx: &mut PluginContext) {
        if !self.stats_open {
            if !self.stats_sig.is_empty() {
                let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
                    return;
                };
                ui.send(self.stats_panel, WidgetMessage::Visibility(false));
                ui.send(self.stats_title, WidgetMessage::Visibility(false));
                for h in self.stats_rows.iter() {
                    ui.send(*h, WidgetMessage::Visibility(false));
                }
                ui.send(self.stats_hint, WidgetMessage::Visibility(false));
                self.stats_sig.clear();
            }
            return;
        }
        if self.stats_rows.len() < 5 || self.stats_panel.is_none() {
            return;
        }
        let size = window_size(ctx);
        let (w, h) = (size.x, size.y);
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let a = &self.inner.player.base_attrs;
        let sig = format!(
            "{:.0}x{:.0}|{}|{}|{},{},{},{},{}",
            w, h, self.stats_sel, self.inner.player.unspent_points,
            a.strength, a.agility, a.intellect, a.vitality, a.luck
        );
        if sig == self.stats_sig {
            return;
        }
        self.stats_sig = sig;
        let s = ui_scale(ctx);
        let cx = w * 0.5;
        let py = h * 0.25;
        let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        ui.send(self.stats_panel, WidgetMessage::DesiredPosition(Vector2::new(cx - 284.0 * s, py)));
        ui.send(self.stats_panel, WidgetMessage::Width(568.0 * s));
        ui.send(self.stats_panel, WidgetMessage::Height(384.0 * s));
        ui.send(self.stats_panel, WidgetMessage::Visibility(true));
        ui.send(self.stats_panel, WidgetMessage::ZIndex(0));
        ui.send(self.stats_title, WidgetMessage::DesiredPosition(Vector2::new(cx - 260.0 * s, py + 10.0 * s)));
        ui.send(self.stats_title, WidgetMessage::Width(520.0 * s));
        ui.send(self.stats_title, WidgetMessage::Height(44.0 * s));
        ui.send(self.stats_title, WidgetMessage::Visibility(true));
        ui.send(self.stats_title, WidgetMessage::ZIndex(1));
        ui.send(
            self.stats_title,
            TextMessage::Text(format!("STAT POINTS: {}", self.inner.player.unspent_points)),
        );
        let order = AttrKind::all();
        for (i, rh) in self.stats_rows.iter().enumerate() {
            let kind = order[i];
            let val = match kind {
                AttrKind::Strength => a.strength,
                AttrKind::Agility => a.agility,
                AttrKind::Intellect => a.intellect,
                AttrKind::Vitality => a.vitality,
                AttrKind::Luck => a.luck,
            };
            let sel = i == self.stats_sel;
            ui.send(*rh, WidgetMessage::DesiredPosition(Vector2::new(cx - 260.0 * s, py + (64.0 + i as f32 * 44.0) * s)));
            ui.send(*rh, WidgetMessage::Width(520.0 * s));
            ui.send(*rh, WidgetMessage::Height(40.0 * s));
            ui.send(*rh, WidgetMessage::Visibility(true));
            ui.send(*rh, WidgetMessage::ZIndex(1));
            let txt = if sel {
                format!("> {} {:<3} [-] [+] <", kind.name(), val)
            } else {
                format!("  {} {:<3} [-] [+]  ", kind.name(), val)
            };
            ui.send(*rh, TextMessage::Text(txt));
            let fg = if sel { PATCH_CREAM } else { PATCH_INK };
            ui.send(*rh, WidgetMessage::Foreground(Brush::Solid(col(fg)).into()));
        }
        ui.send(self.stats_hint, WidgetMessage::DesiredPosition(Vector2::new(cx - 260.0 * s, py + (64.0 + 5.0 * 44.0 + 6.0) * s)));
        ui.send(self.stats_hint, WidgetMessage::Width(520.0 * s));
        ui.send(self.stats_hint, WidgetMessage::Height(30.0 * s));
        ui.send(self.stats_hint, WidgetMessage::Visibility(true));
        ui.send(self.stats_hint, WidgetMessage::ZIndex(1));
    }

    /// Window-relative UI scaling. Fonts for every text widget plus all
    /// fixed left-anchored geometry follow the scale bucket (1.0 at 1280x720).
    /// Anchored/centered layouts multiply their own literals by `ui_scale`.
    /// Sends only when the bucket changes — cheap every tick otherwise.
    pub(crate) fn sync_ui_scale(&mut self, ctx: &mut PluginContext) {
        let bucket = (ui_scale(ctx) * 10.0).round() / 10.0;
        if bucket == self.font_scale {
            return;
        }
        self.font_scale = bucket;
        let fs = |base: f32| base * bucket;
        let Some(ui) = ctx.user_interfaces.iter_mut().next() else {
            return;
        };
        // fonts
        ui.send(self.hud_line, TextMessage::FontSize(fs(40.0).into()));
        ui.send(self.left_stats_text, TextMessage::FontSize(fs(30.0).into()));
        ui.send(self.items_text, TextMessage::FontSize(fs(18.0).into()));
        ui.send(self.log_h, TextMessage::FontSize(fs(30.0).into()));
        ui.send(self.banner_h, TextMessage::FontSize(fs(64.0).into()));
        for h in self.float_labels.iter() {
            ui.send(*h, TextMessage::FontSize(fs(40.0).into()));
        }
        ui.send(self.menu_title, TextMessage::FontSize(fs(64.0).into()));
        ui.send(self.menu_sub, TextMessage::FontSize(fs(28.0).into()));
        for h in self.menu_rows.iter() {
            ui.send(*h, TextMessage::FontSize(fs(44.0).into()));
        }
        ui.send(self.menu_help, TextMessage::FontSize(fs(26.0).into()));
        ui.send(self.menu_footer, TextMessage::FontSize(fs(26.0).into()));
        ui.send(self.draft_title, TextMessage::FontSize(fs(54.0).into()));
        for h in self.draft_rarity.iter() {
            ui.send(*h, TextMessage::FontSize(fs(22.0).into()));
        }
        for h in self.draft_names.iter() {
            ui.send(*h, TextMessage::FontSize(fs(32.0).into()));
        }
        for h in self.draft_descs.iter() {
            ui.send(*h, TextMessage::FontSize(fs(20.0).into()));
        }
        ui.send(self.draft_hint, TextMessage::FontSize(fs(26.0).into()));
        ui.send(self.sk_title, TextMessage::FontSize(fs(26.0).into()));
        for h in self.sk_names.iter() {
            ui.send(*h, TextMessage::FontSize(fs(20.0).into()));
        }
        for h in self.sk_keys.iter() {
            ui.send(*h, TextMessage::FontSize(fs(18.0).into()));
        }
        ui.send(self.stats_title, TextMessage::FontSize(fs(34.0).into()));
        for h in self.stats_rows.iter() {
            ui.send(*h, TextMessage::FontSize(fs(28.0).into()));
        }
        ui.send(self.stats_hint, TextMessage::FontSize(fs(20.0).into()));
        ui.send(self.pause_title, TextMessage::FontSize(fs(60.0).into()));
        for h in self.pause_rows.iter() {
            ui.send(*h, TextMessage::FontSize(fs(40.0).into()));
        }
        ui.send(self.pause_hint, TextMessage::FontSize(fs(22.0).into()));
        ui.send(self.path_title, TextMessage::FontSize(fs(48.0).into()));
        for h in self.path_rows.iter() {
            ui.send(*h, TextMessage::FontSize(fs(20.0).into()));
        }
        ui.send(self.path_hint, TextMessage::FontSize(fs(20.0).into()));
        ui.send(self.story_title, TextMessage::FontSize(fs(34.0).into()));
        ui.send(self.story_body, TextMessage::FontSize(fs(19.0).into()));
        for h in self.story_subs.iter() {
            ui.send(*h, TextMessage::FontSize(fs(20.0).into()));
        }
        for h in self.story_rows.iter() {
            ui.send(*h, TextMessage::FontSize(fs(18.0).into()));
        }
        ui.send(self.story_left_title, TextMessage::FontSize(fs(26.0).into()));
        ui.send(self.story_left_body, TextMessage::FontSize(fs(20.0).into()));
        ui.send(self.story_log, TextMessage::FontSize(fs(16.0).into()));
        ui.send(self.story_status, TextMessage::FontSize(fs(18.0).into()));
        ui.send(self.story_reinc, TextMessage::FontSize(fs(20.0).into()));
        ui.send(self.settings_title, TextMessage::FontSize(fs(60.0).into()));
        for h in self.settings_rows.iter() {
            ui.send(*h, TextMessage::FontSize(fs(40.0).into()));
        }
        ui.send(self.settings_hint, TextMessage::FontSize(fs(22.0).into()));
        // fixed left-anchored geometry (hearts / level / stats / skills)
        for (i, h) in self.hearts.iter().enumerate() {
            ui.send(*h, WidgetMessage::DesiredPosition(Vector2::new((16.0 + i as f32 * 44.0) * bucket, 40.0 * bucket)));
            ui.send(*h, WidgetMessage::Width(36.0 * bucket));
            ui.send(*h, WidgetMessage::Height(30.0 * bucket));
        }
        // dark heart twins ride the same slots (shown only when dimmed)
        for (i, h) in self.heart_darks.iter().enumerate() {
            ui.send(*h, WidgetMessage::DesiredPosition(Vector2::new((16.0 + i as f32 * 44.0) * bucket, 40.0 * bucket)));
            ui.send(*h, WidgetMessage::Width(36.0 * bucket));
            ui.send(*h, WidgetMessage::Height(30.0 * bucket));
        }
        ui.send(self.hud_line, WidgetMessage::DesiredPosition(Vector2::new(16.0 * bucket, 76.0 * bucket)));
        ui.send(self.hud_line, WidgetMessage::Width(500.0 * bucket));
        ui.send(self.hud_line, WidgetMessage::Height(48.0 * bucket));
        ui.send(self.left_stats_panel, WidgetMessage::DesiredPosition(Vector2::new(16.0 * bucket, 130.0 * bucket)));
        ui.send(self.left_stats_panel, WidgetMessage::Width(156.0 * bucket));
        ui.send(self.left_stats_panel, WidgetMessage::Height(260.0 * bucket));
        ui.send(self.stats_dim, WidgetMessage::DesiredPosition(Vector2::new(16.0 * bucket, 130.0 * bucket)));
        ui.send(self.stats_dim, WidgetMessage::Width(156.0 * bucket));
        ui.send(self.stats_dim, WidgetMessage::Height(260.0 * bucket));
        ui.send(self.left_stats_text, WidgetMessage::DesiredPosition(Vector2::new(28.0 * bucket, 142.0 * bucket)));
        ui.send(self.left_stats_text, WidgetMessage::Width(132.0 * bucket));
        ui.send(self.left_stats_text, WidgetMessage::Height(236.0 * bucket));
        ui.send(self.sk_panel, WidgetMessage::DesiredPosition(Vector2::new(16.0 * bucket, 400.0 * bucket)));
        ui.send(self.sk_panel, WidgetMessage::Width(248.0 * bucket));
        ui.send(self.sk_panel, WidgetMessage::Height(312.0 * bucket));
        ui.send(self.skills_dim, WidgetMessage::DesiredPosition(Vector2::new(16.0 * bucket, 400.0 * bucket)));
        ui.send(self.skills_dim, WidgetMessage::Width(248.0 * bucket));
        ui.send(self.skills_dim, WidgetMessage::Height(312.0 * bucket));
        ui.send(self.sk_title, WidgetMessage::DesiredPosition(Vector2::new(28.0 * bucket, 410.0 * bucket)));
        ui.send(self.sk_title, WidgetMessage::Width(224.0 * bucket));
        ui.send(self.sk_title, WidgetMessage::Height(44.0 * bucket));
        for (i, nm) in self.sk_names.iter().enumerate() {
            ui.send(*nm, WidgetMessage::DesiredPosition(Vector2::new(28.0 * bucket, (462.0 + i as f32 * 62.0) * bucket)));
            ui.send(*nm, WidgetMessage::Width(140.0 * bucket));
            ui.send(*nm, WidgetMessage::Height(56.0 * bucket));
        }
        for (i, kb) in self.sk_keys.iter().enumerate() {
            ui.send(*kb, WidgetMessage::DesiredPosition(Vector2::new(172.0 * bucket, (462.0 + i as f32 * 62.0) * bucket)));
            ui.send(*kb, WidgetMessage::Width(76.0 * bucket));
            ui.send(*kb, WidgetMessage::Height(56.0 * bucket));
        }
    }

    pub(crate)     fn update_hud(&mut self, ctx: &mut PluginContext) {
        if self.hearts.is_empty() {
            return;
        }
        // Whole play UI lives in the game only (menu hides it); the draft
        // keeps it visible but dimmed under the cards.
        let in_game = self.inner.started;
        if in_game != self.hud_ui_on {
            if let Some(ui) = ctx.user_interfaces.iter_mut().next() {
                self.set_play_ui_visible_ui(ui, in_game);
            }
            self.hud_ui_on = in_game;
        }
        // window-relative scale for fonts + fixed geometry (bucket-gated)
        self.sync_ui_scale(ctx);
        let s = ui_scale(ctx);
        // HUD dim: pause + upgrade drafts dim every panel via the lighter dark
        // twins; map cells repaint darker. Transition-only, no per-tick churn.
        // The log twin stays dark only while the log itself is enabled.
        let dimmed = self.paused || self.inner.draft_open;
        if dimmed != self.hud_dim_on {
            if let Some(ui) = ctx.user_interfaces.iter_mut().next() {
                for twin in [self.stats_dim, self.skills_dim, self.map_dim, self.items_dim] {
                    ui.send(twin, WidgetMessage::Visibility(dimmed));
                }
                ui.send(self.log_dim, WidgetMessage::Visibility(dimmed && self.log_on));
                for h in self.heart_darks.iter() {
                    ui.send(*h, WidgetMessage::Visibility(dimmed));
                }
            }
            // force the minimap to repaint in the new palette below
            self.map_cache = vec![(9, 9, 9); MAP_W * MAP_H];
            self.hud_dim_on = dimmed;
        }
        // XP bar pinned to the very top of the screen (widths follow window + progress)
        {
            let (xp_now, need_now, maxed) = {
                let pl = &self.inner.player;
                (pl.xp, pl.xp_needed(), pl.level >= MAX_LEVEL)
            };
            let ws = window_size(ctx);
            let frac = if maxed {
                1.0
            } else {
                (xp_now as f32 / need_now.max(1) as f32).clamp(0.0, 1.0)
            };
            if let Some(ui) = ctx.user_interfaces.iter_mut().next() {
                ui.send(self.xp_bar_bg, WidgetMessage::Width(ws.x));
                ui.send(self.xp_bar_bg, WidgetMessage::Height(26.0 * s));
                ui.send(self.xp_bar_fg, WidgetMessage::Width((ws.x * frac).max(2.0)));
                ui.send(self.xp_bar_fg, WidgetMessage::Height(18.0 * s));
                ui.send(self.xp_bar_fg, WidgetMessage::DesiredPosition(Vector2::new(4.0 * s, 4.0 * s)));
            }
            let want_on = self.inner.started;
            if want_on != self.xpbar_on {
                if let Some(ui) = ctx.user_interfaces.iter_mut().next() {
                    ui.send(self.xp_bar_bg, WidgetMessage::Visibility(want_on));
                    ui.send(self.xp_bar_fg, WidgetMessage::Visibility(want_on));
                }
                self.xpbar_on = want_on;
            }
        }
        // level-up banner, recentered on the current window
        if !self.banner_h.is_none() && self.banner_timer > 0.0 && !self.banner_shown {
            let ws = window_size(ctx);
            let lv = self.inner.player.level;
            let pts = self.inner.player.unspent_points;
            if let Some(ui) = ctx.user_interfaces.iter_mut().next() {
                ui.send(self.banner_h, WidgetMessage::DesiredPosition(Vector2::new((ws.x - 1000.0 * s).max(0.0) * 0.5, ws.y * 0.3)));
                ui.send(self.banner_h, WidgetMessage::Width(1000.0 * s));
                ui.send(self.banner_h, WidgetMessage::Height(96.0 * s));
                ui.send(
                    self.banner_h,
                        TextMessage::Text(format!(
                            "* LEVEL {}! +{} pts *",
                            lv, pts
                        )),
                );
            }
            self.banner_shown = true;
        } else if self.banner_timer <= 0.0 && self.banner_shown {
            if let Some(ui) = ctx.user_interfaces.iter_mut().next() {
                ui.send(self.banner_h, TextMessage::Text(String::new()));
            }
            self.banner_shown = false;
        }
        let p = &self.inner.player;
        let d: DerivedStats = p.derived();
        let ws = window_size(ctx);
        // --- hearts: 10 containers, each max_hp/10; lowest sliver blinks ---
        {
            let unit = (d.max_hp / 10.0).max(1.0);
            let frac = (p.hp / d.max_hp).clamp(0.0, 1.0);
            let blink_off = frac < 0.25 && ((self.inner.time * 3.0) as u64) % 2 == 1;
            // lowest non-empty heart index (for the low-HP blink)
            let mut lowest: Option<usize> = None;
            for i in (0..10).rev() {
                let f = ((p.hp - i as f32 * unit) / unit).clamp(0.0, 1.0);
                if f > 0.01 {
                    lowest = Some(i);
                    break;
                }
            }
            let mut sig = String::with_capacity(10);
            for i in 0..10 {
                let f = ((p.hp - i as f32 * unit) / unit).clamp(0.0, 1.0);
                let mut st = if f >= 0.99 { 'F' } else if f <= 0.01 { 'E' } else { 'H' };
                if blink_off && Some(i) == lowest {
                    st = 'E';
                }
                sig.push(st);
            }
            if sig != self.heart_sig {
                let old: Vec<char> = self.heart_sig.chars().collect();
                let new: Vec<char> = sig.chars().collect();
                if let Some(ui) = ctx.user_interfaces.iter_mut().next() {
                    for (i, h) in self.hearts.iter().enumerate() {
                        let nc = new.get(i).copied().unwrap_or('E');
                        if old.get(i).copied().unwrap_or('?') != nc {
                            let tex = match nc {
                                'F' => self.heart_full.clone(),
                                'H' => self.heart_half.clone(),
                                _ => self.heart_empty.clone(),
                            };
                            ui.send(*h, ImageMessage::Texture(Some(tex)));
                            // dark twin mirrors the same state for dim mode
                            if let Some(dh) = self.heart_darks.get(i) {
                                let dtex = match nc {
                                    'F' => self.heart_d_full.clone(),
                                    'H' => self.heart_d_half.clone(),
                                    _ => self.heart_d_empty.clone(),
                                };
                                ui.send(*dh, ImageMessage::Texture(Some(dtex)));
                            }
                        }
                    }
                }
                self.heart_sig = sig;
            }
        }
        // --- level line + left stats column + skill panel + items + minimap ---
        // keybinds shown in brackets, reference-style: Z/X/C/V (+1-4 alt)
        const SK_KEYS: [&str; 4] = ["Z", "X", "C", "V"];
        let skills = p.class.skills();
        let stats_txt = format!(
            "ATK {:.0}\nMAG {:.0}\nSPD {:.1}\nCRT {:.0}%\nDEF {:.0}\nRGN {:.1}\nESS {:.0}/{:.0}",
            d.phys_atk * p.class.passive().power,
            d.magic_atk * p.class.passive().power,
            d.move_speed * p.class.passive().speed,
            d.crit_chance * 100.0, d.defense, d.hp_regen * p.class.passive().regen,
            p.essence, d.max_essence
        );
        let mut items_txt = format!(
            "ITEMS\n{:.12} [{}]\nPot x{} · G {}",
            self.inner.held.name, self.inner.held.arch.name(), p.potions, p.gold
        );
        if !self.inner.loadout.is_empty() {
            let names: Vec<&str> = self.inner.loadout.iter().take(2).map(|w| w.name.split(' ').last().unwrap_or("?")).collect();
            items_txt.push_str(&format!("\nAuto({}): {}", self.inner.loadout.len(), names.join(", ")));
        }
        if !p.inventory.is_empty() {
            let names: Vec<&str> = p.inventory.iter().take(2)
                .map(|&i| GU.get(i).map(|d| d.name).unwrap_or("?"))
                .collect();
            items_txt.push_str(&format!("\n[{}]: {}", p.inventory.len(), names.join(", ")));
        }
        for loot in self.inner.loot_history.iter().take(1) {
            items_txt.push_str(&format!("\n- {:.14}", loot));
        }
        let hud_line_txt = format!("Lv{} · K{} · x{}", p.level, p.kills, p.combo_count);
        // corner-anchored panels follow the window; map cells follow on resize
        let map_sig = format!("{:.0}x{:.0}", ws.x, ws.y);
        let map_moved = map_sig != self.map_sig;
        if map_moved {
            self.map_sig = map_sig;
        }
        if let Some(ui) = ctx.user_interfaces.iter_mut().next() {
            ui.send(self.hud_line, TextMessage::Text(hud_line_txt));
            ui.send(self.left_stats_text, TextMessage::Text(stats_txt));
            ui.send(self.items_text, TextMessage::Text(items_txt));
            // skill panel: class header + per-skill name / [key] rows.
            // locked skills show ??? + their unlock level; cooldowns tick live.
            ui.send(self.sk_title, TextMessage::Text(p.class.name().to_uppercase()));
            for (i, nm) in self.sk_names.iter().enumerate() {
                let s = &skills[i];
                let locked = p.level < s.unlock_level;
                let nm_txt = if locked && s.unlock_level > 50 {
                    "???".to_string()
                } else {
                    format!("{:.14}", s.name)
                };
                ui.send(*nm, TextMessage::Text(nm_txt));
                let fg = if locked { (150, 120, 95) } else { PATCH_INK };
                ui.send(*nm, WidgetMessage::Foreground(Brush::Solid(col(fg)).into()));
                if let Some(kb) = self.sk_keys.get(i) {
                    let cd = p.skill_cooldowns[i].max(0.0);
                    let kb_txt = if locked {
                        format!("[{}]\nLv{}", SK_KEYS[i], s.unlock_level)
                    } else if cd > 0.05 {
                        format!("[{}]\n{:.0}s", SK_KEYS[i], cd)
                    } else {
                        format!("[{}]\nREADY", SK_KEYS[i])
                    };
                    ui.send(*kb, TextMessage::Text(kb_txt));
                    ui.send(*kb, WidgetMessage::Foreground(Brush::Solid(col(fg)).into()));
                }
            }
            // bottom-right items corner
            // bottom-right items corner (position anchored, size scaled)
            ui.send(self.items_panel, WidgetMessage::DesiredPosition(Vector2::new(ws.x - 248.0 * s, ws.y - 226.0 * s)));
            ui.send(self.items_panel, WidgetMessage::Width(232.0 * s));
            ui.send(self.items_panel, WidgetMessage::Height(214.0 * s));
            ui.send(self.items_dim, WidgetMessage::DesiredPosition(Vector2::new(ws.x - 248.0 * s, ws.y - 226.0 * s)));
            ui.send(self.items_dim, WidgetMessage::Width(232.0 * s));
            ui.send(self.items_dim, WidgetMessage::Height(214.0 * s));
            ui.send(self.items_text, WidgetMessage::DesiredPosition(Vector2::new(ws.x - 236.0 * s, ws.y - 214.0 * s)));
            ui.send(self.items_text, WidgetMessage::Width(208.0 * s));
            ui.send(self.items_text, WidgetMessage::Height(190.0 * s));
            // minimap top-right; cells reposition/resize only when window changes
            if map_moved {
                let cell = MAP_CELL * s;
                ui.send(self.map_panel, WidgetMessage::DesiredPosition(Vector2::new(ws.x - 328.0 * s, 36.0 * s)));
                ui.send(self.map_panel, WidgetMessage::Width(312.0 * s));
                ui.send(self.map_panel, WidgetMessage::Height(240.0 * s));
                ui.send(self.map_dim, WidgetMessage::DesiredPosition(Vector2::new(ws.x - 328.0 * s, 36.0 * s)));
                ui.send(self.map_dim, WidgetMessage::Width(312.0 * s));
                ui.send(self.map_dim, WidgetMessage::Height(240.0 * s));
                for (i, c) in self.map_cells.iter().enumerate() {
                    let cx = (i % MAP_W) as f32;
                    let cy = (i / MAP_W) as f32;
                    ui.send(*c, WidgetMessage::DesiredPosition(Vector2::new(
                        ws.x - 316.0 * s + cx * cell,
                        48.0 * s + cy * cell,
                    )));
                    ui.send(*c, WidgetMessage::Width(cell));
                    ui.send(*c, WidgetMessage::Height(cell));
                }
            }
            // minimap content: player > boss > enemy > pickup > empty
            let mut grid = [MAP_EMPTY; MAP_W * MAP_H];
            let to_cell = |x: f32, y: f32| -> Option<usize> {
                let cx = ((x + 16.0) / 32.0 * MAP_W as f32) as i32;
                let cy = ((12.0 - y) / 24.0 * MAP_H as f32) as i32;
                if (0..MAP_W as i32).contains(&cx) && (0..MAP_H as i32).contains(&cy) {
                    Some((cy as usize) * MAP_W + cx as usize)
                } else {
                    None
                }
            };
            for pk in self.inner.pickups.iter() {
                if let Some(i) = to_cell(pk.pos.0, pk.pos.1) {
                    grid[i] = MAP_PICKUP;
                }
            }
            for e in self.inner.enemies.iter() {
                if let Some(i) = to_cell(e.pos.0, e.pos.1) {
                    grid[i] = if e.boss { MAP_BOSS } else { MAP_ENEMY };
                }
            }
            if let Some(i) = to_cell(self.inner.player_pos.0, self.inner.player_pos.1) {
                grid[i] = MAP_PLAYER;
            }
            // dimmed HUD repaints the whole map darker (cache was reset above)
            if dimmed {
                for c in grid.iter_mut() {
                    *c = darken(*c, 0.35);
                }
            }
            for (i, c) in self.map_cells.iter().enumerate() {
                if self.map_cache.get(i).copied().unwrap_or((0, 0, 0)) != grid[i] {
                    ui.send(*c, WidgetMessage::Background(Brush::Solid(col(grid[i])).into()));
                    if let Some(slot) = self.map_cache.get_mut(i) {
                        *slot = grid[i];
                    }
                }
            }
        }
        // log lives on the RIGHT under the minimap, half width; long lines
        // wrap and the patch grows by *visual* lines, capped to fit the screen.
        // Skipped entirely while the LOG toggle is off.
        if self.log_on {
            let log_txt = self.inner.messages.join("\n");
        {
            let ws = window_size(ctx);
            let lw = (350.0f32).min((ws.x - 32.0).max(200.0));
            let lx = (ws.x - lw - 16.0).max(8.0);
            // below the minimap: map ends at 36+240 scaled, plus a gap
            let ly = 292.0 * s;
            // ~15px per char at 1.0 scale; bytes overcount multibyte glyphs,
            // which only errs toward a slightly taller (safe) panel.
            let per_line = ((lw / (15.0 * s)).max(10.0)) as usize;
            let mut visual = 0usize;
            for m in self.inner.messages.iter() {
                visual += (m.len().max(1) + per_line - 1) / per_line;
            }
            let visual = visual.clamp(1, 10) as f32;
            let ph = visual * 38.0 * s + 28.0 * s;
            if let Some(ui2) = ctx.user_interfaces.iter_mut().next() {
                ui2.send(self.log_h, WidgetMessage::DesiredPosition(Vector2::new(lx, ly)));
                ui2.send(self.log_h, WidgetMessage::Width(lw));
                ui2.send(self.log_h, WidgetMessage::Height(ph));
                ui2.send(self.log_bg, WidgetMessage::DesiredPosition(Vector2::new(lx - 12.0, ly - 12.0)));
                ui2.send(self.log_bg, WidgetMessage::Width(lw + 24.0));
                ui2.send(self.log_bg, WidgetMessage::Height(ph));
                ui2.send(self.log_dim, WidgetMessage::DesiredPosition(Vector2::new(lx - 12.0, ly - 12.0)));
                ui2.send(self.log_dim, WidgetMessage::Width(lw + 24.0));
                ui2.send(self.log_dim, WidgetMessage::Height(ph));
                // re-assert ink every tick so the log can never drift color
                ui2.send(self.log_h, WidgetMessage::Foreground(Brush::Solid(col(PATCH_INK)).into()));
                ui2.send(self.log_h, TextMessage::Text(log_txt));
            }
        }
        }
    }

}
