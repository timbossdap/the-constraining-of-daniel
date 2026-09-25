//! Named Gu synergies: exact single-path worm sets (2-5 worms) that grant
//! unique bonuses when every member sits in the inventory. Data lives in
//! `sy_a/b/c/d.rs` (30 per path, canon combos); this file holds the engine:
//! detection, folding into [`crate::gu::ItemTotals`], and activation feed.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::gu::{ItemTotals, GU};

/// Mechanical bonus bundle for one synergy. Mults default 1, adds 0.
#[derive(Debug, Clone, Copy)]
pub struct Fx {
    pub power: f32,
    pub guard: f32,
    pub crit: f32,
    pub lifesteal: f32,
    pub regen: f32,
    pub dodge: f32,
    pub speed: f32,
    pub xp: f32,
    pub gold: f32,
    pub thorns: f32,
    pub magnet: f32,
    pub homing: f32,
    pub arrows: u32,
    pub shieldkill: f32,
    pub potion: f32,
    pub range: f32,
    pub max_hp: f32,
    pub essence_max: f32,
    pub essence_regen: f32,
    pub revive: bool,
}

impl Fx {
    pub const NONE: Fx = Fx {
        power: 1.0,
        guard: 1.0,
        crit: 0.0,
        lifesteal: 0.0,
        regen: 0.0,
        dodge: 0.0,
        speed: 1.0,
        xp: 1.0,
        gold: 1.0,
        thorns: 0.0,
        magnet: 0.0,
        homing: 0.0,
        arrows: 0,
        shieldkill: 0.0,
        potion: 1.0,
        range: 1.0,
        max_hp: 1.0,
        essence_max: 0.0,
        essence_regen: 0.0,
        revive: false,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct SynergyDef {
    pub id: u16,
    pub path: u8,
    pub name: &'static str,
    pub desc: &'static str,
    pub members: &'static [&'static str],
    pub fx: Fx,
}

macro_rules! syn {
    ($id:expr, $path:expr, $name:expr, $desc:expr, $members:expr, $fx:expr) => {
        SynergyDef { id: $id, path: $path, name: $name, desc: $desc, members: $members, fx: $fx }
    };
}

mod a;
mod b;
mod c;
mod d;

static PARTS: [&[SynergyDef]; 4] = [a::SY_A, b::SY_B, c::SY_C, d::SY_D];

/// Canon Gu name -> table index (built once; warns None for typos via test).
static NAME_IDX: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();

pub fn name_index() -> &'static HashMap<&'static str, usize> {
    NAME_IDX.get_or_init(|| {
        let mut m = HashMap::new();
        for (i, def) in GU.iter().enumerate() {
            m.insert(def.name, i);
        }
        m
    })
}

/// Member table indices for one synergy (skips unknown names defensively).
fn member_idxs(s: &SynergyDef) -> Vec<usize> {
    let map = name_index();
    s.members.iter().filter_map(|n| map.get(n).copied()).collect()
}

/// All synergies whose full member set sits in `inv`.
pub fn active_synergies(inv: &[usize]) -> Vec<&'static SynergyDef> {
    let have: HashSet<usize> = inv.iter().copied().collect();
    let mut out = Vec::new();
    for part in PARTS.iter() {
        for s in part.iter() {
            let mem = member_idxs(s);
            if !mem.is_empty() && mem.len() == s.members.len() && mem.iter().all(|i| have.contains(i)) {
                out.push(s);
            }
        }
    }
    out
}

/// Fold every active synergy into totals (mults multiply, adds add).
pub fn apply_synergies(t: &mut ItemTotals, inv: &[usize]) {
    for s in active_synergies(inv) {
        let f = s.fx;
        t.power_mult *= f.power;
        t.guard_mult *= f.guard;
        t.crit_add += f.crit;
        t.lifesteal += f.lifesteal;
        t.regen_add += f.regen;
        t.dodge_add += f.dodge;
        t.speed_mult *= f.speed;
        t.xp_mult *= f.xp;
        t.gold_mult *= f.gold;
        t.thorns += f.thorns;
        t.magnet = t.magnet.max(f.magnet);
        t.homing = t.homing.max(f.homing);
        t.arrows_add += f.arrows;
        t.shield_kill += f.shieldkill;
        t.potion_mult *= f.potion;
        t.range_mult *= f.range;
        t.max_hp_mult *= f.max_hp;
        t.essence_max_add += f.essence_max;
        t.essence_regen_add += f.essence_regen;
        t.revive = t.revive || f.revive;
    }
}

/// Total synergy count (360 canon).
pub fn synergy_count() -> usize {
    PARTS.iter().map(|p| p.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_shape() {
        assert_eq!(synergy_count(), 360, "want all 360 canon synergies");
        let mut ids = std::collections::HashSet::new();
        for part in PARTS.iter() {
            for s in part.iter() {
                assert!(ids.insert(s.id), "duplicate synergy id {}", s.id);
                assert!((2..=5).contains(&s.members.len()), "{} has {} members", s.name, s.members.len());
                let mem = member_idxs(s);
                assert_eq!(mem.len(), s.members.len(), "{} has unknown members", s.name);
                for i in mem {
                    assert_eq!(GU[i].path, s.path, "{} member from wrong path", s.name);
                }
            }
        }
    }

    #[test]
    fn pair_activates() {
        // Sanguine Drain: Blood Dripping (Qi? no — Blood rank 1) + Blood Siphon
        let map = name_index();
        let inv = vec![map["Blood Dripping Gu"], map["Blood Siphon Gu"]];
        let active = active_synergies(&inv);
        assert!(active.iter().any(|s| s.id == 0), "Sanguine Drain must fire");
        let mut t = ItemTotals::default();
        apply_synergies(&mut t, &inv);
        assert!(t.lifesteal > 0.0);
        // one half alone fires nothing
        assert!(active_synergies(&inv[..1]).is_empty());
    }
}
