//! Loot: rarities + the 50-weapon armory, each with its own slash-arc.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn multiplier(&self) -> f32 {
        match self {
            Rarity::Common => 1.0,
            Rarity::Uncommon => 1.15,
            Rarity::Rare => 1.35,
            Rarity::Epic => 1.6,
            Rarity::Legendary => 2.0,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Uncommon => "Uncommon",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
        }
    }
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Rarity::Common => (200, 200, 200),
            Rarity::Uncommon => (110, 210, 120),
            Rarity::Rare => (110, 170, 255),
            Rarity::Epic => (200, 120, 255),
            Rarity::Legendary => (255, 210, 90),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Weapon {
    pub name: String,
    pub rarity: Rarity,
    pub bonus_atk: f32,
    pub arch: WeaponArch,
}

impl Weapon {
    /// The starter stick-stats every run begins holding.
    pub fn rusty() -> Self {
        Self {
            name: "Rusty Blade".to_string(),
            rarity: Rarity::Common,
            bonus_atk: 0.0,
            arch: WeaponArch::Blade,
        }
    }
}

/// Weapon family: each swings a different arc (radius, spread, feel).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponArch {
    Blade,
    Bow,
    Staff,
    Dagger,
    Maul,
    Axe,
    Spear,
    Oddity,
}

impl WeaponArch {
    pub fn name(&self) -> &'static str {
        match self {
            WeaponArch::Blade => "Blade",
            WeaponArch::Bow => "Bow",
            WeaponArch::Staff => "Staff",
            WeaponArch::Dagger => "Dagger",
            WeaponArch::Maul => "Maul",
            WeaponArch::Axe => "Axe",
            WeaponArch::Spear => "Spear",
            WeaponArch::Oddity => "Oddity",
        }
    }

    /// Slash-arc hitbox profile: (radius, spread_deg, swoosh segs, pellets, pellet speed).
    /// Daggers stab quick and narrow; mauls sweep the screen; spears skewer far.
    pub fn arc(&self) -> (f32, f32, usize, usize, f32) {
        match self {
            WeaponArch::Blade => (2.2, 110.0, 6, 4, 12.0),
            WeaponArch::Bow => (1.8, 60.0, 4, 2, 11.0),
            WeaponArch::Staff => (2.3, 90.0, 5, 3, 11.0),
            WeaponArch::Dagger => (1.6, 55.0, 4, 3, 14.0),
            WeaponArch::Maul => (2.6, 150.0, 7, 5, 9.0),
            WeaponArch::Axe => (2.4, 130.0, 6, 4, 10.0),
            WeaponArch::Spear => (3.2, 28.0, 4, 3, 14.0),
            WeaponArch::Oddity => (2.0, 140.0, 6, 3, 10.0),
        }
    }

    /// Maps a name-table index to its family (8 blades, then 6 of each).
    pub fn of_index(i: usize) -> Self {
        match i {
            0..=7 => WeaponArch::Blade,
            8..=13 => WeaponArch::Bow,
            14..=19 => WeaponArch::Staff,
            20..=25 => WeaponArch::Dagger,
            26..=31 => WeaponArch::Maul,
            32..=37 => WeaponArch::Axe,
            38..=43 => WeaponArch::Spear,
            _ => WeaponArch::Oddity,
        }
    }

    /// Hold-attack pattern: every family shoots its basic differently.
    /// A bow splits your hold into twin shots; a maul lobs one huge slow
    /// slug; a dagger flurries fast and weak.
    pub fn hold(&self) -> HoldPattern {
        match self {
            WeaponArch::Blade => HoldPattern { count: 1, spread: 0.0, speed: 1.0, size: 1.0, dmg: 1.0, life: 1.0, cooldown: 1.0 },
            WeaponArch::Bow => HoldPattern { count: 2, spread: 0.16, speed: 1.0, size: 0.9, dmg: 0.85, life: 1.0, cooldown: 1.0 },
            WeaponArch::Staff => HoldPattern { count: 1, spread: 0.0, speed: 0.7, size: 1.8, dmg: 1.4, life: 1.2, cooldown: 1.25 },
            WeaponArch::Dagger => HoldPattern { count: 1, spread: 0.0, speed: 1.15, size: 0.85, dmg: 0.8, life: 0.9, cooldown: 0.7 },
            WeaponArch::Maul => HoldPattern { count: 1, spread: 0.0, speed: 0.6, size: 2.2, dmg: 1.6, life: 0.8, cooldown: 1.5 },
            WeaponArch::Axe => HoldPattern { count: 2, spread: 0.2, speed: 0.95, size: 1.1, dmg: 0.9, life: 1.0, cooldown: 1.1 },
            WeaponArch::Spear => HoldPattern { count: 1, spread: 0.0, speed: 1.4, size: 0.9, dmg: 1.0, life: 1.6, cooldown: 1.0 },
            WeaponArch::Oddity => HoldPattern { count: 3, spread: 0.35, speed: 0.9, size: 0.9, dmg: 0.6, life: 1.0, cooldown: 1.1 },
        }
    }
}

/// One hold-attack volley: arrow count, fan spread (radians), and
/// multipliers over the standard arrow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoldPattern {
    pub count: usize,
    pub spread: f32,
    pub speed: f32,
    pub size: f32,
    pub dmg: f32,
    pub life: f32,
    pub cooldown: f32,
}
