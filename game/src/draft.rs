//! Level-up draft: frozen 3-card upgrade picks (Vampire-Survivors style).

use crate::{
    player_class::CharacterClass,
    sim::GameInner,
    stats::AttrKind,
    weapon_list::{Rarity, Weapon, WeaponArch, roll_loot},
};


/// One level-up draft card (Isaac/Vampire-Survivors style 3-pick).
#[derive(Debug, Clone)]
pub(crate) enum DraftKind {
    Stat(AttrKind, u32),
    Heal(f32),
    Potions(u32),
    ManaFull,
    Weapon(Weapon),
    Shield(f32),
    Combo(u32),
    Gold(u32),
    /// Class evolution pick (checkpoint drafts).
    Class(CharacterClass),
    /// Paragon surge: stay your class, +2 all stats, full heal.
    Surge,
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
        if self.draft_open || self.class_open || self.draft_queue == 0 {
            return;
        }
        self.draft_queue -= 1;
        self.draft_id += 1;
        self.gen_draft_cards();
        self.draft_open = true;
        self.draft_sel = 0;
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
            2 => DraftCard { title: "ARCANE MIND".to_string(), desc: "+1 INT · magic · mana".to_string(), kind: DraftKind::Stat(AttrKind::Intellect, 1) },
            3 => DraftCard { title: "OAK HEART".to_string(), desc: "+1 VIT · HP · regen".to_string(), kind: DraftKind::Stat(AttrKind::Vitality, 1) },
            4 => DraftCard { title: "LUCKY COIN".to_string(), desc: "+1 LUK · crit · loot".to_string(), kind: DraftKind::Stat(AttrKind::Luck, 1) },
            5 => DraftCard { title: "FEAST".to_string(), desc: "Heal 45% HP".to_string(), kind: DraftKind::Heal(0.45) },
            6 => DraftCard { title: "POTION BELT".to_string(), desc: "+1 potion · press H".to_string(), kind: DraftKind::Potions(1) },
            7 => DraftCard { title: "MANA FONT".to_string(), desc: "Full mana".to_string(), kind: DraftKind::ManaFull },
            8 => DraftCard { title: "IRON SKIN".to_string(), desc: "+30 shield · 20s".to_string(), kind: DraftKind::Shield(30.0) },
            9 => DraftCard { title: "ADRENALINE".to_string(), desc: "+6 combo · more dmg".to_string(), kind: DraftKind::Combo(6) },
            10 => {
                // weapon cache: always offers Rare or better
                let mut s = self.next_seed();
                let mut got: Option<Weapon> = None;
                for _ in 0..8 {
                    if let Some(cand) = roll_loot(self.player.kills + 10, self.player.base_attrs.luck + 3, s) {
                        got = Some(cand);
                        break;
                    }
                    s = s.wrapping_add(97);
                }
                let mut w = got.unwrap_or(Weapon {
                    name: "Rare Battlebrand".to_string(),
                    rarity: Rarity::Rare,
                    bonus_atk: 14.0,
                    arch: WeaponArch::Blade,
                });
                if matches!(w.rarity, Rarity::Common | Rarity::Magic) {
                    w.rarity = Rarity::Rare;
                    w.bonus_atk = (12.0 + (s % 8) as f32) * w.rarity.multiplier();
                }
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
                self.player.mp = self.player.mp.min(d.max_mp);
            }
            DraftKind::Heal(frac) => {
                let d = self.player.derived();
                self.player.hp = (self.player.hp + d.max_hp * frac).min(d.max_hp);
                self.burst(self.player_pos, (120, 255, 140), 12, 3.5, 0.6, 0.22);
            }
            DraftKind::Potions(n) => {
                self.player.potions = (self.player.potions + n).min(9);
            }
            DraftKind::ManaFull => {
                self.player.mp = self.player.derived().max_mp;
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
            // Class/Surge cards belong to checkpoint drafts (apply_class).
            // If one ever lands here, ignore it rather than crashing the pick.
            DraftKind::Class(_) | DraftKind::Surge => {
                return;
            }
        }
        self.log(format!("Chosen: {}!", card.title));
        self.draft_cards.clear();
        self.draft_open = false;
        // chained level-ups re-open immediately
        self.open_draft_if_needed();
    }

    /// Checkpoint offers live in the same 3-card UI:
    /// - Drifters (tier 0) pick 1 of 3 random base classes (first class).
    /// - Base classes (tier 1) pick a 2-branch evolution or Paragon surge.
    pub(crate) fn open_class_draft(&mut self) {
        if self.class_open {
            return;
        }
        let tier = self.player.class.tier();
        let mut cards = Vec::new();
        if tier == 0 {
            let mut pool: Vec<usize> = (0..8).collect();
            for _ in 0..3 {
                if pool.is_empty() {
                    break;
                }
                let s = self.next_seed();
                let c = CharacterClass::from_index(pool.remove((s as usize) % pool.len()));
                cards.push(DraftCard {
                    title: c.name().to_string(),
                    desc: c.title().to_string(),
                    kind: DraftKind::Class(c),
                });
            }
        } else {
            for evo in self.player.class.evolutions() {
                cards.push(DraftCard {
                    title: evo.name().to_string(),
                    desc: evo.title().to_string(),
                    kind: DraftKind::Class(evo),
                });
            }
            cards.push(DraftCard {
                title: "PARAGON".to_string(),
                desc: "+2 all stats · full heal".to_string(),
                kind: DraftKind::Surge,
            });
        }
        if cards.is_empty() {
            return;
        }
        self.draft_cards = cards;
        self.class_open = true;
        self.draft_sel = 0;
        self.draft_id += 1;
        self.log("CHECKPOINT! Choose your evolution.".to_string());
    }

    /// Applies a class evolution pick from the checkpoint cards: new base +
    /// new growth curve, carrying every earned point above the old base.
    /// Unspent points are untouched — spend them in the Tab menu.
    /// Paragon surge instead: +2 all stats and a full heal, same class.
    pub(crate) fn apply_class(&mut self, idx: usize) {
        let Some(card) = self.draft_cards.get(idx).cloned() else {
            return;
        };
        match card.kind {
            DraftKind::Surge => {
                let ba = &mut self.player.base_attrs;
                ba.strength += 2;
                ba.agility += 2;
                ba.intellect += 2;
                ba.vitality += 2;
                ba.luck += 2;
                let d = self.player.derived();
                self.player.hp = d.max_hp;
                self.player.mp = d.max_mp;
                self.log("PARAGON SURGE! +2 all stats, fully healed.".to_string());
                self.burst(self.player_pos, (255, 240, 150), 24, 6.0, 1.0, 0.28);
            }
            DraftKind::Class(c) => {
                let old = self.player.class;
                let m = self.player.level.saturating_sub(1);
                let nb = c.base_attributes();
                let ng = c.growth_per_level();
                let ob = old.base_attributes();
                let og = old.growth_per_level();
                let carried = |new_base: u32, new_grow: u32, had: u32, old_base: u32, old_grow: u32| -> u32 {
                    new_base + new_grow * m + had.saturating_sub(old_base + old_grow * m)
                };
                {
                    let ba = &mut self.player.base_attrs;
                    let had = *ba;
                    ba.strength = carried(nb.strength, ng.strength, had.strength, ob.strength, og.strength);
                    ba.agility = carried(nb.agility, ng.agility, had.agility, ob.agility, og.agility);
                    ba.intellect = carried(nb.intellect, ng.intellect, had.intellect, ob.intellect, og.intellect);
                    ba.vitality = carried(nb.vitality, ng.vitality, had.vitality, ob.vitality, og.vitality);
                    ba.luck = carried(nb.luck, ng.luck, had.luck, ob.luck, og.luck);
                }
                self.player.class = c;
                let d = self.player.derived();
                self.player.hp = self.player.hp.min(d.max_hp);
                self.player.mp = self.player.mp.min(d.max_mp);
                self.log(format!("EVOLVED into {} — {}!", c.name(), c.title()));
                self.burst(self.player_pos, (255, 240, 150), 24, 6.0, 1.0, 0.28);
            }
            _ => return,
        }
        self.draft_cards.clear();
        self.class_open = false;
        // leftover queued upgrade picks resolve after the evolution
        self.open_draft_if_needed();
    }

}
