//! Headless game simulation: player, enemies, arrows, XP, loot.
//! No engine types here except KeyCode history — fully testable.

use crate::{
    combat::{apply_crit, player_attack_damage, roll_crit},
    draft::DraftCard,
    enemy::{Affix, Enemy},
    pickup::{Pickup, PickupKind},
    player_class::{CharacterClass, SkillKind},
    progression::PlayerCore,
    story::StoryState,
    util::dist,
    weapon_list::Weapon,
    zone::{Zone, zone_enemies},
};
use fyrox::keyboard::KeyCode;

#[derive(Debug, Clone)]
pub(crate) struct Projectile {
    pub(crate) pos: (f32, f32),
    pub(crate) vel: (f32, f32),
    pub(crate) life: f32,
    pub(crate) dmg: f32,
    pub(crate) friendly: bool,
    pub(crate) color: (u8, u8, u8),
    pub(crate) size: f32,
    pub(crate) burn: f32,
    pub(crate) slow: f32,
    pub(crate) lifesteal: f32,
    /// homing turn rate (rad/s); 0 = flies straight. Telepathy lives here.
    pub(crate) homing: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct FloatText {
    pub(crate) pos: (f32, f32),
    pub(crate) life: f32,
    #[allow(dead_code)]
    pub(crate) text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Particle {
    pub(crate) pos: (f32, f32),
    pub(crate) vel: (f32, f32),
    pub(crate) life: f32,
    pub(crate) max_life: f32,
    pub(crate) color: (u8, u8, u8),
    pub(crate) size: f32,
}

/// One segment of a blade-arc swoosh: a short rect placed on the arc and
/// rotated tangentially. Pure VFX — damage travels in fan pellets.
#[derive(Debug, Clone)]
pub(crate) struct ArcSeg {
    pub(crate) pos: (f32, f32),
    pub(crate) vel: (f32, f32),
    pub(crate) angle: f32,
    pub(crate) life: f32,
    pub(crate) max_life: f32,
    pub(crate) len: f32,
    pub(crate) thick: f32,
    pub(crate) color: (u8, u8, u8),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Duel {
    pub(crate) title: String,
    pub(crate) reward_stones: u32,
    pub(crate) reward_xp: u32,
    pub(crate) reward_rank: u8,
    pub(crate) started: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct GameInner {
    pub(crate) player: PlayerCore,
    pub(crate) player_pos: (f32, f32),
    pub(crate) facing: f32,
    pub(crate) enemies: Vec<Enemy>,
    pub(crate) pickups: Vec<Pickup>,
    pub(crate) projectiles: Vec<Projectile>,
    pub(crate) floats: Vec<FloatText>,
    pub(crate) particles: Vec<Particle>,
    pub(crate) zone: Zone,
    pub(crate) time: f32,
    pub(crate) seed: u64,
    pub(crate) spawn_timer: f32,
    pub(crate) boss_dead_timer: f32,
    pub(crate) boss_spawned_for_zone: bool,
    pub(crate) started: bool,
    /// story sparring duel: one elite, no spawns, victor takes the purse
    pub(crate) duel: Option<Duel>,
    /// named synergy ids already heralded (the feed fires once each)
    pub(crate) seen_syn: Vec<u16>,
    /// cultivation saga state (path, sect, flags, stamina, endings...)
    pub(crate) story: StoryState,
    /// chosen Gu path (None until path select); survives death, not rebirth
    pub(crate) path: Option<usize>,
    pub(crate) messages: Vec<String>,
    pub(crate) hud_timer: f32,
    pub(crate) attack_cd: f32,
    pub(crate) hit_flash: f32,
    pub(crate) warn_cd: f32,
    pub(crate) hitstop: f32,
    // level-up draft: game freezes while open, picks queue up
    pub(crate) draft_open: bool,
    pub(crate) draft_queue: u32,
    /// input lockout after a draft opens (Space-holders don't insta-pick)
    pub(crate) draft_lock: f32,
    pub(crate) draft_cards: Vec<DraftCard>,
    pub(crate) draft_sel: usize,
    pub(crate) draft_id: u32,
    pub(crate) dash_timer: f32,
    pub(crate) dash_cd: f32,
    pub(crate) dash_dir: (f32, f32),
    /// Held main-hand weapon (drives manual attacks) + the auto-fire battery
    /// of every previous main hand (HoloCure-style: new weapons ADD attacks).
    pub(crate) held: Weapon,
    pub(crate) loadout: Vec<Weapon>,
    pub(crate) auto_cd: Vec<f32>,
    pub(crate) swing_t: f32,
    pub(crate) swing_ang: f32,
    pub(crate) arc_fx: Vec<ArcSeg>,
    pub(crate) loot_history: Vec<String>,
    pub(crate) prev_keys: Vec<KeyCode>,
    pub(crate) kills_this_zone: u32,
    pub(crate) cam_shake: f32,
    pub(crate) cam_pos: (f32, f32),
    pub(crate) move_mag: f32,
    pub(crate) frame: u64,
}

impl Default for GameInner {
    fn default() -> Self {
        Self {
            player: PlayerCore::new(CharacterClass::Drifter),
            player_pos: (0.0, 0.0),
            facing: 1.0,
            enemies: Vec::new(),
            pickups: Vec::new(),
            projectiles: Vec::new(),
            floats: Vec::new(),
            particles: Vec::new(),
            zone: Zone::Meadow,
            time: 0.0,
            seed: 12345,
            spawn_timer: 0.0,
            boss_dead_timer: 0.0,
            boss_spawned_for_zone: false,
            started: false,
            duel: None,
            seen_syn: Vec::new(),
            story: StoryState::new(),
            path: None,
            messages: vec!["YOU = a vessel. Arrows/Space shoot. Z/X/C/V burn essence. Reach Lv 5 for a worm.".to_string()],
            hud_timer: 0.0,
            attack_cd: 0.0,
            hit_flash: 0.0,
            warn_cd: 0.0,
            hitstop: 0.0,
            draft_open: false,
            draft_queue: 0,
            draft_lock: 0.0,
            draft_cards: Vec::new(),
            draft_sel: 0,
            draft_id: 0,
            dash_timer: 0.0,
            dash_cd: 0.0,
            dash_dir: (1.0, 0.0),
            held: Weapon::rusty(),
            loadout: Vec::new(),
            auto_cd: Vec::new(),
            swing_t: 0.0,
            swing_ang: 0.0,
            arc_fx: Vec::new(),
            loot_history: Vec::new(),
            prev_keys: Vec::new(),
            kills_this_zone: 0,
            cam_shake: 0.0,
            cam_pos: (0.0, 0.0),
            move_mag: 0.0,
            frame: 0,
        }
    }
}

impl GameInner {

    pub(crate) fn next_seed(&mut self) -> u64 {
        // xorshift64 — no deps
        let mut x = self.seed;
        if x == 0 {
            x = 0x9E3779B97F4A7C15;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.seed = x;
        x
    }

    pub(crate) fn log(&mut self, s: String) {
        self.messages.push(s);
        if self.messages.len() > 6 {
            self.messages.remove(0);
        }
    }

    pub(crate) fn burst(&mut self, pos: (f32, f32), color: (u8, u8, u8), n: usize, speed: f32, life: f32, size: f32) {
        for _ in 0..n {
            if self.particles.len() > 220 {
                break;
            }
            let s = self.next_seed();
            let a = ((s % 628) as f32) / 100.0;
            let sp = speed * (0.4 + ((s / 7 % 100) as f32) / 100.0);
            self.particles.push(Particle {
                pos,
                vel: (a.cos() * sp, a.sin() * sp),
                life: life * (0.6 + ((s / 13 % 60) as f32) / 100.0),
                max_life: life,
                color,
                size: size * (0.7 + ((s % 50) as f32) / 100.0),
            });
        }
    }

    pub(crate)     fn tick_particles(&mut self, dt: f32) {
        for pt in self.particles.iter_mut() {
            pt.life -= dt;
            pt.pos.0 += pt.vel.0 * dt;
            pt.pos.1 += pt.vel.1 * dt;
            pt.vel.0 *= 1.0 - dt * 2.5;
            pt.vel.1 *= 1.0 - dt * 2.5;
        }
        self.particles.retain(|p| p.life > 0.0);
        // swoosh segments drift out and die fast
        for sg in self.arc_fx.iter_mut() {
            sg.life -= dt;
            sg.pos.0 += sg.vel.0 * dt;
            sg.pos.1 += sg.vel.1 * dt;
        }
        self.arc_fx.retain(|s| s.life > 0.0);
        if self.cam_shake > 0.0 {
            self.cam_shake = (self.cam_shake - dt * 2.2).max(0.0);
        }
    }

    pub(crate) fn ensure_zone(&mut self) {
        let want = Zone::for_level(self.player.level);
        if want != self.zone {
            self.zone = want;
            self.enemies.clear();
            self.projectiles.clear();
            self.boss_spawned_for_zone = false;
            self.kills_this_zone = 0;
            self.log(format!(">> Entering {} (Lv {}-{:?})!", want.name(), want.level_range().0, want.level_range().1));
            // zone-entry gift: fun, not grind
            self.player.potions = (self.player.potions + 2).min(9);
            let d = self.player.derived();
            self.player.hp = d.max_hp;
            self.player.essence = d.max_essence;
            // sprinkle pickups
            for i in 0..6 {
                let s = self.next_seed();
                let a = ((s % 628) as f32) / 100.0;
                self.pickups.push(Pickup {
                    pos: (a.cos() * (3.0 + i as f32), a.sin() * (3.0 + i as f32)),
                    kind: if i % 3 == 0 { PickupKind::Heart } else if i % 3 == 1 { PickupKind::Gold } else { PickupKind::Essence },
                    bob: 0.0,
                });
            }
        }
    }

    pub(crate) fn spawn_wave(&mut self) {
        // duels are single combat: no waves while the ring is set
        if self.duel.is_some() {
            return;
        }
        let defs = zone_enemies(self.zone);
        let target = match self.zone {
            Zone::Meadow => 6,
            Zone::CinderCaves => 9,
            Zone::FrostKeep => 12,
        };
        if self.enemies.len() >= target {
            return;
        }
        let n = (target - self.enemies.len()).min(3);
        for _ in 0..n {
            let s = self.next_seed();
            let def = &defs[(s as usize) % defs.len()];
            let ang = ((s / 13 % 628) as f32) / 100.0;
            let dist = 5.0 + ((s % 30) as f32) / 10.0;
            let mut pos = (self.player_pos.0 + ang.cos() * dist, self.player_pos.1 + ang.sin() * dist);
            pos.0 = pos.0.clamp(-13.0, 13.0);
            pos.1 = pos.1.clamp(-9.0, 9.0);
            // Boss every 5 levels, once per zone visit
            let is_boss_level = self.player.level % 5 == 0 && !self.boss_spawned_for_zone && self.kills_this_zone >= 8;
            let lvl = self.player.level;
            let e = Enemy::spawn(def, lvl, pos, s, is_boss_level);
            if is_boss_level {
                self.boss_spawned_for_zone = true;
                self.log(format!("!!! BOSS: {} appears! Slay it for loot!", e.name));
            }
            self.enemies.push(e);
        }
        // ambient pickups
        if self.pickups.len() < 5 && self.next_seed() % 100 < 20 {
            let s = self.seed;
            // ambient: hearts/essence/gold/bombs/shrines drift in, but never
            // chests — those only fall from kills, rarely
            let kinds = [PickupKind::Heart, PickupKind::Essence, PickupKind::Gold, PickupKind::Bomb, PickupKind::Shrine];
            self.pickups.push(Pickup {
                pos: (self.player_pos.0 + ((s % 17) as f32 - 8.0), self.player_pos.1 + ((s / 17 % 17) as f32 - 8.0)),
                kind: kinds[(s as usize) % kinds.len()],
                bob: 0.0,
            });
        }
    }

    pub(crate) fn try_cast(&mut self, idx: usize) -> bool {
        if idx > 3 {
            return false;
        }
        let skills = self.player.class.skills();
        let sk = &skills[idx];
        if self.player.level < sk.unlock_level {
            return false;
        }
        if self.player.skill_cooldowns[idx] > 0.0 {
            return false;
        }
        if (self.player.essence as u32) < sk.essence_cost {
            return false;
        }
        self.player.essence -= sk.essence_cost as f32;
        self.player.skill_cooldowns[idx] = sk.cooldown;
        let seed = self.next_seed();
        let (mut dmg, kind) = player_attack_damage(&self.player, idx, seed);
        dmg += self.held.bonus_atk * self.held.rarity.multiplier();
        if roll_crit(&self.player, seed / 3 + 1) {
            dmg = apply_crit(dmg, &self.player);
        }
        match kind {
            SkillKind::Melee | SkillKind::Backstab | SkillKind::Holy => {
                // weapon-arc swing, autotargeted in reach (arrows-only rule
                // holds: the fan of pellets IS the hitbox, plus the swoosh).
                let mut sdmg = dmg;
                if matches!(kind, SkillKind::Backstab) && self.player.combo_count >= 5 {
                    sdmg *= 1.3;
                }
                let ls = if matches!(kind, SkillKind::Holy) { 0.1 } else { 0.0 };
                let dir = self.melee_aim(self.swing_radius());
                self.melee_fan(dir, sdmg, ls);
            }
            SkillKind::Projectile | SkillKind::Burning { .. } | SkillKind::Chill { .. } | SkillKind::Lifesteal { .. } => {
                let (burn, slow, ls) = match kind {
                    SkillKind::Burning { dot } => (dot, 0.0, 0.0),
                    SkillKind::Chill { slow } => (0.0, slow, 0.0),
                    SkillKind::Lifesteal { fraction } => (0.0, 0.0, fraction),
                    _ => (0.0, 0.0, 0.0),
                };
                // aim at nearest enemy, else facing
                let dir = self.aim_dir();
                self.projectiles.push(Projectile {
                    pos: self.player_pos,
                    vel: (dir.0 * 9.0, dir.1 * 9.0),
                    life: 1.2 * self.player.item_totals().range_mult,
                    dmg,
                    friendly: true,
                    color: self.player.class.tint(),
                    size: 0.45,
                    burn,
                    slow,
                    lifesteal: ls,
                    homing: self.player.item_totals().homing,
                });
            }
            SkillKind::Multishot { count } => {
                let base = self.aim_dir();
                for i in 0..count {
                    let off = (i as f32 - count as f32 / 2.0) * 0.22;
                    let (dx, dy) = (base.0 * off.cos() - base.1 * off.sin(), base.0 * off.sin() + base.1 * off.cos());
                    self.projectiles.push(Projectile {
                        pos: self.player_pos,
                        vel: (dx * 10.0, dy * 10.0),
                        life: 0.9 * self.player.item_totals().range_mult,
                        dmg: dmg * 0.8,
                        friendly: true,
                        color: self.player.class.tint(),
                        size: 0.4,
                        burn: 0.0,
                        slow: 0.0,
                        lifesteal: 0.0,
                        homing: self.player.item_totals().homing,
                    });
                }
            }
            SkillKind::Whirlwind | SkillKind::Nova { .. } => {
                let slow = if let SkillKind::Nova { slow } = kind { slow } else { 0.0 };
                // expanding ring of live arrows — no instant radius damage
                for k in 0..8 {
                    let a = k as f32 * std::f32::consts::TAU / 8.0;
                    self.projectiles.push(Projectile {
                        pos: self.player_pos,
                        vel: (a.cos() * 7.0, a.sin() * 7.0),
                        life: 0.32 * self.player.item_totals().range_mult,
                        dmg: dmg * 0.7,
                        friendly: true,
                        color: self.player.class.tint(),
                        size: 0.45,
                        burn: 0.0,
                        slow,
                        lifesteal: 0.0,
                        homing: self.player.item_totals().homing,
                    });
                }
                if matches!(kind, SkillKind::Nova { .. }) {
                    self.player.hp = (self.player.hp + dmg * 0.1).min(self.player.derived().max_hp);
                }
            }
            SkillKind::Dash => {
                // gap-closer: dashes at the nearest enemy when there is one,
                // else along movement intent. Deals no damage itself.
                self.dash_timer = 0.22;
                self.dash_cd = 0.6;
                self.dash_dir = if self.enemies.is_empty() {
                    self.move_intent_dir()
                } else {
                    self.aim_dir()
                };
            }
            SkillKind::Shield { amount } => {
                let amt = amount + self.player.derived().max_hp * 0.08;
                self.player.shield_hp = amt;
                self.player.shield_timer = 5.0;
                self.log(format!("Shield +{}!", amt as u32));
            }
            SkillKind::Heal { fraction } => {
                let d = self.player.derived();
                self.player.hp = (self.player.hp + d.max_hp * fraction).min(d.max_hp);
                self.log("Healed!".to_string());
            }
            SkillKind::Barrage | SkillKind::Meteor | SkillKind::LeapSlam | SkillKind::Execute => {
                // aimed volley toward nearest enemy (or facing): every arrow must connect
                let (count, spread, speed, mult, burn) = match kind {
                    SkillKind::Barrage => (5, 0.12, 11.0, 0.9, 0.0),
                    SkillKind::Meteor => (4, 0.20, 9.0, 1.2, 12.0),
                    SkillKind::LeapSlam => (3, 0.30, 10.0, 1.1, 0.0),
                    SkillKind::Execute => (2, 0.08, 13.0, 1.0, 0.0),
                    _ => (3, 0.15, 10.0, 1.0, 0.0),
                };
                let mut vdmg = dmg * mult;
                if matches!(kind, SkillKind::Execute) {
                    // executioner bonus if the aimed target is already weak
                    let aim = self.aim_point(3.0);
                    for e in &self.enemies {
                        if dist(e.pos, aim) < 3.0 && e.hp < e.max_hp * 0.4 {
                            vdmg *= 1.8;
                            break;
                        }
                    }
                }
                let base = self.aim_dir();
                if base.0 != 0.0 {
                    self.facing = base.0.signum();
                }
                for i in 0..count {
                    let off = (i as f32 - count as f32 / 2.0) * spread;
                    let (dx, dy) = (base.0 * off.cos() - base.1 * off.sin(), base.0 * off.sin() + base.1 * off.cos());
                    self.projectiles.push(Projectile {
                        pos: self.player_pos,
                        vel: (dx * speed, dy * speed),
                        life: 1.0 * self.player.item_totals().range_mult,
                        dmg: vdmg,
                        friendly: true,
                        color: self.player.class.tint(),
                        size: 0.5,
                        burn,
                        slow: 0.0,
                        lifesteal: 0.0,
                        homing: self.player.item_totals().homing,
                    });
                }
                self.burst(self.player_pos, (255, 200, 100), 8, 4.0, 0.35, 0.2);
            }
        }
        true
    }

    /// Fires one arrow: travels, then damages ONLY on contact.
    /// This is the single source of player->enemy damage (plus skill volleys).
    /// Hold-attack: the equipped weapon family reshapes your basic shot.
    /// A bow splits it twin; a maul lobs one huge slow slug; oddities spray.
    /// Item quivers add arrows on top (5 total max); glasses stretch range.
    pub(crate) fn fire_arrow(&mut self, dir: (f32, f32)) {
        let seed = self.next_seed();
        let (mut dmg, _) = player_attack_damage(&self.player, 0, seed);
        dmg += self.held.bonus_atk * self.held.rarity.multiplier();
        if roll_crit(&self.player, seed / 3 + 1) {
            dmg = apply_crit(dmg, &self.player);
        }
        let pat = self.held.arch.hold();
        let tote = self.player.item_totals();
        let n = (pat.count + tote.arrows_add.min(4) as usize).min(5).max(1);
        let l = (dir.0 * dir.0 + dir.1 * dir.1).sqrt().max(0.001);
        let (dx, dy) = (dir.0 / l, dir.1 / l);
        let base = dy.atan2(dx);
        let hom = tote.homing;
        let rng = tote.range_mult;
        for i in 0..n {
            let off = if n == 1 {
                0.0
            } else {
                (i as f32 - (n - 1) as f32 * 0.5) * pat.spread / (n - 1).max(1) as f32
            };
            // item quivers widen the fan a touch past the weapon spread
            let off = off * (1.0 + tote.arrows_add.min(4) as f32 * 0.15);
            let a = base + off;
            let spd = 11.0 * pat.speed;
            self.projectiles.push(Projectile {
                pos: self.player_pos,
                vel: (a.cos() * spd, a.sin() * spd),
                life: 0.9 * pat.life * rng,
                dmg: dmg * pat.dmg,
                friendly: true,
                color: self.player.class.tint(),
                size: 0.45 * pat.size,
                burn: 0.0,
                slow: 0.0,
                lifesteal: 0.0,
                homing: hom,
            });
        }
        if dir.0 != 0.0 {
            self.facing = dir.0.signum();
        }
        // muzzle puff
        self.burst(self.player_pos, self.player.class.tint(), 2, 2.0, 0.2, 0.14);
    }

    /// Cooldown for hold-attacks (daggers flurry, mauls lumber).
    pub(crate) fn hold_cooldown(&self) -> f32 {
        let base = self.player.class.skills()[0].cooldown;
        base * self.held.arch.hold().cooldown
    }

    /// Melee autotarget: nearest enemy inside swing reach, else facing.
    /// Swing/fallback skills snap to targets so melee never whiffs blindly.
    /// New weapon picked up: the current main hand joins the auto-fire
    /// battery and the newcomer takes the hand. Attacks ADD, never replace.
    /// Returns the equipped name for logs.
    pub(crate) fn equip_weapon(&mut self, w: Weapon) -> String {
        let old = std::mem::replace(&mut self.held, w);
        self.loadout.push(old);
        self.auto_cd.push(0.3 + self.loadout.len() as f32 * 0.15);
        self.remember_loot(self.held.name.clone());
        self.held.name.clone()
    }

    /// Auto battery tick: every banked weapon fires its own hold pattern at
    /// the nearest enemy (HoloCure-style). Staggered cooldowns, no combo feed.
    pub(crate) fn tick_auto_weapons(&mut self, dt: f32) {
        for cd in self.auto_cd.iter_mut() {
            *cd = (*cd - dt).max(0.0);
        }
        if self.loadout.is_empty() || self.enemies.is_empty() {
            return;
        }
        let tote = self.player.item_totals();
        let power = self.player.class.passive().power;
        let tint = self.player.class.tint();
        let n = self.loadout.len();
        for i in 0..n {
            if i >= self.auto_cd.len() {
                break;
            }
            if self.auto_cd[i] > 0.0 {
                continue;
            }
            let w = self.loadout[i].clone();
            let pat = w.arch.hold();
            self.auto_cd[i] = 1.1 * pat.cooldown;
            let mut dmg = (4.0 + w.bonus_atk * w.rarity.multiplier() * 0.35) * power;
            let seed = self.next_seed();
            if roll_crit(&self.player, seed) {
                dmg = apply_crit(dmg, &self.player);
            }
            let base = self.aim_dir();
            let a0 = base.1.atan2(base.0);
            for k in 0..pat.count {
                let off = if pat.count == 1 {
                    0.0
                } else {
                    (k as f32 - (pat.count - 1) as f32 * 0.5) * pat.spread / (pat.count - 1).max(1) as f32
                };
                let a = a0 + off;
                let spd = 11.0 * pat.speed;
                self.projectiles.push(Projectile {
                    pos: self.player_pos,
                    vel: (a.cos() * spd, a.sin() * spd),
                    life: 0.9 * pat.life * tote.range_mult,
                    dmg: dmg * pat.dmg,
                    friendly: true,
                    color: tint,
                    size: 0.45 * pat.size,
                    burn: 0.0,
                    slow: 0.0,
                    lifesteal: 0.0,
                    homing: tote.homing,
                });
            }
        }
    }

    pub(crate) fn swing_radius(&self) -> f32 {
        let (r, _, _, _, _) = self.held.arch.arc();
        if matches!(self.player.class, CharacterClass::Drifter) {
            // the stick is stubby: short swing until you evolve
            r * 0.55
        } else {
            r
        }
    }

    pub(crate) fn melee_aim(&self, radius: f32) -> (f32, f32) {
        let mut best: Option<(f32, f32, f32)> = None;
        for e in &self.enemies {
            let dx = e.pos.0 - self.player_pos.0;
            let dy = e.pos.1 - self.player_pos.1;
            let d = (dx * dx + dy * dy).sqrt();
            if d < radius + 1.5 && (best.is_none() || d < best.unwrap().2) {
                let l = d.max(0.001);
                best = Some((dx / l, dy / l, d));
            }
        }
        best.map(|(x, y, _)| (x, y)).unwrap_or((self.facing, 0.0))
    }
    /// (arrows-only rule holds), plus the rotated-rect swoosh. Records swing
    /// angle/time so the held weapon anim can follow it.
    pub(crate) fn melee_fan(&mut self, dir: (f32, f32), dmg: f32, lifesteal: f32) {
        let l = (dir.0 * dir.0 + dir.1 * dir.1).sqrt().max(0.001);
        let (dx, dy) = (dir.0 / l, dir.1 / l);
        if dir.0 != 0.0 {
            self.facing = dir.0.signum();
        }
        let (_, spread_deg, segs, pellets, speed) = self.held.arch.arc();
        let radius = self.swing_radius();
        let base = dy.atan2(dx);
        self.swing_ang = base;
        let spread = spread_deg.to_radians();
        // damage fan: pellets spread across the arc, range = radius
        let n = pellets.max(1);
        for i in 0..n {
            let off = if n == 1 {
                0.0
            } else {
                (i as f32 - (n - 1) as f32 * 0.5) * spread / (n - 1) as f32
            };
            let a = base + off;
            self.projectiles.push(Projectile {
                pos: (self.player_pos.0 + a.cos() * 0.5, self.player_pos.1 + a.sin() * 0.5),
                vel: (a.cos() * speed, a.sin() * speed),
                life: (radius / speed + 0.03) * self.player.item_totals().range_mult,
                dmg,
                friendly: true,
                color: self.player.class.tint(),
                size: 0.5,
                burn: 0.0,
                slow: 0.0,
                lifesteal,
                homing: self.player.item_totals().homing,
            });
        }
        // swoosh VFX: segments along the arc, rotated tangentially
        let tint = self.player.class.tint();
        for k in 0..segs {
            let a = if segs == 1 {
                base
            } else {
                base - spread * 0.5 + spread * k as f32 / (segs - 1) as f32
            };
            let r = radius * 0.85;
            let chunk = 2.0 * std::f32::consts::PI * r * (spread / (2.0 * std::f32::consts::PI)) / segs as f32;
            self.arc_fx.push(ArcSeg {
                pos: (self.player_pos.0 + a.cos() * r, self.player_pos.1 + a.sin() * r),
                vel: (a.cos() * 2.0, a.sin() * 2.0),
                angle: a + std::f32::consts::FRAC_PI_2,
                life: 0.16,
                max_life: 0.16,
                len: (chunk * 1.4).max(0.3),
                thick: 0.15,
                color: if k % 2 == 0 { tint } else { (255, 255, 255) },
            });
        }
        if self.arc_fx.len() > 28 {
            let cut = self.arc_fx.len() - 28;
            self.arc_fx.drain(..cut);
        }
        self.swing_t = 0.22;
        self.burst(self.player_pos, tint, 4, 4.0, 0.25, 0.18);
    }

    /// Basic attack for sword-line classes: class skill-0 damage as a swing
    /// in the given direction (twin-stick melee).
    pub(crate) fn swing_basic(&mut self, dir: (f32, f32)) {
        let seed = self.next_seed();
        let (mut dmg, _) = player_attack_damage(&self.player, 0, seed);
        dmg += self.held.bonus_atk * self.held.rarity.multiplier();
        if roll_crit(&self.player, seed / 3 + 1) {
            dmg = apply_crit(dmg, &self.player);
        }
        self.melee_fan(dir, dmg, 0.0);
    }

    pub(crate) fn aim_dir(&self) -> (f32, f32) {
        let mut best: Option<(f32, f32, f32)> = None;
        for e in &self.enemies {
            let dx = e.pos.0 - self.player_pos.0;
            let dy = e.pos.1 - self.player_pos.1;
            let d = (dx * dx + dy * dy).sqrt();
            if d < 8.0 && (best.is_none() || d < best.unwrap().2) {
                let l = d.max(0.001);
                best = Some((dx / l, dy / l, d));
            }
        }
        if let Some((x, y, _)) = best {
            (x, y)
        } else {
            (self.facing, 0.0)
        }
    }

    pub(crate) fn aim_point(&self, ahead: f32) -> (f32, f32) {
        let d = self.aim_dir();
        (self.player_pos.0 + d.0 * ahead, self.player_pos.1 + d.1 * ahead)
    }

    pub(crate) fn move_intent_dir(&self) -> (f32, f32) {
        (self.facing, 0.0)
    }

    /// Routes fresh level-ups: milestone levels (5/10/15/20/25/30) open
    /// Gu-choice boons (rank 2 at 5 up to rank 6 at 25+), everything else
    /// queues a normal upgrade draft. No classes — worms are the build.
    pub(crate) fn handle_level_ups(&mut self, ups: u32) {
        if ups == 0 {
            return;
        }
        let lvl = self.player.level;
        let mut boon: Option<u8> = None;
        for crossed in lvl.saturating_sub(ups) + 1..=lvl {
            if crossed % 5 == 0 && (5..=30).contains(&crossed) {
                boon = Some(((crossed / 5) + 1).min(6) as u8);
            }
        }
        if let Some(rank) = boon {
            self.draft_queue += ups.saturating_sub(1);
            self.open_gu_boon(rank);
        } else {
            self.draft_queue += ups;
            self.open_draft_if_needed();
        }
    }
    /// Remembers a named pickup for the ITEMS panel (keeps the last 3).
    pub(crate) fn remember_loot(&mut self, name: String) {
        self.loot_history.push(name);
        while self.loot_history.len() > 3 {
            self.loot_history.remove(0);
        }
    }

    pub(crate) fn kill_enemy(&mut self, idx: usize) {
        if idx >= self.enemies.len() {
            return;
        }
        let e = self.enemies.remove(idx);
        self.player.kills += 1;
        self.kills_this_zone += 1;
        self.player.gold += ((e.gold as f32) * self.player.item_totals().gold_mult) as u32;
        // kills distill primeval essence back into the aperture
        {
            let dmax = self.player.derived().max_essence;
            self.player.essence = (self.player.essence + 3.0).min(dmax);
        }
        // aegis-style items bank a shield on every kill
        let sk = self.player.item_totals().shield_kill;
        if sk > 0.0 {
            self.player.shield_hp += sk;
            self.player.shield_timer = self.player.shield_timer.max(5.0);
        }
        // vampiric classes drink a little of every kill
        let kh = self.player.class.passive().kill_heal;
        if kh > 0.0 {
            let dmax = self.player.derived().max_hp;
            self.player.hp = (self.player.hp + dmax * kh).min(dmax);
        }
        // death poof + shake
        self.burst(e.pos, e.color, if e.boss { 28 } else if e.elite { 16 } else { 10 }, if e.boss { 7.0 } else { 4.5 }, 0.6, if e.boss { 0.32 } else { 0.24 });
        self.burst(e.pos, (255, 255, 255), 4, 3.0, 0.3, 0.18);
        self.cam_shake = (self.cam_shake + if e.boss { 1.0 } else if e.elite { 0.45 } else { 0.18 }).min(1.2);
        // dramatic pause on every kill: longer for bosses/elites
        self.hitstop = if e.boss { 0.25 } else if e.elite { 0.09 } else { 0.045 };
        if e.boss {
            self.boss_dead_timer = 5.0;
            self.log(format!("BOSS SLAIN! +{} xp +{}g", e.xp, e.gold));
            self.burst(e.pos, (255, 200, 100), 20, 6.0, 0.9, 0.3);
        }
        let ups = self.player.add_xp(e.xp);
        if ups > 0 {
            self.handle_level_ups(ups);
        }
        // visible XP reward rising off the corpse — the levelling feed
        self.floats.push(FloatText {
            pos: e.pos,
            life: 0.9,
            text: format!("+{} XP", e.xp),
        });
        if ups > 0 {
            let d = self.player.derived();
            self.log(format!(
                "LEVEL UP! Lv {} ({} pts — Z/X/C/V/B spend, T auto) HP {:.0}/{:.0}",
                self.player.level, self.player.unspent_points, self.player.hp, d.max_hp
            ));
            // level heal is 35% max — show it
            self.floats.push(FloatText {
                pos: e.pos,
                life: 1.1,
                text: format!("+{:.0}", d.max_hp * 0.35),
            });
        }
        // drops: weapons/items NEVER drop directly — very rarely (5%) a
        // chest falls instead, and chests hold gear only ~10% of the time.
        // (Hearts/gold still drop small and often to keep runs alive.)
        let s = self.next_seed();
        if s % 100 < 5 {
            self.pickups.push(Pickup { pos: e.pos, kind: PickupKind::Chest, bob: 0.0 });
        } else if s % 100 < 25 {
            self.pickups.push(Pickup { pos: e.pos, kind: if s % 2 == 0 { PickupKind::Gold } else { PickupKind::Heart }, bob: 0.0 });
        }
        if e.affix == Affix::Explosive {
            // boom hurts other enemies — fun!
            let mut dead2 = Vec::new();
            for (j, o) in self.enemies.iter_mut().enumerate() {
                if dist(o.pos, e.pos) < 2.5 {
                    if o.take_damage(e.max_hp * 0.4) {
                        dead2.push(j);
                    }
                }
            }
            for j in dead2.into_iter().rev() {
                if j != idx {
                    self.kill_enemy(j);
                    break; // avoid recursion blowup; one chain is enough
                }
            }
        }
        self.player.register_hit();
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weapon_list::{Rarity, WeaponArch};

    fn test_weapon(name: &str) -> Weapon {
        Weapon {
            name: name.to_string(),
            rarity: Rarity::Rare,
            bonus_atk: 10.0,
            arch: WeaponArch::Bow,
        }
    }

    #[test]
    fn weapons_accumulate_dont_replace() {
        let mut g = GameInner::default();
        assert_eq!(g.held.name, "Rusty Blade");
        assert!(g.loadout.is_empty());
        g.equip_weapon(test_weapon("Stormcaller"));
        assert_eq!(g.held.name, "Stormcaller");
        assert_eq!(g.loadout.len(), 1);
        assert_eq!(g.loadout[0].name, "Rusty Blade");
        assert_eq!(g.auto_cd.len(), 1);
        g.equip_weapon(test_weapon("Doombringer"));
        assert_eq!(g.held.name, "Doombringer");
        assert_eq!(g.loadout.len(), 2);
        // battery ticks cooldowns even with no enemies, fires nothing
        g.tick_auto_weapons(0.5);
        assert!(g.projectiles.is_empty());
    }

    #[test]
    fn milestone_opens_worm_boon() {
        use crate::draft::DraftKind;
        use crate::gu::GU;
        let mut g = GameInner::default();
        g.path = Some(7); // Blood
        g.player.level = 5;
        g.handle_level_ups(1);
        assert!(g.draft_open, "milestone must open a draft");
        assert_eq!(g.draft_cards.len(), 3);
        for card in g.draft_cards.iter() {
            match card.kind {
                DraftKind::GuWorm(idx) => {
                    let d = &GU[idx];
                    assert_eq!(d.rank, 2, "Lv 5 boon is rank 2");
                    assert_eq!(d.path, 7, "boon favors your path");
                }
                _ => panic!("milestone cards must all be worms"),
            }
        }
        let n_inv = g.player.inventory.len();
        g.apply_draft(0);
        assert!(!g.draft_open);
        assert!(g.player.inventory.len() > n_inv || g.loadout.len() == 1);
    }

    #[test]
    fn plain_level_opens_upgrade_draft() {
        let mut g = GameInner::default();
        g.player.level = 6;
        g.handle_level_ups(1);
        assert!(g.draft_open);
        assert_eq!(g.draft_cards.len(), 3);
    }
}
