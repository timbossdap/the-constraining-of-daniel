//! Pickups: hearts, essence, gold, bombs, chests, shrines.

/// Pickups strewn around — anti-grind fun: free heals, bombs, magnets.
#[derive(Debug, Clone)]
pub struct Pickup {
    pub pos: (f32, f32),
    pub kind: PickupKind,
    pub bob: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickupKind {
    Heart,
    Essence,
    Gold,
    Bomb,
    Chest,
    Shrine,
}
