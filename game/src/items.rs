//! Item armory: 43 take-home treasures. 25 are plain stat buffs, 18 bend
//! the rules (homing shots, thorns, magnets...). Ranked by rarity — a
//! telepathy-style game-changer is Epic, a double-dip combo is Legendary.
//!
//! Weapons and items NEVER drop from enemies directly: kills rarely (5%)
//! drop a chest, and a chest holds gear only ~10% of the time.

use crate::stats::AttrKind;
use crate::weapon_list::Rarity;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemEffect {
    Stat(AttrKind, u32),
    Power(f32),
    Guard(f32),
    Speed(f32),
    Regen(f32),
    Crit(f32),
    Range(f32),
    Homing(f32),
    Arrows(u32),
    Lifesteal(f32),
    Thorns(f32),
    Magnet(f32),
    Dodge(f32),
    Potion(f32),
    Xp(f32),
    ShieldKill(f32),
    Gold(f32),
}

#[derive(Debug, Clone, Copy)]
pub struct ItemDef {
    pub name: &'static str,
    pub rarity: Rarity,
    pub desc: &'static str,
    pub effects: &'static [ItemEffect],
}

use ItemEffect as E;

pub const ITEMS: [ItemDef; 43] = [
    // ---- Common stat tonics (10) ----
    ItemDef { name: "Rusty Nail", rarity: Rarity::Common, desc: "+1 STR", effects: &[E::Stat(AttrKind::Strength, 1)] },
    ItemDef { name: "Swift Socks", rarity: Rarity::Common, desc: "+1 AGI", effects: &[E::Stat(AttrKind::Agility, 1)] },
    ItemDef { name: "Dull Prism", rarity: Rarity::Common, desc: "+1 INT", effects: &[E::Stat(AttrKind::Intellect, 1)] },
    ItemDef { name: "Pebble Heart", rarity: Rarity::Common, desc: "+1 VIT", effects: &[E::Stat(AttrKind::Vitality, 1)] },
    ItemDef { name: "Bent Coin", rarity: Rarity::Common, desc: "+1 LUK", effects: &[E::Stat(AttrKind::Luck, 1)] },
    ItemDef { name: "Iron Ration", rarity: Rarity::Common, desc: "+2 STR", effects: &[E::Stat(AttrKind::Strength, 2)] },
    ItemDef { name: "Tailwind", rarity: Rarity::Common, desc: "+2 AGI", effects: &[E::Stat(AttrKind::Agility, 2)] },
    ItemDef { name: "Old Tome", rarity: Rarity::Common, desc: "+2 INT", effects: &[E::Stat(AttrKind::Intellect, 2)] },
    ItemDef { name: "Hearty Stew", rarity: Rarity::Common, desc: "+2 VIT", effects: &[E::Stat(AttrKind::Vitality, 2)] },
    ItemDef { name: "Four-Leaf", rarity: Rarity::Common, desc: "+2 LUK", effects: &[E::Stat(AttrKind::Luck, 2)] },
    // ---- Magic stat tonics (9) ----
    ItemDef { name: "Steel Plate", rarity: Rarity::Magic, desc: "+3 STR", effects: &[E::Stat(AttrKind::Strength, 3)] },
    ItemDef { name: "Jet Boots", rarity: Rarity::Magic, desc: "+3 AGI", effects: &[E::Stat(AttrKind::Agility, 3)] },
    ItemDef { name: "Star Chart", rarity: Rarity::Magic, desc: "+3 INT", effects: &[E::Stat(AttrKind::Intellect, 3)] },
    ItemDef { name: "Oak Barrel", rarity: Rarity::Magic, desc: "+3 VIT", effects: &[E::Stat(AttrKind::Vitality, 3)] },
    ItemDef { name: "Loaded Die", rarity: Rarity::Magic, desc: "+3 LUK", effects: &[E::Stat(AttrKind::Luck, 3)] },
    ItemDef { name: "Titan Belt", rarity: Rarity::Magic, desc: "+4 STR", effects: &[E::Stat(AttrKind::Strength, 4)] },
    ItemDef { name: "Titan Heart", rarity: Rarity::Magic, desc: "+4 VIT", effects: &[E::Stat(AttrKind::Vitality, 4)] },
    ItemDef { name: "Titan Mind", rarity: Rarity::Magic, desc: "+4 INT", effects: &[E::Stat(AttrKind::Intellect, 4)] },
    ItemDef { name: "Titan Step", rarity: Rarity::Magic, desc: "+4 AGI", effects: &[E::Stat(AttrKind::Agility, 4)] },
    // ---- Rare stat tonics (6) ----
    ItemDef { name: "Godslayer Hilt", rarity: Rarity::Rare, desc: "+5 STR", effects: &[E::Stat(AttrKind::Strength, 5)] },
    ItemDef { name: "Godslayer String", rarity: Rarity::Rare, desc: "+5 AGI", effects: &[E::Stat(AttrKind::Agility, 5)] },
    ItemDef { name: "Godslayer Lens", rarity: Rarity::Rare, desc: "+5 INT", effects: &[E::Stat(AttrKind::Intellect, 5)] },
    ItemDef { name: "Godslayer Core", rarity: Rarity::Rare, desc: "+5 VIT", effects: &[E::Stat(AttrKind::Vitality, 5)] },
    ItemDef { name: "Godslayer Charm", rarity: Rarity::Rare, desc: "+5 LUK", effects: &[E::Stat(AttrKind::Luck, 5)] },
    ItemDef { name: "World Turtle", rarity: Rarity::Rare, desc: "+6 VIT", effects: &[E::Stat(AttrKind::Vitality, 6)] },
    // ---- Rare rule-benders (7) ----
    ItemDef { name: "Glasses", rarity: Rarity::Rare, desc: "Shots fly 60% farther.", effects: &[E::Range(1.6)] },
    ItemDef { name: "Magnet Charm", rarity: Rarity::Rare, desc: "Pickups drift to you.", effects: &[E::Magnet(4.5)] },
    ItemDef { name: "Lucky Penny", rarity: Rarity::Rare, desc: "+8% crit.", effects: &[E::Crit(0.08)] },
    ItemDef { name: "Swift Boots", rarity: Rarity::Rare, desc: "Move 12% faster.", effects: &[E::Speed(1.12)] },
    ItemDef { name: "Stone Skin", rarity: Rarity::Rare, desc: "Take 8% less damage.", effects: &[E::Guard(0.92)] },
    ItemDef { name: "Smoke Bomb", rarity: Rarity::Rare, desc: "+8% dodge.", effects: &[E::Dodge(0.08)] },
    ItemDef { name: "Campfire", rarity: Rarity::Rare, desc: "+1.5 HP/s regen.", effects: &[E::Regen(1.5)] },
    // ---- Epic game-changers (6) ----
    ItemDef { name: "Telepathy", rarity: Rarity::Epic, desc: "Shots seek foes.", effects: &[E::Homing(2.2)] },
    ItemDef { name: "Vampire Fang", rarity: Rarity::Epic, desc: "Heal 6% of hits.", effects: &[E::Lifesteal(0.06)] },
    ItemDef { name: "Thornmail", rarity: Rarity::Epic, desc: "Reflect 25% touch.", effects: &[E::Thorns(0.25)] },
    ItemDef { name: "Triple Quiver", rarity: Rarity::Epic, desc: "+1 hold arrow.", effects: &[E::Arrows(1)] },
    ItemDef { name: "Greed Sigil", rarity: Rarity::Epic, desc: "+50% gold.", effects: &[E::Gold(1.5)] },
    ItemDef { name: "Big Flask", rarity: Rarity::Epic, desc: "+40% potion heal.", effects: &[E::Potion(1.4)] },
    // ---- Legendary double-dips (5) ----
    ItemDef { name: "Mind and Might", rarity: Rarity::Legendary, desc: "Seeking, brutal shots.", effects: &[E::Homing(2.2), E::Power(1.25)] },
    ItemDef { name: "Blood Crown", rarity: Rarity::Legendary, desc: "Drink deep.", effects: &[E::Lifesteal(0.1), E::Power(1.15)] },
    ItemDef { name: "Aegis", rarity: Rarity::Legendary, desc: "Kills grant 15 shield.", effects: &[E::Guard(0.85), E::ShieldKill(15.0)] },
    ItemDef { name: "Titan Soul", rarity: Rarity::Legendary, desc: "Fast and final.", effects: &[E::Power(1.2), E::Speed(1.1)] },
    ItemDef { name: "Philosopher Crown", rarity: Rarity::Legendary, desc: "Rich in every way.", effects: &[E::Xp(1.5), E::Gold(1.5)] },
];

/// Chest rarity roll: legends are mythology (2%), commons are lunch (53%).
pub fn roll_item_rarity(seed: u64) -> Rarity {
    match (seed / 31) % 100 {
        98..=99 => Rarity::Legendary,
        93..=97 => Rarity::Epic,
        81..=92 => Rarity::Rare,
        56..=80 => Rarity::Magic,
        _ => Rarity::Common,
    }
}

/// Random item index of the given rarity (deterministic from seed).
pub fn random_item_of(rarity: Rarity, seed: u64) -> usize {
    let mut idxs = Vec::new();
    for (i, def) in ITEMS.iter().enumerate() {
        if def.rarity == rarity {
            idxs.push(i);
        }
    }
    if idxs.is_empty() {
        return 0;
    }
    idxs[(seed as usize) % idxs.len()]
}

/// Summed-up inventory power. Multipliers start at 1, adds at 0, best-wins
/// for homing (strongest telepathy rules) and magnet (widest charm wins).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemTotals {
    pub power_mult: f32,
    pub guard_mult: f32,
    pub speed_mult: f32,
    pub regen_add: f32,
    pub crit_add: f32,
    pub range_mult: f32,
    pub homing: f32,
    pub arrows_add: u32,
    pub lifesteal: f32,
    pub thorns: f32,
    pub magnet: f32,
    pub dodge_add: f32,
    pub potion_mult: f32,
    pub xp_mult: f32,
    pub shield_kill: f32,
    pub gold_mult: f32,
}

impl Default for ItemTotals {
    fn default() -> Self {
        Self {
            power_mult: 1.0,
            guard_mult: 1.0,
            speed_mult: 1.0,
            regen_add: 0.0,
            crit_add: 0.0,
            range_mult: 1.0,
            homing: 0.0,
            arrows_add: 0,
            lifesteal: 0.0,
            thorns: 0.0,
            magnet: 0.0,
            dodge_add: 0.0,
            potion_mult: 1.0,
            xp_mult: 1.0,
            shield_kill: 0.0,
            gold_mult: 1.0,
        }
    }
}

impl ItemTotals {
    pub fn of_inventory(inv: &[usize]) -> Self {
        let mut t = Self::default();
        for &i in inv {
            let Some(def) = ITEMS.get(i) else {
                continue;
            };
            for fx in def.effects.iter().copied() {
                match fx {
                    ItemEffect::Stat(_, _) => {}
                    ItemEffect::Power(m) => t.power_mult *= m,
                    ItemEffect::Guard(m) => t.guard_mult *= m,
                    ItemEffect::Speed(m) => t.speed_mult *= m,
                    ItemEffect::Regen(v) => t.regen_add += v,
                    ItemEffect::Crit(v) => t.crit_add += v,
                    ItemEffect::Range(m) => t.range_mult *= m,
                    ItemEffect::Homing(v) => t.homing = t.homing.max(v),
                    ItemEffect::Arrows(n) => t.arrows_add += n,
                    ItemEffect::Lifesteal(v) => t.lifesteal += v,
                    ItemEffect::Thorns(v) => t.thorns += v,
                    ItemEffect::Magnet(v) => t.magnet = t.magnet.max(v),
                    ItemEffect::Dodge(v) => t.dodge_add += v,
                    ItemEffect::Potion(m) => t.potion_mult *= m,
                    ItemEffect::Xp(m) => t.xp_mult *= m,
                    ItemEffect::ShieldKill(v) => t.shield_kill += v,
                    ItemEffect::Gold(m) => t.gold_mult *= m,
                }
            }
        }
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armory_shape() {
        assert!(ITEMS.len() >= 40, "want 40+ items, have {}", ITEMS.len());
        // every rarity must be reachable from chests
        for r in [Rarity::Common, Rarity::Magic, Rarity::Rare, Rarity::Epic, Rarity::Legendary] {
            assert!(ITEMS.iter().any(|d| d.rarity == r), "no {r:?} items");
            let idx = random_item_of(r, 7);
            assert_eq!(ITEMS[idx].rarity, r);
        }
        // exactly 25 plain stat buffs, names unique
        let stats = ITEMS.iter().filter(|d| d.effects.iter().any(|e| matches!(e, ItemEffect::Stat(_, _)))).count();
        assert_eq!(stats, 25, "want 25 stat items, have {stats}");
        let mut names = std::collections::HashSet::new();
        for d in ITEMS.iter() {
            assert!(names.insert(d.name), "duplicate item name: {}", d.name);
        }
    }
}
