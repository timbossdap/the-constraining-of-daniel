//! Core attributes and derived stats. 100% engine-agnostic.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Attributes {
    pub strength: u32,
    pub agility: u32,
    pub intellect: u32,
    pub vitality: u32,
    pub luck: u32,
}

impl Attributes {
    pub fn new(str: u32, agi: u32, int: u32, vit: u32, luk: u32) -> Self {
        Self {
            strength: str,
            agility: agi,
            intellect: int,
            vitality: vit,
            luck: luk,
        }
    }

    pub fn total(&self) -> u32 {
        self.strength + self.agility + self.intellect + self.vitality + self.luck
    }
}

impl fmt::Display for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "STR {} | AGI {} | INT {} | VIT {} | LUK {}",
            self.strength, self.agility, self.intellect, self.vitality, self.luck
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DerivedStats {
    pub max_hp: f32,
    pub max_mp: f32,
    pub phys_atk: f32,
    pub magic_atk: f32,
    pub defense: f32,
    /// units per second
    pub move_speed: f32,
    /// 0.0 - 0.6
    pub crit_chance: f32,
    pub crit_mult: f32,
    /// 0.0 - 0.4
    pub dodge_chance: f32,
    pub hp_regen: f32,
    pub mp_regen: f32,
}

impl DerivedStats {
    pub fn from_attributes(base: &Attributes, level: u32) -> Self {
        let s = base.strength as f32;
        let a = base.agility as f32;
        let i = base.intellect as f32;
        let v = base.vitality as f32;
        let l = base.luck as f32;
        let lvl = level as f32;

        let max_hp = 20.0 + v * 6.0 + s * 1.0 + lvl * 4.0;
        let max_mp = 30.0 + i * 10.0 + v * 2.0 + lvl * 3.0;
        let phys_atk = 5.0 + s * 1.8 + a * 0.9 + l * 0.3 + lvl * 1.0;
        let magic_atk = 5.0 + i * 2.0 + a * 0.4 + l * 0.4 + lvl * 1.0;
        let defense = 1.5 + v * 1.1 + s * 0.4 + lvl * 0.5;
        let move_speed = (3.2 + a * 0.06).clamp(3.2, 6.5);
        let crit_chance = (0.05 + a * 0.004 + l * 0.008).clamp(0.05, 0.6);
        let crit_mult = 1.6 + l * 0.02;
        let dodge_chance = (a * 0.003 + l * 0.004).clamp(0.0, 0.4);
        let hp_regen = 0.5 + v * 0.12;
        let mp_regen = 0.6 + i * 0.12;

        Self {
            max_hp,
            max_mp,
            phys_atk,
            magic_atk,
            defense,
            move_speed,
            crit_chance,
            crit_mult,
            dodge_chance,
            hp_regen,
            mp_regen,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrKind {
    Strength,
    Agility,
    Intellect,
    Vitality,
    Luck,
}

impl AttrKind {
    pub fn all() -> [AttrKind; 5] {
        [
            AttrKind::Strength,
            AttrKind::Agility,
            AttrKind::Intellect,
            AttrKind::Vitality,
            AttrKind::Luck,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            AttrKind::Strength => "STR",
            AttrKind::Agility => "AGI",
            AttrKind::Intellect => "INT",
            AttrKind::Vitality => "VIT",
            AttrKind::Luck => "LUK",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AttrKind::Strength => "+Phys Atk, +HP, +Defense",
            AttrKind::Agility => "+Speed, +Crit, +Dodge",
            AttrKind::Intellect => "+Magic, +Mana, +Regen",
            AttrKind::Vitality => "+HP big, +Defense, +Regen",
            AttrKind::Luck => "+Crit dmg, +Crit, +Loot",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_scales() {
        let low = DerivedStats::from_attributes(&Attributes::new(5, 5, 5, 5, 5), 1);
        let high = DerivedStats::from_attributes(&Attributes::new(20, 20, 20, 20, 20), 20);
        assert!(high.max_hp > low.max_hp);
        assert!(high.phys_atk > low.phys_atk);
    }
}
