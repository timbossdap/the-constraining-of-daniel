//! Zones: names, palettes, level ranges, enemy tables.

use crate::enemy::EnemyDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Meadow,
    CinderCaves,
    FrostKeep,
}

impl Zone {
    pub fn all() -> [Zone; 3] {
        [Zone::Meadow, Zone::CinderCaves, Zone::FrostKeep]
    }
    pub fn name(&self) -> &'static str {
        match self {
            Zone::Meadow => "Sunlit Meadow",
            Zone::CinderCaves => "Cinder Caves",
            Zone::FrostKeep => "Frost Keep",
        }
    }
    pub fn level_range(&self) -> (u32, u32) {
        match self {
            Zone::Meadow => (1, 10),
            Zone::CinderCaves => (8, 25),
            Zone::FrostKeep => (20, 50),
        }
    }
    /// ground palette (two checker colors) + fog tint
    pub fn palette(&self) -> ((u8, u8, u8), (u8, u8, u8)) {
        match self {
            Zone::Meadow => ((70, 130, 80), (60, 115, 70)),
            Zone::CinderCaves => ((120, 60, 45), (95, 45, 38)),
            Zone::FrostKeep => ((140, 170, 200), (120, 150, 185)),
        }
    }
    pub fn for_level(level: u32) -> Zone {
        if level < 8 {
            Zone::Meadow
        } else if level < 20 {
            Zone::CinderCaves
        } else {
            Zone::FrostKeep
        }
    }
}

pub fn zone_enemies(zone: Zone) -> Vec<EnemyDef> {
    match zone {
        Zone::Meadow => vec![
            EnemyDef { name: "Gloom Slime", hp_mult: 0.8, atk_mult: 0.7, xp_mult: 1.0, speed: 1.2, color: (90, 200, 120), size: 0.7 , ranged: false },
            EnemyDef { name: "Thorn Wolf", hp_mult: 0.9, atk_mult: 1.0, xp_mult: 1.2, speed: 2.6, color: (160, 120, 80), size: 0.65 , ranged: false },
            EnemyDef { name: "Moss Brute", hp_mult: 1.6, atk_mult: 1.1, xp_mult: 1.8, speed: 1.0, color: (60, 140, 90), size: 1.0 , ranged: false },
            EnemyDef { name: "Spore Spitter", hp_mult: 0.7, atk_mult: 0.9, xp_mult: 1.5, speed: 1.0, color: (140, 180, 90), size: 0.6, ranged: true },
            EnemyDef { name: "Bramble Hog", hp_mult: 0.6, atk_mult: 1.1, xp_mult: 0.9, speed: 3.2, color: (120, 90, 60), size: 0.55, ranged: false },
        ],
        Zone::CinderCaves => vec![
            EnemyDef { name: "Ash Imp", hp_mult: 0.9, atk_mult: 1.2, xp_mult: 1.4, speed: 2.2, color: (255, 120, 60), size: 0.6 , ranged: false },
            EnemyDef { name: "Magma Golem", hp_mult: 2.2, atk_mult: 1.4, xp_mult: 2.4, speed: 0.9, color: (180, 60, 40), size: 1.2 , ranged: false },
            EnemyDef { name: "Cinder Bat", hp_mult: 0.7, atk_mult: 1.0, xp_mult: 1.3, speed: 3.0, color: (220, 100, 90), size: 0.5 , ranged: false },
            EnemyDef { name: "Ash Cultist", hp_mult: 0.8, atk_mult: 1.3, xp_mult: 1.7, speed: 1.4, color: (200, 80, 120), size: 0.6, ranged: true },
            EnemyDef { name: "Slag Hound", hp_mult: 0.7, atk_mult: 1.2, xp_mult: 1.1, speed: 3.6, color: (230, 120, 50), size: 0.5, ranged: false },
        ],
        Zone::FrostKeep => vec![
            EnemyDef { name: "Ice Revenant", hp_mult: 1.4, atk_mult: 1.5, xp_mult: 2.0, speed: 1.6, color: (150, 200, 255), size: 0.8 , ranged: false },
            EnemyDef { name: "Frost Wyrm", hp_mult: 2.6, atk_mult: 1.8, xp_mult: 3.2, speed: 1.4, color: (200, 230, 255), size: 1.3 , ranged: false },
            EnemyDef { name: "Snow Stalker", hp_mult: 1.0, atk_mult: 1.6, xp_mult: 1.9, speed: 2.8, color: (180, 190, 220), size: 0.65 , ranged: false },
            EnemyDef { name: "Shard Slinger", hp_mult: 0.9, atk_mult: 1.4, xp_mult: 1.8, speed: 1.2, color: (170, 220, 255), size: 0.6, ranged: true },
            EnemyDef { name: "Glacier Ooze", hp_mult: 3.0, atk_mult: 1.2, xp_mult: 2.2, speed: 0.6, color: (180, 220, 235), size: 1.4, ranged: false },
        ],
    }
}
