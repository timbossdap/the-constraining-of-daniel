//! 8 playable classes + skills. Fun-first design: every class gets
//! mobility + AoE early so it never feels like grind.

use crate::stats::Attributes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterClass {
    Knight,
    Ranger,
    Pyromancer,
    Frostwarden,
    Rogue,
    Cleric,
    Necromancer,
    Spellblade,
    // Tier 0: the only starting class. Everyone begins a nobody.
    Drifter,
    // Tier 2 evolutions (two branches per base class).
    Crusader,
    Doomblade,
    Deadeye,
    Nightstalker,
    Inferno,
    Ashcaller,
    Glacial,
    Stormcaller,
    Assassin,
    Trickster,
    Saint,
    Inquisitor,
    Lich,
    Plaguebearer,
    Runelord,
    Hexblade,
}

impl CharacterClass {
    pub fn all() -> [CharacterClass; 8] {
        [
            CharacterClass::Knight,
            CharacterClass::Ranger,
            CharacterClass::Pyromancer,
            CharacterClass::Frostwarden,
            CharacterClass::Rogue,
            CharacterClass::Cleric,
            CharacterClass::Necromancer,
            CharacterClass::Spellblade,
        ]
    }

    pub fn index(&self) -> usize {
        match self {
            CharacterClass::Knight => 0,
            CharacterClass::Ranger => 1,
            CharacterClass::Pyromancer => 2,
            CharacterClass::Frostwarden => 3,
            CharacterClass::Rogue => 4,
            CharacterClass::Cleric => 5,
            CharacterClass::Necromancer => 6,
            CharacterClass::Spellblade => 7,
            CharacterClass::Drifter => 8,
            CharacterClass::Crusader => 9,
            CharacterClass::Doomblade => 10,
            CharacterClass::Deadeye => 11,
            CharacterClass::Nightstalker => 12,
            CharacterClass::Inferno => 13,
            CharacterClass::Ashcaller => 14,
            CharacterClass::Glacial => 15,
            CharacterClass::Stormcaller => 16,
            CharacterClass::Assassin => 17,
            CharacterClass::Trickster => 18,
            CharacterClass::Saint => 19,
            CharacterClass::Inquisitor => 20,
            CharacterClass::Lich => 21,
            CharacterClass::Plaguebearer => 22,
            CharacterClass::Runelord => 23,
            CharacterClass::Hexblade => 24,
        }
    }

    /// Evolution tier: 0 = starter, 1 = first class, 2 = evolved branch.
    pub fn tier(&self) -> u8 {
        match self {
            CharacterClass::Drifter => 0,
            CharacterClass::Knight
            | CharacterClass::Ranger
            | CharacterClass::Pyromancer
            | CharacterClass::Frostwarden
            | CharacterClass::Rogue
            | CharacterClass::Cleric
            | CharacterClass::Necromancer
            | CharacterClass::Spellblade => 1,
            _ => 2,
        }
    }

    /// The two evolution branches of a tier-1 class. Only meaningful on tier 1.
    pub fn evolutions(&self) -> [CharacterClass; 2] {
        match self {
            CharacterClass::Knight => [CharacterClass::Crusader, CharacterClass::Doomblade],
            CharacterClass::Ranger => [CharacterClass::Deadeye, CharacterClass::Nightstalker],
            CharacterClass::Pyromancer => [CharacterClass::Inferno, CharacterClass::Ashcaller],
            CharacterClass::Frostwarden => [CharacterClass::Glacial, CharacterClass::Stormcaller],
            CharacterClass::Rogue => [CharacterClass::Assassin, CharacterClass::Trickster],
            CharacterClass::Cleric => [CharacterClass::Saint, CharacterClass::Inquisitor],
            CharacterClass::Necromancer => [CharacterClass::Lich, CharacterClass::Plaguebearer],
            CharacterClass::Spellblade => [CharacterClass::Runelord, CharacterClass::Hexblade],
            _ => [CharacterClass::Drifter, CharacterClass::Drifter],
        }
    }

    /// Sword-line classes swing their weapon on arrow keys / Space instead of
    /// firing ranged arrows: the stick, swords, daggers, and the saint's mace.
    /// Everyone else shoots.
    pub fn swings(&self) -> bool {
        matches!(
            self,
            CharacterClass::Drifter
                | CharacterClass::Knight
                | CharacterClass::Crusader
                | CharacterClass::Doomblade
                | CharacterClass::Rogue
                | CharacterClass::Assassin
                | CharacterClass::Hexblade
                | CharacterClass::Saint
        )
    }

    /// Mechanical identity: every class bends the core numbers its own way.
    /// power = damage dealt mult, guard = damage taken mult (lower = tankier).
    pub fn passive(&self) -> ClassPassive {
        // (power, guard, speed, regen, potion, combo_window, kill_heal, burn_bonus)
        let p = match self {
            CharacterClass::Drifter => (1.00, 1.00, 1.00, 1.00, 1.00, 3.00, 0.00, 0.0),
            CharacterClass::Knight => (0.95, 0.80, 0.95, 1.20, 1.00, 3.00, 0.00, 0.0),
            CharacterClass::Ranger => (1.00, 1.05, 1.15, 1.00, 1.00, 3.50, 0.00, 0.0),
            CharacterClass::Pyromancer => (1.15, 1.10, 1.00, 0.90, 1.00, 3.00, 0.00, 1.5),
            CharacterClass::Frostwarden => (0.95, 0.85, 0.95, 1.10, 1.00, 3.00, 0.00, 0.0),
            CharacterClass::Rogue => (1.05, 1.05, 1.10, 1.00, 1.00, 5.00, 0.00, 0.0),
            CharacterClass::Cleric => (0.90, 0.90, 1.00, 1.50, 1.50, 3.00, 0.00, 0.0),
            CharacterClass::Necromancer => (1.05, 1.00, 1.00, 1.30, 1.00, 3.00, 0.02, 0.0),
            CharacterClass::Spellblade => (1.10, 1.00, 1.05, 1.00, 1.00, 3.50, 0.00, 0.0),
            CharacterClass::Crusader => (0.90, 0.70, 0.95, 1.40, 1.25, 3.00, 0.00, 0.0),
            CharacterClass::Doomblade => (1.25, 1.10, 1.05, 1.00, 1.00, 4.00, 0.01, 0.0),
            CharacterClass::Deadeye => (1.15, 1.05, 1.15, 1.00, 1.00, 4.00, 0.00, 0.0),
            CharacterClass::Nightstalker => (1.15, 1.10, 1.10, 1.00, 1.00, 5.00, 0.01, 0.0),
            CharacterClass::Inferno => (1.30, 1.15, 1.00, 0.85, 1.00, 3.00, 0.00, 2.0),
            CharacterClass::Ashcaller => (1.10, 1.05, 1.10, 1.00, 1.00, 3.00, 0.00, 1.0),
            CharacterClass::Glacial => (0.85, 0.65, 0.95, 1.20, 1.25, 3.00, 0.00, 0.0),
            CharacterClass::Stormcaller => (1.20, 1.05, 1.05, 1.00, 1.00, 3.00, 0.00, 0.0),
            CharacterClass::Assassin => (1.30, 1.10, 1.10, 1.00, 1.00, 5.00, 0.02, 0.0),
            CharacterClass::Trickster => (1.00, 1.05, 1.20, 1.10, 1.00, 5.00, 0.00, 0.0),
            CharacterClass::Saint => (0.85, 0.85, 1.00, 1.80, 1.50, 3.00, 0.00, 0.0),
            CharacterClass::Inquisitor => (1.20, 1.00, 1.00, 1.00, 1.00, 3.00, 0.01, 0.0),
            CharacterClass::Lich => (1.15, 1.00, 1.00, 1.50, 1.00, 3.00, 0.03, 0.0),
            CharacterClass::Plaguebearer => (1.15, 1.00, 1.00, 1.20, 1.00, 3.00, 0.01, 1.0),
            CharacterClass::Runelord => (1.20, 1.00, 1.05, 1.00, 1.00, 3.50, 0.00, 0.0),
            CharacterClass::Hexblade => (1.20, 1.05, 1.05, 1.00, 1.00, 4.00, 0.01, 0.0),
        };
        ClassPassive {
            power: p.0,
            guard: p.1,
            speed: p.2,
            regen: p.3,
            potion: p.4,
            combo_window: p.5,
            kill_heal: p.6,
            burn_bonus: p.7,
        }
    }

    pub fn from_index(i: usize) -> Self {
        Self::all()[i % 8]
    }

    pub fn name(&self) -> &'static str {
        match self {
            CharacterClass::Knight => "Knight",
            CharacterClass::Ranger => "Ranger",
            CharacterClass::Pyromancer => "Pyromancer",
            CharacterClass::Frostwarden => "Frostwarden",
            CharacterClass::Rogue => "Rogue",
            CharacterClass::Cleric => "Cleric",
            CharacterClass::Necromancer => "Necromancer",
            CharacterClass::Spellblade => "Spellblade",
            CharacterClass::Drifter => "Drifter",
            CharacterClass::Crusader => "Crusader",
            CharacterClass::Doomblade => "Doomblade",
            CharacterClass::Deadeye => "Deadeye",
            CharacterClass::Nightstalker => "Nightstalker",
            CharacterClass::Inferno => "Inferno",
            CharacterClass::Ashcaller => "Ashcaller",
            CharacterClass::Glacial => "Glacial",
            CharacterClass::Stormcaller => "Stormcaller",
            CharacterClass::Assassin => "Assassin",
            CharacterClass::Trickster => "Trickster",
            CharacterClass::Saint => "Saint",
            CharacterClass::Inquisitor => "Inquisitor",
            CharacterClass::Lich => "Lich",
            CharacterClass::Plaguebearer => "Plaguebearer",
            CharacterClass::Runelord => "Runelord",
            CharacterClass::Hexblade => "Hexblade",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            CharacterClass::Knight => "Bulwark of Dawn",
            CharacterClass::Ranger => "Windrunner",
            CharacterClass::Pyromancer => "Cinder Heart",
            CharacterClass::Frostwarden => "Glacier Oath",
            CharacterClass::Rogue => "Silent Edge",
            CharacterClass::Cleric => "Radiant Vow",
            CharacterClass::Necromancer => "Grave Whisperer",
            CharacterClass::Spellblade => "Rune Dancer",
            CharacterClass::Drifter => "Nobody Yet",
            CharacterClass::Crusader => "Dawn's Wall",
            CharacterClass::Doomblade => "Edge of Ruin",
            CharacterClass::Deadeye => "Never Misses",
            CharacterClass::Nightstalker => "Dark Between Stars",
            CharacterClass::Inferno => "Living Wildfire",
            CharacterClass::Ashcaller => "Gray Tempest",
            CharacterClass::Glacial => "Still As Death",
            CharacterClass::Stormcaller => "Voice of Thunder",
            CharacterClass::Assassin => "Final Argument",
            CharacterClass::Trickster => "Loaded Dice",
            CharacterClass::Saint => "Mercy Itself",
            CharacterClass::Inquisitor => "No Forgiveness",
            CharacterClass::Lich => "Death, Ongoing",
            CharacterClass::Plaguebearer => "Sick Season",
            CharacterClass::Runelord => "Written in Fire",
            CharacterClass::Hexblade => "Cursed Edge",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CharacterClass::Knight => "Tanky melee. Huge HP, shield block, whirlwind. Forgiving and fun.",
            CharacterClass::Ranger => "Fast ranged. Multi-shot, dash, crit storms. Kite everything.",
            CharacterClass::Pyromancer => "Burn it all. Fireball, immolate DoT, meteor. Big numbers.",
            CharacterClass::Frostwarden => "Control mage. Slows, ice nova, tanky with mana shield.",
            CharacterClass::Rogue => "Combo assassin. Backstab x3, shadowstep, dodge-tank.",
            CharacterClass::Cleric => "Holy brawler. Heal, smite AoE, never dies. Great first class.",
            CharacterClass::Necromancer => "Lifesteal caster. Siphon, corpse burst, out-sustain crowds.",
            CharacterClass::Spellblade => "Hybrid melee/magic. Blink strike, runes, highest skill ceiling.",
            CharacterClass::Drifter => "A nobody with a stick. Reach level 5.",
            CharacterClass::Crusader => "Holy tank. Outlast everything.",
            CharacterClass::Doomblade => "Bleed them dry. Execute the weak.",
            CharacterClass::Deadeye => "Arrows for days. Crit storms.",
            CharacterClass::Nightstalker => "Strike unseen. Vanish.",
            CharacterClass::Inferno => "Everything burns.",
            CharacterClass::Ashcaller => "Choking storms, searing wind.",
            CharacterClass::Glacial => "Unmoving. Unbreakable. Cold.",
            CharacterClass::Stormcaller => "The sky answers. Violently.",
            CharacterClass::Assassin => "One cut. Silence.",
            CharacterClass::Trickster => "Fast hands, faster exits.",
            CharacterClass::Saint => "Heal through anything.",
            CharacterClass::Inquisitor => "Smite first. No questions.",
            CharacterClass::Lich => "Your HP is my HP.",
            CharacterClass::Plaguebearer => "Share the sickness.",
            CharacterClass::Runelord => "Every rune a detonation.",
            CharacterClass::Hexblade => "Cut now, rot later.",
        }
    }

    /// (r,g,b) 0-255 for 2D sprite tint — gives "decent graphics" variety.
    pub fn tint(&self) -> (u8, u8, u8) {
        match self {
            CharacterClass::Knight => (90, 140, 255),
            CharacterClass::Ranger => (120, 220, 120),
            CharacterClass::Pyromancer => (255, 110, 40),
            CharacterClass::Frostwarden => (140, 220, 255),
            CharacterClass::Rogue => (180, 120, 255),
            CharacterClass::Cleric => (255, 235, 150),
            CharacterClass::Necromancer => (120, 255, 180),
            CharacterClass::Spellblade => (255, 120, 200),
            CharacterClass::Drifter => (170, 160, 140),
            CharacterClass::Crusader => (255, 230, 160),
            CharacterClass::Doomblade => (180, 40, 60),
            CharacterClass::Deadeye => (160, 255, 160),
            CharacterClass::Nightstalker => (90, 80, 160),
            CharacterClass::Inferno => (255, 90, 20),
            CharacterClass::Ashcaller => (200, 120, 80),
            CharacterClass::Glacial => (180, 240, 255),
            CharacterClass::Stormcaller => (120, 180, 255),
            CharacterClass::Assassin => (220, 60, 120),
            CharacterClass::Trickster => (140, 255, 200),
            CharacterClass::Saint => (255, 250, 210),
            CharacterClass::Inquisitor => (255, 180, 100),
            CharacterClass::Lich => (150, 255, 220),
            CharacterClass::Plaguebearer => (140, 220, 120),
            CharacterClass::Runelord => (255, 150, 230),
            CharacterClass::Hexblade => (200, 100, 255),
        }
    }

    /// Everyone starts with base nothing — power comes from levels, drafts,
    /// and class evolutions at checkpoints, not from the starting sheet.
    pub fn base_attributes(&self) -> Attributes {
        Attributes::new(1, 1, 1, 1, 1)
    }

    /// Per-level auto growth — keeps leveling exciting without mandatory grinding.
    pub fn growth_per_level(&self) -> Attributes {
        match self {
            CharacterClass::Knight => Attributes::new(2, 1, 0, 2, 0),
            CharacterClass::Ranger => Attributes::new(1, 2, 0, 1, 1),
            CharacterClass::Pyromancer => Attributes::new(0, 1, 3, 1, 0),
            CharacterClass::Frostwarden => Attributes::new(0, 1, 2, 2, 0),
            CharacterClass::Rogue => Attributes::new(1, 2, 0, 1, 1),
            CharacterClass::Cleric => Attributes::new(1, 0, 2, 2, 0),
            CharacterClass::Necromancer => Attributes::new(0, 1, 2, 1, 1),
            CharacterClass::Spellblade => Attributes::new(1, 1, 2, 1, 0),
            CharacterClass::Drifter => Attributes::new(1, 1, 1, 1, 0),
            CharacterClass::Crusader => Attributes::new(2, 1, 1, 3, 0),
            CharacterClass::Doomblade => Attributes::new(3, 1, 0, 2, 1),
            CharacterClass::Deadeye => Attributes::new(1, 3, 0, 1, 1),
            CharacterClass::Nightstalker => Attributes::new(2, 2, 0, 1, 1),
            CharacterClass::Inferno => Attributes::new(0, 1, 4, 1, 0),
            CharacterClass::Ashcaller => Attributes::new(1, 2, 2, 1, 0),
            CharacterClass::Glacial => Attributes::new(0, 1, 3, 3, 0),
            CharacterClass::Stormcaller => Attributes::new(0, 2, 3, 1, 0),
            CharacterClass::Assassin => Attributes::new(2, 3, 0, 1, 1),
            CharacterClass::Trickster => Attributes::new(1, 3, 1, 1, 1),
            CharacterClass::Saint => Attributes::new(1, 0, 3, 3, 0),
            CharacterClass::Inquisitor => Attributes::new(2, 0, 3, 2, 0),
            CharacterClass::Lich => Attributes::new(0, 1, 3, 2, 1),
            CharacterClass::Plaguebearer => Attributes::new(1, 1, 3, 1, 1),
            CharacterClass::Runelord => Attributes::new(1, 2, 3, 1, 0),
            CharacterClass::Hexblade => Attributes::new(2, 2, 2, 1, 0),
        }
    }

    pub fn attack_range(&self) -> f32 {
        match self {
            CharacterClass::Ranger | CharacterClass::Deadeye => 5.5,
            CharacterClass::Pyromancer | CharacterClass::Inferno => 5.0,
            CharacterClass::Frostwarden | CharacterClass::Glacial => 4.5,
            CharacterClass::Necromancer
            | CharacterClass::Lich
            | CharacterClass::Plaguebearer
            | CharacterClass::Stormcaller
            | CharacterClass::Ashcaller
            | CharacterClass::Runelord => 4.5,
            CharacterClass::Cleric | CharacterClass::Saint | CharacterClass::Inquisitor => 3.2,
            _ => 1.9,
        }
    }

    pub fn is_ranged(&self) -> bool {
        matches!(
            self,
            CharacterClass::Ranger
                | CharacterClass::Pyromancer
                | CharacterClass::Frostwarden
                | CharacterClass::Necromancer
                | CharacterClass::Deadeye
                | CharacterClass::Inferno
                | CharacterClass::Ashcaller
                | CharacterClass::Glacial
                | CharacterClass::Stormcaller
                | CharacterClass::Lich
                | CharacterClass::Plaguebearer
                | CharacterClass::Runelord
        )
    }

    pub fn skills(&self) -> [SkillDef; 4] {
        match self {
            CharacterClass::Knight => [
                SkillDef::basic("Valiant Slash", "Reliable arc slash.", 0, 0.45, 1.2, SkillKind::Melee),
                SkillDef::new("Bulwark", "70% dmg shield 4s.", 15, 10.0, 0.0, 2, SkillKind::Shield { amount: 25.0 }),
                SkillDef::new("Whirlwind", "360 spin, hits all near.", 20, 6.0, 1.8, 6, SkillKind::Whirlwind),
                SkillDef::new("Judgement", "Leap slam, big AoE.", 30, 12.0, 3.2, 12, SkillKind::LeapSlam),
            ],
            CharacterClass::Ranger => [
                SkillDef::basic("Quick Shot", "Fast piercing arrow.", 0, 0.4, 1.1, SkillKind::Projectile),
                SkillDef::new("Heavy Shot", "One heavy arrow.", 12, 5.0, 0.9, 3, SkillKind::Multishot { count: 1 }),
                SkillDef::new("Gale Dash", "Dash + brief speed boost.", 10, 6.0, 0.0, 5, SkillKind::Dash),
                SkillDef::new("Storm Volley", "Rain of arrows, huge AoE.", 28, 14.0, 2.6, 14, SkillKind::Barrage),
            ],
            CharacterClass::Pyromancer => [
                SkillDef::basic("Ember Bolt", "Fire bolt + burn.", 5, 0.55, 1.3, SkillKind::Burning { dot: 6.0 }),
                SkillDef::new("Immolate", "Burn all nearby.", 18, 7.0, 1.4, 4, SkillKind::Burning { dot: 12.0 }),
                SkillDef::new("Flame Dash", "Dash leaving fire.", 12, 7.0, 1.0, 8, SkillKind::Dash),
                SkillDef::new("Meteor", "Sky doom, massive AoE burn.", 35, 16.0, 3.8, 16, SkillKind::Meteor),
            ],
            CharacterClass::Frostwarden => [
                SkillDef::basic("Ice Shard", "Chill + slow.", 4, 0.5, 1.1, SkillKind::Chill { slow: 0.45 }),
                SkillDef::new("Glacial Armor", "Mana shield 5s.", 15, 12.0, 0.0, 4, SkillKind::Shield { amount: 30.0 }),
                SkillDef::new("Frost Nova", "Freeze everything close.", 22, 9.0, 1.7, 9, SkillKind::Nova { slow: 0.6 }),
                SkillDef::new("Blizzard", "Zone storm, slows + dmg.", 32, 15.0, 2.8, 15, SkillKind::Barrage),
            ],
            CharacterClass::Rogue => [
                SkillDef::basic("Stab", "Fast, high crit.", 0, 0.32, 1.0, SkillKind::Backstab),
                SkillDef::new("Shadowstep", "Blink behind + guaranteed crit.", 12, 6.0, 1.6, 3, SkillKind::Dash),
                SkillDef::new("Fan of Knives", "Hit all around.", 16, 7.0, 1.5, 7, SkillKind::Whirlwind),
                SkillDef::new("Assassinate", "500% if target <40% HP.", 25, 10.0, 3.0, 13, SkillKind::Execute),
            ],
            CharacterClass::Cleric => [
                SkillDef::basic("Smite", "Holy bolt, small heal.", 4, 0.5, 1.1, SkillKind::Holy),
                SkillDef::new("Radiant Mend", "Heal 40% max HP.", 20, 9.0, 0.0, 4, SkillKind::Heal { fraction: 0.25 }),
                SkillDef::new("Consecrate", "Burning holy ground.", 18, 10.0, 1.6, 8, SkillKind::Nova { slow: 0.2 }),
                SkillDef::new("Wrath", "Screen-wide holy nova.", 30, 14.0, 3.0, 14, SkillKind::Nova { slow: 0.3 }),
            ],
            CharacterClass::Necromancer => [
                SkillDef::basic("Siphon", "Dmg + heal you.", 5, 0.55, 1.1, SkillKind::Lifesteal { fraction: 0.3 }),
                SkillDef::new("Grave Chill", "AoE + slow + lifesteal.", 16, 8.0, 1.4, 5, SkillKind::Lifesteal { fraction: 0.25 }),
                SkillDef::new("Corpse Burst", "Detonate: bonus vs wounded.", 20, 9.0, 2.0, 10, SkillKind::Execute),
                SkillDef::new("Soul Harvest", "Big AoE + big heal.", 32, 16.0, 3.2, 16, SkillKind::Lifesteal { fraction: 0.4 }),
            ],
            CharacterClass::Spellblade => [
                SkillDef::basic("Rune Edge", "Melee + magic hybrid.", 3, 0.42, 1.25, SkillKind::Melee),
                SkillDef::new("Blink Strike", "Dash + shock.", 14, 6.0, 1.8, 5, SkillKind::Dash),
                SkillDef::new("Rune Storm", "Spinning magic blades.", 20, 8.0, 1.9, 9, SkillKind::Whirlwind),
                SkillDef::new("Starfall Blade", "Leap + meteor blades.", 30, 13.0, 3.4, 15, SkillKind::Meteor),
            ],
            // Tier 0 starter: a stick, a rock, fast feet, deep breaths.
            // Only the stick swing works at first — the rest are ??? until
            // you pick your first real class at level 5.
            CharacterClass::Drifter => [
                SkillDef::basic("Stick Swing", "A stick. It works.", 0, 0.5, 0.8, SkillKind::Melee),
                SkillDef::new("???", "Pick a class at Lv 5.", 0, 1.0, 0.0, 99, SkillKind::Projectile),
                SkillDef::new("???", "Pick a class at Lv 5.", 0, 1.0, 0.0, 99, SkillKind::Dash),
                SkillDef::new("???", "Pick a class at Lv 5.", 0, 1.0, 0.0, 99, SkillKind::Heal { fraction: 0.0 }),
            ],
            // Tier 2 evolutions: sharper kits, all usable the moment you evolve.
            CharacterClass::Crusader => [
                SkillDef::basic("Dawn Slash", "Consecrated arc.", 4, 0.45, 1.5, SkillKind::Melee),
                SkillDef::new("Sacred Bulwark", "Big holy shield.", 18, 11.0, 0.0, 1, SkillKind::Shield { amount: 30.0 }),
                SkillDef::new("Consecrated Spin", "Radiant 360.", 22, 7.0, 2.2, 4, SkillKind::Whirlwind),
                SkillDef::new("Divine Judgement", "The sky falls.", 34, 13.0, 3.8, 8, SkillKind::LeapSlam),
            ],
            CharacterClass::Doomblade => [
                SkillDef::basic("Cruel Edge", "Bleeds arrogance.", 3, 0.4, 1.7, SkillKind::Melee),
                SkillDef::new("Shadowstep", "Behind you.", 12, 6.0, 1.6, 1, SkillKind::Dash),
                SkillDef::new("Hemorrhage", "Bonus vs wounded.", 20, 9.0, 3.2, 4, SkillKind::Execute),
                SkillDef::new("Blade Vortex", "Shred everything near.", 24, 8.0, 2.0, 8, SkillKind::Whirlwind),
            ],
            CharacterClass::Deadeye => [
                SkillDef::basic("Piercing Shot", "Through rows.", 3, 0.38, 1.4, SkillKind::Projectile),
                SkillDef::new("Bullseye", "One perfect shot.", 14, 5.0, 1.0, 1, SkillKind::Multishot { count: 1 }),
                SkillDef::new("Windrunner", "Untouchable dash.", 10, 6.0, 0.0, 3, SkillKind::Dash),
                SkillDef::new("Arrow Storm", "The sky goes dark.", 30, 14.0, 3.0, 8, SkillKind::Barrage),
            ],
            CharacterClass::Nightstalker => [
                SkillDef::basic("Silent Shot", "You never hear it.", 3, 0.4, 1.3, SkillKind::Backstab),
                SkillDef::new("Smoke Dash", "Gone.", 10, 5.0, 0.0, 1, SkillKind::Dash),
                SkillDef::new("Caltrops", "Sharp ground, slow feet.", 16, 8.0, 1.6, 3, SkillKind::Nova { slow: 0.5 }),
                SkillDef::new("Nightfall", "Execute from dark.", 24, 10.0, 2.8, 8, SkillKind::Execute),
            ],
            CharacterClass::Inferno => [
                SkillDef::basic("Fireball", "Classic. Hot.", 5, 0.5, 1.5, SkillKind::Burning { dot: 8.0 }),
                SkillDef::new("Immolate+", "Everything nearby burns.", 20, 7.0, 1.6, 1, SkillKind::Burning { dot: 16.0 }),
                SkillDef::new("Flame Dash", "Dash leaving fire.", 12, 7.0, 1.0, 3, SkillKind::Dash),
                SkillDef::new("Cataclysm", "Erase the zone.", 38, 16.0, 4.4, 8, SkillKind::Meteor),
            ],
            CharacterClass::Ashcaller => [
                SkillDef::basic("Cinder Bolt", "Hot ash, fast.", 4, 0.45, 1.5, SkillKind::Projectile),
                SkillDef::new("Ash Cyclone", "Choking spin.", 20, 8.0, 2.0, 1, SkillKind::Whirlwind),
                SkillDef::new("Ash Cloud", "Blind + slow.", 16, 9.0, 1.5, 3, SkillKind::Nova { slow: 0.4 }),
                SkillDef::new("Firestorm", "Wind-driven barrage.", 30, 14.0, 3.0, 8, SkillKind::Barrage),
            ],
            CharacterClass::Glacial => [
                SkillDef::basic("Shard", "Deep chill.", 4, 0.48, 1.3, SkillKind::Chill { slow: 0.55 }),
                SkillDef::new("Ice Fortress", "Near-immortal 5s.", 18, 12.0, 0.0, 1, SkillKind::Shield { amount: 35.0 }),
                SkillDef::new("Deep Freeze", "Statues everywhere.", 24, 9.0, 2.0, 4, SkillKind::Nova { slow: 0.7 }),
                SkillDef::new("Avalanche", "The mountain moves.", 32, 13.0, 3.2, 8, SkillKind::LeapSlam),
            ],
            CharacterClass::Stormcaller => [
                SkillDef::basic("Spark", "Fast lightning.", 4, 0.42, 1.4, SkillKind::Projectile),
                SkillDef::new("Thunder Nova", "Deafening ring.", 20, 8.0, 2.2, 1, SkillKind::Nova { slow: 0.4 }),
                SkillDef::new("Gale Dash", "Ride the wind.", 10, 6.0, 0.0, 3, SkillKind::Dash),
                SkillDef::new("Tempest", "Storm answers.", 32, 14.0, 3.0, 8, SkillKind::Barrage),
            ],
            CharacterClass::Assassin => [
                SkillDef::basic("Throat Cut", "Quiet work.", 3, 0.3, 1.6, SkillKind::Backstab),
                SkillDef::new("Vanish", "Never there.", 12, 5.0, 1.4, 1, SkillKind::Dash),
                SkillDef::new("Death Mark", "Marked: deleted.", 22, 9.0, 3.6, 4, SkillKind::Execute),
                SkillDef::new("Killing Spree", "Everyone. Now.", 26, 10.0, 1.8, 8, SkillKind::Whirlwind),
            ],
            CharacterClass::Trickster => [
                SkillDef::basic("Sharp Wit", "Words hurt. Cards hurt more.", 3, 0.36, 1.3, SkillKind::Projectile),
                SkillDef::new("Blink", "Cheating, basically.", 10, 5.0, 1.2, 1, SkillKind::Dash),
                SkillDef::new("Card Fan", "A single ace.", 16, 6.0, 1.1, 3, SkillKind::Multishot { count: 1 }),
                SkillDef::new("Dirty Trick", "Pocket sand, arcane.", 18, 8.0, 1.6, 8, SkillKind::Nova { slow: 0.5 }),
            ],
            CharacterClass::Saint => [
                SkillDef::basic("Blessed Mace", "Mercy, swung.", 4, 0.48, 1.3, SkillKind::Holy),
                SkillDef::new("Greater Mend", "Heal 55% max HP.", 24, 10.0, 0.0, 1, SkillKind::Heal { fraction: 0.3 }),
                SkillDef::new("Sanctuary", "Safe ground.", 20, 12.0, 0.0, 4, SkillKind::Shield { amount: 30.0 }),
                SkillDef::new("Ascension", "Rise above it all.", 34, 14.0, 3.2, 8, SkillKind::Nova { slow: 0.3 }),
            ],
            CharacterClass::Inquisitor => [
                SkillDef::basic("Condemn", "Guilty.", 4, 0.46, 1.6, SkillKind::Holy),
                SkillDef::new("Purge", "Burn the doubt.", 18, 8.0, 1.5, 1, SkillKind::Burning { dot: 10.0 }),
                SkillDef::new("Sentence", "Carried out.", 22, 10.0, 3.0, 4, SkillKind::Execute),
                SkillDef::new("Wrathfall", "No survivors.", 36, 15.0, 3.6, 8, SkillKind::Meteor),
            ],
            CharacterClass::Lich => [
                SkillDef::basic("Drain", "Yours. Mine now.", 5, 0.5, 1.3, SkillKind::Lifesteal { fraction: 0.35 }),
                SkillDef::new("Death Chill", "Cold siphon.", 16, 8.0, 1.5, 1, SkillKind::Lifesteal { fraction: 0.3 }),
                SkillDef::new("Phylactery", "Can't kill what's stored.", 20, 13.0, 0.0, 4, SkillKind::Shield { amount: 30.0 }),
                SkillDef::new("Soul Tithe", "Everyone pays.", 34, 16.0, 3.4, 8, SkillKind::Lifesteal { fraction: 0.5 }),
            ],
            CharacterClass::Plaguebearer => [
                SkillDef::basic("Virulent Touch", "Just a scratch.", 4, 0.5, 1.2, SkillKind::Lifesteal { fraction: 0.3 }),
                SkillDef::new("Miasma", "Breathe deep.", 18, 8.0, 1.4, 1, SkillKind::Burning { dot: 10.0 }),
                SkillDef::new("Outbreak", "It spreads.", 20, 9.0, 2.0, 4, SkillKind::Nova { slow: 0.3 }),
                SkillDef::new("Pandemic", "No quarantine.", 32, 15.0, 2.8, 8, SkillKind::Barrage),
            ],
            CharacterClass::Runelord => [
                SkillDef::basic("Rune Bolt", "Punctuation.", 4, 0.4, 1.5, SkillKind::Projectile),
                SkillDef::new("Glyph Dash", "Written in motion.", 12, 6.0, 1.6, 1, SkillKind::Dash),
                SkillDef::new("Rune Tempest", "A storm of meaning.", 22, 8.0, 2.2, 4, SkillKind::Whirlwind),
                SkillDef::new("Starfall", "The sky is text.", 32, 14.0, 3.8, 8, SkillKind::Meteor),
            ],
            CharacterClass::Hexblade => [
                SkillDef::basic("Hex Edge", "Itches later.", 3, 0.4, 1.6, SkillKind::Melee),
                SkillDef::new("Siphon Blade", "Drink deep.", 14, 7.0, 1.5, 1, SkillKind::Lifesteal { fraction: 0.35 }),
                SkillDef::new("Witch Step", "Between heartbeats.", 12, 6.0, 1.4, 3, SkillKind::Dash),
                SkillDef::new("Doom Brand", "Marked for rot.", 24, 10.0, 3.2, 8, SkillKind::Execute),
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClassPassive {
    /// damage dealt multiplier
    pub power: f32,
    /// damage taken multiplier (lower = tankier)
    pub guard: f32,
    /// move speed multiplier
    pub speed: f32,
    /// HP regen multiplier
    pub regen: f32,
    /// potion strength multiplier
    pub potion: f32,
    /// seconds a hit refreshes the combo timer for
    pub combo_window: f32,
    /// fraction of max HP restored on kill
    pub kill_heal: f32,
    /// extra burn seconds applied by your ignites
    pub burn_bonus: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkillKind {
    Melee,
    Projectile,
    Multishot { count: u32 },
    Whirlwind,
    Dash,
    Shield { amount: f32 },
    Heal { fraction: f32 },
    Burning { dot: f32 },
    Chill { slow: f32 },
    Nova { slow: f32 },
    Barrage,
    Meteor,
    LeapSlam,
    Backstab,
    Execute,
    Lifesteal { fraction: f32 },
    Holy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkillDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub mana_cost: u32,
    pub cooldown: f32,
    pub power: f32,
    pub unlock_level: u32,
    pub kind: SkillKind,
}

impl SkillDef {
    pub fn basic(
        name: &'static str,
        desc: &'static str,
        mana: u32,
        cd: f32,
        power: f32,
        kind: SkillKind,
    ) -> Self {
        Self {
            name,
            desc,
            mana_cost: mana,
            cooldown: cd,
            power,
            unlock_level: 1,
            kind,
        }
    }
    pub fn new(
        name: &'static str,
        desc: &'static str,
        mana: u32,
        cd: f32,
        power: f32,
        unlock: u32,
        kind: SkillKind,
    ) -> Self {
        Self {
            name,
            desc,
            mana_cost: mana,
            cooldown: cd,
            power,
            unlock_level: unlock,
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_shape() {
        // one starter, eight first-classes, sixteen evolutions
        assert_eq!(CharacterClass::Drifter.tier(), 0);
        assert_eq!(CharacterClass::Drifter.evolutions().len(), 2);
        for i in 0..8 {
            let c = CharacterClass::from_index(i);
            assert_eq!(c.tier(), 1, "{:?} should be tier 1", c);
            let ev = c.evolutions();
            assert!(ev[0] != ev[1]);
            for e in ev {
                assert_eq!(e.tier(), 2, "{:?} should be tier 2", e);
                assert_eq!(e.skills().len(), 4);
                let p = e.passive();
                assert!(p.power > 0.0 && p.guard > 0.0 && p.speed > 0.0);
            }
        }
        // evolutions never offer the Drifter and never repeat a base
        let mut seen = std::collections::HashSet::new();
        for i in 0..8 {
            for e in CharacterClass::from_index(i).evolutions() {
                assert!(seen.insert(e.index()), "duplicate evolution {e:?}");
            }
        }
        assert_eq!(seen.len(), 16);
    }
}
