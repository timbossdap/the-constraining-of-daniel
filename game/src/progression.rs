//! XP curve, leveling, stat allocation. Designed "fun > grind":
//! generous XP, 3 points/level, catch-up bonus, rest bonus.

use crate::items::{ItemTotals, ITEMS};
use crate::player_class::CharacterClass;
use crate::stats::{AttrKind, Attributes, DerivedStats};

pub const MAX_LEVEL: u32 = 50;
pub const POINTS_PER_LEVEL: u32 = 3;

pub fn xp_for_level(level: u32) -> u32 {
    // 80 * level^1.45 — fast early, smooth late. L50 ≈ 20k total, very reachable.
    (80.0 * (level as f32).powf(1.45)) as u32
}

#[derive(Debug, Clone)]
pub struct PlayerCore {
    pub class: CharacterClass,
    pub level: u32,
    pub xp: u32,
    pub base_attrs: Attributes,
    pub unspent_points: u32,
    pub hp: f32,
    pub mp: f32,
    /// extra buff timers
    pub shield_hp: f32,
    pub shield_timer: f32,
    pub combo_count: u32,
    pub combo_timer: f32,
    pub skill_cooldowns: [f32; 4],
    pub potions: u32,
    pub gold: u32,
    pub kills: u32,
    /// collected item indices into ITEMS (chest loot only)
    pub inventory: Vec<usize>,
}

impl PlayerCore {
    pub fn new(class: CharacterClass) -> Self {
        let base = class.base_attributes();
        let d = DerivedStats::from_attributes(&base, 1);
        Self {
            class,
            level: 1,
            xp: 0,
            base_attrs: base,
            unspent_points: 0,
            hp: d.max_hp,
            mp: d.max_mp,
            shield_hp: 0.0,
            shield_timer: 0.0,
            combo_count: 0,
            combo_timer: 0.0,
            skill_cooldowns: [0.0; 4],
            potions: 2,
            gold: 0,
            kills: 0,
            inventory: Vec::new(),
        }
    }

    pub fn derived(&self) -> DerivedStats {
        DerivedStats::from_attributes(&self.base_attrs, self.level)
    }

    /// Summed-up inventory power (see items.rs). Read at every use site so
    /// pickups apply instantly with no caching bugs.
    pub fn item_totals(&self) -> ItemTotals {
        ItemTotals::of_inventory(&self.inventory)
    }

    /// Grants an item by table index: stat tonics land directly in base
    /// attributes, rule-benders join the inventory for ongoing effect.
    /// Returns the display name for logs.
    pub fn grant_item(&mut self, idx: usize) -> String {
        let Some(def) = ITEMS.get(idx) else {
            return "???".to_string();
        };
        let mut applied_stat: Option<String> = None;
        for fx in def.effects.iter().copied() {
            if let crate::items::ItemEffect::Stat(kind, n) = fx {
                match kind {
                    AttrKind::Strength => self.base_attrs.strength += n,
                    AttrKind::Agility => self.base_attrs.agility += n,
                    AttrKind::Intellect => self.base_attrs.intellect += n,
                    AttrKind::Vitality => self.base_attrs.vitality += n,
                    AttrKind::Luck => self.base_attrs.luck += n,
                }
                applied_stat = Some(format!("+{n} {}", kind.name()));
            }
        }
        if applied_stat.is_none() {
            self.inventory.push(idx);
        }
        let d = self.derived();
        self.hp = self.hp.min(d.max_hp);
        self.mp = self.mp.min(d.max_mp);
        if let Some(s) = applied_stat {
            format!("{} ({s})", def.name)
        } else {
            def.name.to_string()
        }
    }

    pub fn xp_needed(&self) -> u32 {
        if self.level >= MAX_LEVEL {
            return u32::MAX;
        }
        xp_for_level(self.level)
    }

    /// Returns number of level-ups.
    pub fn add_xp(&mut self, amount: u32) -> u32 {
        if self.level >= MAX_LEVEL {
            return 0;
        }
        // Catch-up: +25% if behind zone curve (kills low for level) — anti-grind.
        let catchup = if self.kills < self.level * 6 { 1.25 } else { 1.0 };
        let gain = ((amount as f32) * catchup * self.item_totals().xp_mult) as u32;
        self.xp += gain;
        let mut ups = 0;
        while self.level < MAX_LEVEL && self.xp >= self.xp_needed() {
            self.xp -= self.xp_needed();
            self.level += 1;
            self.unspent_points += POINTS_PER_LEVEL;
            // auto growth
            let g = self.class.growth_per_level();
            self.base_attrs.strength += g.strength;
            self.base_attrs.agility += g.agility;
            self.base_attrs.intellect += g.intellect;
            self.base_attrs.vitality += g.vitality;
            self.base_attrs.luck += g.luck;
            // full-ish heal on level: fun!
            let d = self.derived();
            self.hp = (self.hp + d.max_hp * 0.35).min(d.max_hp);
            self.mp = d.max_mp;
            self.potions = (self.potions + 1).min(9);
            ups += 1;
        }
        ups
    }

    pub fn allocate(&mut self, kind: AttrKind) -> bool {
        if self.unspent_points == 0 {
            return false;
        }
        match kind {
            AttrKind::Strength => self.base_attrs.strength += 1,
            AttrKind::Agility => self.base_attrs.agility += 1,
            AttrKind::Intellect => self.base_attrs.intellect += 1,
            AttrKind::Vitality => self.base_attrs.vitality += 1,
            AttrKind::Luck => self.base_attrs.luck += 1,
        }
        self.unspent_points -= 1;
        // allocating VIT/STR immediately heals a bit — feels good
        let d = self.derived();
        self.hp = self.hp.min(d.max_hp);
        self.mp = self.mp.min(d.max_mp);
        true
    }

    /// Refunds one point of `kind` back to the pool. Never drops below the
    /// class starting attributes, so level growth and draft bonuses can only
    /// be refunded down to base — never into debt.
    pub fn deallocate(&mut self, kind: AttrKind) -> bool {
        let base = self.class.base_attributes();
        let (cur, min) = match kind {
            AttrKind::Strength => (self.base_attrs.strength, base.strength),
            AttrKind::Agility => (self.base_attrs.agility, base.agility),
            AttrKind::Intellect => (self.base_attrs.intellect, base.intellect),
            AttrKind::Vitality => (self.base_attrs.vitality, base.vitality),
            AttrKind::Luck => (self.base_attrs.luck, base.luck),
        };
        if cur <= min {
            return false;
        }
        match kind {
            AttrKind::Strength => self.base_attrs.strength -= 1,
            AttrKind::Agility => self.base_attrs.agility -= 1,
            AttrKind::Intellect => self.base_attrs.intellect -= 1,
            AttrKind::Vitality => self.base_attrs.vitality -= 1,
            AttrKind::Luck => self.base_attrs.luck -= 1,
        }
        self.unspent_points += 1;
        let d = self.derived();
        self.hp = self.hp.min(d.max_hp);
        self.mp = self.mp.min(d.max_mp);
        true
    }
    /// Smart auto-allocate for players who hate menus (press T).
    pub fn auto_allocate(&mut self) {
        while self.unspent_points > 0 {
            let k = match self.class {
                CharacterClass::Knight => AttrKind::Vitality,
                CharacterClass::Ranger => AttrKind::Agility,
                CharacterClass::Pyromancer => AttrKind::Intellect,
                CharacterClass::Frostwarden => AttrKind::Intellect,
                CharacterClass::Rogue => {
                    if self.base_attrs.agility % 3 == 0 {
                        AttrKind::Luck
                    } else {
                        AttrKind::Agility
                    }
                }
                CharacterClass::Cleric => {
                    if self.base_attrs.vitality <= self.base_attrs.intellect {
                        AttrKind::Vitality
                    } else {
                        AttrKind::Intellect
                    }
                }
                CharacterClass::Necromancer => AttrKind::Intellect,
                CharacterClass::Spellblade => {
                    if self.base_attrs.strength <= self.base_attrs.intellect {
                        AttrKind::Strength
                    } else {
                        AttrKind::Intellect
                    }
                }
                CharacterClass::Drifter => AttrKind::Vitality,
                CharacterClass::Crusader => AttrKind::Vitality,
                CharacterClass::Doomblade => AttrKind::Strength,
                CharacterClass::Deadeye => AttrKind::Agility,
                CharacterClass::Nightstalker => AttrKind::Agility,
                CharacterClass::Inferno => AttrKind::Intellect,
                CharacterClass::Ashcaller => AttrKind::Intellect,
                CharacterClass::Glacial => AttrKind::Vitality,
                CharacterClass::Stormcaller => AttrKind::Intellect,
                CharacterClass::Assassin => AttrKind::Agility,
                CharacterClass::Trickster => AttrKind::Agility,
                CharacterClass::Saint => {
                    if self.base_attrs.vitality <= self.base_attrs.intellect {
                        AttrKind::Vitality
                    } else {
                        AttrKind::Intellect
                    }
                }
                CharacterClass::Inquisitor => AttrKind::Intellect,
                CharacterClass::Lich => AttrKind::Intellect,
                CharacterClass::Plaguebearer => AttrKind::Intellect,
                CharacterClass::Runelord => AttrKind::Intellect,
                CharacterClass::Hexblade => {
                    if self.base_attrs.strength <= self.base_attrs.intellect {
                        AttrKind::Strength
                    } else {
                        AttrKind::Intellect
                    }
                }
            };
            self.allocate(k);
        }
    }

    pub fn combo_multiplier(&self) -> f32 {
        // 1.0 -> 1.6 at 20 combo. Rewards aggression without exploding.
        1.0 + (self.combo_count.min(20) as f32) * 0.03
    }

    pub fn tick(&mut self, dt: f32) {
        let d = self.derived();
        // regen
        self.hp = (self.hp + (d.hp_regen * self.class.passive().regen + self.item_totals().regen_add) * dt).min(d.max_hp);
        self.mp = (self.mp + d.mp_regen * dt).min(d.max_mp);
        for cd in self.skill_cooldowns.iter_mut() {
            if *cd > 0.0 {
                *cd -= dt;
            }
        }
        if self.combo_timer > 0.0 {
            self.combo_timer -= dt;
            if self.combo_timer <= 0.0 {
                self.combo_count = 0;
            }
        }
        if self.shield_timer > 0.0 {
            self.shield_timer -= dt;
            if self.shield_timer <= 0.0 {
                self.shield_hp = 0.0;
            }
        }
    }

    pub fn register_hit(&mut self) {
        self.combo_count += 1;
        self.combo_timer = self.class.passive().combo_window;
    }

    pub fn take_damage(&mut self, raw: f32) -> f32 {
        let d = self.derived();
        // class guard: tanks shave hits down, glass cannons feel them fully
        let raw = raw * self.class.passive().guard * self.item_totals().guard_mult;
        // deterministic dodge check uses luck-seeded pseudo random via kills+hp — no RNG dep
        let dodge_roll = ((self.kills.wrapping_mul(73) as f32 + self.hp * 13.7) % 100.0) / 100.0;
        if dodge_roll < d.dodge_chance + self.item_totals().dodge_add {
            return 0.0;
        }
        let mitigated = (raw - d.defense * 0.7).max(raw * 0.25);
        let mut left = mitigated;
        if self.shield_hp > 0.0 {
            let absorbed = left.min(self.shield_hp);
            self.shield_hp -= absorbed;
            left -= absorbed;
        }
        self.hp -= left;
        // getting hit halves combo — still forgiving
        self.combo_count /= 2;
        self.combo_timer = self.combo_timer.min(1.5);
        left
    }

    pub fn drink_potion(&mut self) -> bool {
        if self.potions == 0 {
            return false;
        }
        let d = self.derived();
        if self.hp >= d.max_hp * 0.99 {
            return false;
        }
        self.potions -= 1;
        self.hp = (self.hp + d.max_hp * 0.45 * self.class.passive().potion * self.item_totals().potion_mult).min(d.max_hp);
        self.mp = (self.mp + d.max_mp * 0.3).min(d.max_mp);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_fast() {
        let mut p = PlayerCore::new(CharacterClass::Knight);
        let ups = p.add_xp(500);
        assert!(ups >= 2);
        assert!(p.unspent_points >= 6);
    }

    #[test]
    fn refund_floors_at_base() {
        let mut p = PlayerCore::new(CharacterClass::Knight);
        let base_str = p.class.base_attributes().strength;
        assert!(!p.deallocate(AttrKind::Strength)); // nothing to refund
        p.unspent_points = 2;
        assert!(p.allocate(AttrKind::Strength));
        assert!(p.allocate(AttrKind::Strength));
        assert_eq!(p.unspent_points, 0);
        assert!(p.deallocate(AttrKind::Strength));
        assert_eq!(p.base_attrs.strength, base_str + 1);
        assert_eq!(p.unspent_points, 1);
        assert!(p.deallocate(AttrKind::Strength));
        assert!(!p.deallocate(AttrKind::Strength)); // at base: stuck
        assert_eq!(p.base_attrs.strength, base_str);
    }

    #[test]
    fn item_totals_stack() {
        use crate::items::ITEMS;
        let mut p = PlayerCore::new(CharacterClass::Knight);
        let t0 = p.item_totals();
        assert_eq!(t0.power_mult, 1.0);
        assert_eq!(t0.homing, 0.0);
        let tele = ITEMS.iter().position(|d| d.name == "Telepathy").unwrap();
        let nail = ITEMS.iter().position(|d| d.name == "Rusty Nail").unwrap();
        p.inventory.push(tele);
        assert_eq!(p.base_attrs.strength, 1); // everyone starts at base 1
        let label = p.grant_item(nail);
        assert!(label.contains("+1"));
        assert_eq!(p.base_attrs.strength, 2);
        let t = p.item_totals();
        assert!(t.homing > 0.0);
        assert!(p.inventory.contains(&tele));
        assert!(!p.inventory.contains(&nail)); // stat tonics don't linger
    }
}
