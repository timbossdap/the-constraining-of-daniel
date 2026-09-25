//! Combat math + loot. Juicy crits, executes, lifesteal — fun > grind.

use crate::player_class::SkillKind;
use crate::progression::PlayerCore;

#[derive(Debug, Clone)]
pub struct HitResult {
    pub damage: f32,
    pub crit: bool,
    pub dodged: bool,
    pub killed: bool,
}

pub fn player_attack_damage(player: &PlayerCore, skill_idx: usize, seed: u64) -> (f32, SkillKind) {
    let skills = player.class.skills();
    let s = &skills[skill_idx.min(3)];
    let d = player.derived();
    let base = if player.class.is_ranged() || matches!(s.kind, SkillKind::Holy | SkillKind::Lifesteal { .. } | SkillKind::Burning { .. } | SkillKind::Chill { .. } | SkillKind::Nova { .. } | SkillKind::Barrage | SkillKind::Meteor) {
        d.magic_atk.max(d.phys_atk * 0.7)
    } else {
        d.phys_atk.max(d.magic_atk * 0.5)
    };
    // variance ±15% from seed (deterministic, no RNG crate needed)
    let var = 0.85 + ((seed % 30) as f32) / 100.0;
    let mut dmg = base * s.power * var * player.combo_multiplier();

    // Rogue backstab: +50% if combo >= 5
    if matches!(s.kind, SkillKind::Backstab) && player.combo_count >= 5 {
        dmg *= 1.5;
    }
    // class identity: glass cannons hit harder, walls hit softer
    dmg *= player.class.passive().power;
    // inventory power (multiplicative with everything above)
    dmg *= player.item_totals().power_mult;
    // heirloom power from witnessed endings (rebirth reward)
    dmg *= player.heirloom_mult;
    (dmg, s.kind.clone())
}

pub fn roll_crit(player: &PlayerCore, seed: u64) -> bool {
    let d = player.derived();
    ((seed % 100) as f32) / 100.0 < d.crit_chance + player.item_totals().crit_add
}

pub fn apply_crit(dmg: f32, player: &PlayerCore) -> f32 {
    dmg * player.derived().crit_mult
}
