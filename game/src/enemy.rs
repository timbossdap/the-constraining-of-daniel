//! Enemies: affixes, definitions, spawn scaling, damage over time.

use crate::zone::Zone;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Affix {
    None,
    Swift,
    Burning,
    Vampiric,
    Shielded,
    Explosive,
}

impl Affix {
    pub fn name(&self) -> &'static str {
        match self {
            Affix::None => "",
            Affix::Swift => "Swift",
            Affix::Burning => "Burning",
            Affix::Vampiric => "Vampiric",
            Affix::Shielded => "Shielded",
            Affix::Explosive => "Explosive",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnemyDef {
    pub name: &'static str,
    pub hp_mult: f32,
    pub atk_mult: f32,
    pub xp_mult: f32,
    pub speed: f32,
    pub color: (u8, u8, u8),
    pub size: f32,
    pub ranged: bool,
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub name: String,
    pub level: u32,
    pub hp: f32,
    pub max_hp: f32,
    pub atk: f32,
    pub xp: u32,
    pub gold: u32,
    pub speed: f32,
    pub color: (u8, u8, u8),
    pub size: f32,
    pub affix: Affix,
    pub elite: bool,
    pub boss: bool,
    pub pos: (f32, f32),
    pub burn_ticks: f32,
    pub burn_dps: f32,
    pub slow_timer: f32,
    pub flash: f32,
    pub touch_cd: f32,
    pub shooter: bool,
    pub shot_cd: f32,
}

impl Enemy {
    pub fn spawn(def: &EnemyDef, level: u32, pos: (f32, f32), seed: u64, force_boss: bool) -> Self {
        let lvl = level as f32;
        // progressive curves: gentle early, brutal late (quadratic kicks in)
        let base_hp = 18.0 + lvl * 7.0 + lvl * lvl * 0.25;
        let base_atk = 4.0 + lvl * 1.3 + lvl * lvl * 0.04;
        // stage multiplier: stage 1 stays easy, later stages hit much harder
        // (and pay more XP to match)
        let zone = Zone::for_level(level);
        let zm = match zone {
            Zone::Meadow => 1.0,
            Zone::CinderCaves => 1.25,
            Zone::FrostKeep => 1.6,
        };
        // elites get meaner in later stages
        let elite_at = match zone {
            Zone::Meadow => 90,
            Zone::CinderCaves => 84,
            Zone::FrostKeep => 78,
        };
        let elite_roll = seed % 100;
        let elite = force_boss || elite_roll > elite_at;
        let affix = if force_boss {
            Affix::Shielded
        } else if !elite {
            Affix::None
        } else {
            match seed % 5 {
                0 => Affix::Swift,
                1 => Affix::Burning,
                2 => Affix::Vampiric,
                3 => Affix::Shielded,
                _ => Affix::Explosive,
            }
        };
        let mut hp_m = def.hp_mult;
        let mut atk_m = def.atk_mult;
        let mut xp_m = def.xp_mult;
        if elite && !force_boss {
            hp_m *= 2.2;
            atk_m *= 1.25;
            xp_m *= 3.0;
        }
        if force_boss {
            hp_m *= 6.0;
            atk_m *= 1.5;
            xp_m *= 10.0;
        }
        // affix tweaks
        let speed = def.speed * if affix == Affix::Swift { 1.6 } else { 1.0 };
        if affix == Affix::Shielded {
            hp_m *= 1.4;
        }
        let max_hp = base_hp * hp_m * zm;
        let name = if force_boss {
            format!("BOSS: Ancient {} of {:?}", def.name, Zone::for_level(level))
        } else if elite {
            format!("{} {}", affix.name(), def.name)
        } else {
            def.name.to_string()
        };
        Self {
            name,
            level,
            hp: max_hp,
            max_hp,
            atk: base_atk * atk_m * zm,
            xp: ((8.0 + level as f32 * 6.0) * xp_m * zm) as u32,
            gold: (3 + level + (seed % 8) as u32) * if elite { 3 } else { 1 },
            speed,
            color: def.color,
            size: def.size * if force_boss { 1.8 } else if elite { 1.3 } else { 1.0 },
            affix,
            elite,
            boss: force_boss,
            pos,
            burn_ticks: 0.0,
            burn_dps: 0.0,
            slow_timer: 0.0,
            flash: 0.0,
            touch_cd: 0.0,
            shooter: def.ranged,
            shot_cd: 1.5,
        }
    }

    pub fn take_damage(&mut self, dmg: f32) -> bool {
        self.hp -= dmg;
        self.flash = 0.12;
        self.hp <= 0.0
    }

    pub fn tick_dot(&mut self, dt: f32) -> bool {
        if self.burn_ticks > 0.0 {
            self.burn_ticks -= dt;
            self.hp -= self.burn_dps * dt;
            self.flash = self.flash.max(0.05);
        }
        if self.slow_timer > 0.0 {
            self.slow_timer -= dt;
        }
        if self.flash > 0.0 {
            self.flash -= dt;
        }
        self.hp <= 0.0
    }

    pub fn effective_speed(&self) -> f32 {
        if self.slow_timer > 0.0 {
            self.speed * 0.45
        } else {
            self.speed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_def() -> EnemyDef {
        EnemyDef {
            name: "T",
            hp_mult: 1.0,
            atk_mult: 1.0,
            xp_mult: 1.0,
            speed: 1.0,
            color: (0, 0, 0),
            size: 1.0,
            ranged: false,
        }
    }

    #[test]
    fn difficulty_rises_by_stage() {
        // seed 7 -> elite_roll 7, never elite: pure curve check
        let early = Enemy::spawn(&dummy_def(), 1, (0.0, 0.0), 7, false);
        let mid = Enemy::spawn(&dummy_def(), 10, (0.0, 0.0), 7, false);
        let late = Enemy::spawn(&dummy_def(), 25, (0.0, 0.0), 7, false);
        assert!(mid.max_hp > early.max_hp * 2.0);
        assert!(late.max_hp > mid.max_hp * 2.0);
        assert!(mid.atk > early.atk);
        assert!(late.atk > mid.atk);
        assert!(late.xp >= mid.xp);
        assert!(mid.xp >= early.xp);
    }

    #[test]
    fn shooters_come_armed() {
        let mut d = dummy_def();
        d.ranged = true;
        let e = Enemy::spawn(&d, 5, (0.0, 0.0), 7, false);
        assert!(e.shooter);
        assert!(e.shot_cd > 0.0);
    }
}
