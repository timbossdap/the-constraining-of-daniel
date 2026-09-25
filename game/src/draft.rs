//! Level-up draft: frozen 3-card upgrade picks (Vampire-Survivors style).

use crate::{
    sim::GameInner,
    stats::AttrKind,
    gu::{GU, PATHS, gu_of_path_rank, gu_weapon, random_gu_rank},
    weapon_list::{Rarity, Weapon, WeaponArch},
};


/// One level-up draft card (Isaac/Vampire-Survivors style 3-pick).
#[derive(Debug, Clone)]
pub(crate) enum DraftKind {
    Stat(AttrKind, u32),
    Heal(f32),
    Potions(u32),
    EssenceFull,
    Weapon(Weapon),
    Shield(f32),
    Combo(u32),
    Gold(u32),
    /// Milestone boon: a specific Gu worm, granted + equipped on pick.
    GuWorm(usize),
}

#[derive(Debug, Clone)]
pub(crate) struct DraftCard {
    pub(crate) title: String,
    pub(crate) desc: String,
    pub(crate) kind: DraftKind,
}

impl GameInner {

    /// Opens the next queued draft (freezes the game until picked).
    pub(crate) fn open_draft_if_needed(&mut self) {
        if self.draft_open || self.draft_queue == 0 {
            return;
        }
        self.draft_queue -= 1;
        self.draft_id += 1;
        self.gen_draft_cards();
        self.draft_open = true;
        self.draft_sel = 0;
        self.draft_lock = 1.0;
        self.log("LEVEL UP! Pick an upgrade — game frozen.".to_string());
    }

    /// 12-option pool → 3 distinct cards.
    pub(crate) fn gen_draft_cards(&mut self) {
        let mut pool: Vec<u32> = (0..12).collect();
        let mut picks = Vec::new();
        for _ in 0..3 {
            if pool.is_empty() {
                break;
            }
            let s = self.next_seed();
            picks.push(pool.remove((s as usize) % pool.len()));
        }
        self.draft_cards.clear();
        for p in picks {
            let card = self.make_card(p);
            self.draft_cards.push(card);
        }
    }

    pub(crate) fn make_card(&mut self, idx: u32) -> DraftCard {
        match idx {
            0 => DraftCard { title: "IRON ARMS".to_string(), desc: "+1 STR · phys · HP".to_string(), kind: DraftKind::Stat(AttrKind::Strength, 1) },
            1 => DraftCard { title: "SWIFT FEET".to_string(), desc: "+1 AGI · speed · dodge".to_string(), kind: DraftKind::Stat(AttrKind::Agility, 1) },
            2 => DraftCard { title: "ARCANE MIND".to_string(), desc: "+1 INT · magic · essence".to_string(), kind: DraftKind::Stat(AttrKind::Intellect, 1) },
            3 => DraftCard { title: "OAK HEART".to_string(), desc: "+1 VIT · HP · regen".to_string(), kind: DraftKind::Stat(AttrKind::Vitality, 1) },
            4 => DraftCard { title: "LUCKY COIN".to_string(), desc: "+1 LUK · crit · loot".to_string(), kind: DraftKind::Stat(AttrKind::Luck, 1) },
            5 => DraftCard { title: "FEAST".to_string(), desc: "Heal 45% HP".to_string(), kind: DraftKind::Heal(0.45) },
            6 => DraftCard { title: "POTION BELT".to_string(), desc: "+1 potion · press H".to_string(), kind: DraftKind::Potions(1) },
            7 => DraftCard { title: "ESSENCE FONT".to_string(), desc: "Full essence".to_string(), kind: DraftKind::EssenceFull },
            8 => DraftCard { title: "IRON SKIN".to_string(), desc: "+30 shield · 20s".to_string(), kind: DraftKind::Shield(30.0) },
            9 => DraftCard { title: "ADRENALINE".to_string(), desc: "+6 combo · more dmg".to_string(), kind: DraftKind::Combo(6) },
            10 => {
                // Gu cache: a rank 2-4 Gu from the great cycle, auto-firing
                let s = self.next_seed();
                let idx = random_gu_rank(2, 4, s);
                let w = gu_weapon(idx, s).unwrap_or(Weapon {
                    name: "Rare Battlebrand".to_string(),
                    rarity: Rarity::Rare,
                    bonus_atk: 14.0,
                    arch: WeaponArch::Blade,
                });
                let title = w.name.clone();
                let desc = format!("NEW: +{:.0} atk, auto-fires", w.bonus_atk);
                DraftCard { title, desc, kind: DraftKind::Weapon(w) }
            }
            _ => DraftCard { title: "GREED".to_string(), desc: "+25 gold".to_string(), kind: DraftKind::Gold(25) },
        }
    }

    /// Applies the picked card; opens the next queued draft if any.
    pub(crate) fn apply_draft(&mut self, idx: usize) {
        if idx >= self.draft_cards.len() {
            return;
        }
        let card = self.draft_cards[idx].clone();
        match card.kind {
            DraftKind::Stat(kind, n) => {
                for _ in 0..n {
                    // direct (does not consume unspent level points — draft is a bonus)
                    match kind {
                        AttrKind::Strength => self.player.base_attrs.strength += 1,
                        AttrKind::Agility => self.player.base_attrs.agility += 1,
                        AttrKind::Intellect => self.player.base_attrs.intellect += 1,
                        AttrKind::Vitality => self.player.base_attrs.vitality += 1,
                        AttrKind::Luck => self.player.base_attrs.luck += 1,
                    }
                }
                let d = self.player.derived();
                self.player.hp = self.player.hp.min(d.max_hp);
                self.player.essence = self.player.essence.min(d.max_essence);
            }
            DraftKind::Heal(frac) => {
                let d = self.player.derived();
                self.player.hp = (self.player.hp + d.max_hp * frac).min(d.max_hp);
                self.burst(self.player_pos, (120, 255, 140), 12, 3.5, 0.6, 0.22);
            }
            DraftKind::Potions(n) => {
                self.player.potions = (self.player.potions + n).min(9);
            }
            DraftKind::EssenceFull => {
                self.player.essence = self.player.derived().max_essence;
            }
            DraftKind::Weapon(w) => {
                let label = self.equip_weapon(w);
                self.log(format!("WEAPON: {} joins! (auto-fires)", label));
                self.burst(self.player_pos, (255, 220, 130), 14, 4.0, 0.6, 0.22);
            }
            DraftKind::Shield(amt) => {
                self.player.shield_hp = amt + self.player.level as f32 * 2.0;
                self.player.shield_timer = 20.0;
            }
            DraftKind::Combo(n) => {
                for _ in 0..n {
                    self.player.register_hit();
                }
            }
            DraftKind::Gold(n) => {
                self.player.gold += n;
            }
            DraftKind::GuWorm(idx) => {
                // milestone boon: the worm joins the body AND the hand
                let got = self.player.grant_gu(idx);
                let s = self.next_seed();
                if let Some(w) = gu_weapon(idx, s) {
                    self.equip_weapon(w);
                }
                self.log(format!("BOON: {} joins!", got));
                self.burst(self.player_pos, (255, 220, 130), 14, 4.0, 0.6, 0.22);
            }
        }
        self.log(format!("Chosen: {}!", card.title));
        self.draft_cards.clear();
        self.draft_open = false;
        // chained level-ups re-open immediately
        self.open_draft_if_needed();
    }

    /// Milestone boon: choose 1 of 3 worms of your own path at the milestone
    /// rank (rank 2 at Lv 5, 3 at 10, 4 at 15, 5 at 20, 6 at 25+). Thin global
    /// pools fall back to any path so the cards always fill.
    pub(crate) fn open_gu_boon(&mut self, rank: u8) {
        if self.draft_open {
            return;
        }
        let path = self.path.unwrap_or(0);
        let mut pool = gu_of_path_rank(path, rank);
        if pool.len() < 3 {
            for idx in 0..GU.len() {
                if pool.len() >= 6 {
                    break;
                }
                if GU[idx].rank == rank && !pool.contains(&idx) {
                    pool.push(idx);
                }
            }
        }
        let mut cards = Vec::new();
        for _ in 0..3 {
            if pool.is_empty() {
                break;
            }
            let s = self.next_seed();
            let idx = pool.remove((s as usize) % pool.len());
            let def = &GU[idx];
            cards.push(DraftCard {
                title: def.name.to_string(),
                desc: format!("{} · rank {} {}", PATHS[def.path as usize].name, def.rank, def.desc),
                kind: DraftKind::GuWorm(idx),
            });
        }
        if cards.is_empty() {
            return;
        }
        self.draft_cards = cards;
        self.draft_open = true;
        self.draft_sel = 0;
        self.draft_id += 1;
        self.draft_lock = 1.0;
        self.log(format!("MILESTONE! Choose a rank-{rank} worm."));
    }
}
