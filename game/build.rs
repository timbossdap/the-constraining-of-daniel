//! Codegen for the 360 Gu worms (12 paths x 30, canon names/ranks).
//! Effects/arch/descs derive from deterministic per-path rules so the table
//! stays coherent without 360 hand-written entries.

use std::env;
use std::fmt::Write as _;
use std::fs;

// (name, rank) in path order 0..11, 30 each. Ranks from the canon list
// (Guts Gu's "1-5/6" spread resolves to 5).
const PATHS: [&str; 12] = [
    "Qi", "Enslavement", "Wisdom", "Transformation", "Wood", "Soul",
    "Luck", "Blood", "Food", "Theft", "Refinement", "Dream",
];

const NAMES: [[(&str, u8); 30]; 12] = [
    [
        ("Qi Gathering Gu", 1), ("Qi Arrow Gu", 2), ("Qi Blade Gu", 2), ("Qi Shield Gu", 3),
        ("Qi Armor Gu", 4), ("Qi Wall Gu", 4), ("Qi Explosion Gu", 5), ("Rising Qi Gu", 5),
        ("Primordial Qi Gu", 6), ("Qi Grandmaster Gu", 6), ("Qi Breath Gu", 1), ("Cloud Qi Gu", 2),
        ("Qi Wave Gu", 3), ("Qi Current Gu", 3), ("Cold Qi Gu", 3), ("Warm Qi Gu", 3),
        ("Qi Flow Gu", 4), ("Qi Sea Gu", 5), ("Mainstream Qi Gu", 5), ("Dominant Qi Gu", 5),
        ("Qi Form Gu", 6), ("Sword Qi Gu", 3), ("Blade Qi Gu", 3), ("Killing Qi Gu", 4),
        ("Evil Qi Gu", 4), ("Righteous Qi Gu", 4), ("Vital Qi Gu", 5), ("Death Qi Gu", 5),
        ("Qi Domain Gu", 6), ("Heaven-Earth Qi Gu", 6),
    ],
    [
        ("Beast Enslavement Gu", 1), ("Wolf Enslavement Gu", 2), ("Dog Enslavement Gu", 2),
        ("Bear Enslavement Gu", 2), ("Eagle Enslavement Gu", 3), ("Snake Enslavement Gu", 3),
        ("Enslavement Intent Gu", 3), ("Man Enslavement Gu", 4), ("Dragon Enslavement Gu", 5),
        ("Myriad Beast Enslavement Gu", 5), ("Change Soul Enslavement Gu", 6), ("Fox Enslavement Gu", 2),
        ("Tiger Enslavement Gu", 3), ("Shark Enslavement Gu", 4), ("Bat Enslavement Gu", 3),
        ("Crocodile Enslavement Gu", 4), ("Bull Enslavement Gu", 2), ("Deer Enslavement Gu", 2),
        ("Spider Enslavement Gu", 3), ("Rat Enslavement Gu", 1), ("Insect Enslavement Gu", 1),
        ("Bird Enslavement Gu", 2), ("Fish Enslavement Gu", 2), ("Elephant Enslavement Gu", 4),
        ("Rhinoceros Enslavement Gu", 4), ("Phoenix Enslavement Gu", 5), ("Qilin Enslavement Gu", 5),
        ("Enslavement Command Gu", 4), ("Enslavement Mark Gu", 5), ("Group Control Gu", 6),
    ],
    [
        ("Clear Mind Gu", 1), ("Star Thought Gu", 2), ("Memory Gu", 3), ("Heart's Intent Gu", 3),
        ("Delightful Thought Gu", 4), ("Connect Mind Gu", 5), ("Deduce Gu", 5),
        ("Unravel Mystery Gu", 6), ("Wisdom Sword Gu", 6), ("Heavenly Secret Gu", 6),
        ("Calm Thought Gu", 1), ("Fast Thought Gu", 2), ("Deep Thought Gu", 3), ("Will Gu", 3),
        ("Emotion Gu", 3), ("Perception Gu", 4), ("Comprehension Gu", 4), ("Scheme Gu", 5),
        ("Calculation Gu", 5), ("Mind Light Gu", 2), ("Wise Light Gu", 3), ("False Thought Gu", 3),
        ("Malicious Thought Gu", 4), ("Kind Thought Gu", 4), ("Wisdom Heart Gu", 5),
        ("Deductive Light Gu", 5), ("Cognition Gu", 6), ("Wisdom Seal Gu", 6),
        ("Thought Wave Gu", 4), ("Mind Reading Gu", 5),
    ],
    [
        ("Fur Gu", 1), ("Bear Skin Gu", 2), ("Eagle Feather Gu", 3), ("Tiger Claw Gu", 3),
        ("Lion Fur Gu", 4), ("Change Form Gu", 4), ("Dragon Scales Gu", 5),
        ("Flying Bear Transformation Gu", 6), ("Assimilation Wind Transformation Gu", 6),
        ("Myriad Beings Transformation Gu", 6), ("Fish Scale Gu", 1), ("Snake Skin Gu", 2),
        ("Wolf Tail Gu", 2), ("Bull Horn Gu", 3), ("Shark Fin Gu", 3), ("Turtle Shell Gu", 4),
        ("Phoenix Feather Gu", 5), ("Soft Body Gu", 2), ("Bone Morph Gu", 3),
        ("Giant Transformation Gu", 4), ("Miniature Transformation Gu", 4),
        ("Water Transformation Gu", 5), ("Fire Transformation Gu", 5),
        ("Lightning Transformation Gu", 5), ("Shadow Transformation Gu", 5),
        ("Dragon Transformation Gu", 6), ("Phoenix Transformation Gu", 6),
        ("Tiger Transformation Gu", 6), ("Ape Transformation Gu", 6), ("Complete Change Gu", 6),
    ],
    [
        ("Spring Grass Gu", 1), ("Healing Grass Gu", 1), ("Wood Bark Gu", 2), ("Leaf Blade Gu", 2),
        ("Bamboo Root Gu", 3), ("Pine Needle Gu", 3), ("Tree Shield Gu", 4), ("Vitality Gu", 4),
        ("Wood Sprouts Gu", 5), ("Wooden Chicken Gu", 6), ("Vine Whip Gu", 1), ("Leaf Shield Gu", 2),
        ("Tree Bark Armor Gu", 3), ("Thorn Gu", 3), ("Flower Petal Gu", 2), ("Pollen Gu", 2),
        ("Root Bound Gu", 3), ("Wood Spirit Gu", 4), ("Ancient Tree Gu", 5),
        ("Forest Fire Immunity Gu", 4), ("Timber Gu", 3), ("Wood Essence Gu", 5),
        ("Wood Core Gu", 4), ("Willow Leaf Gu", 2), ("Peach Blossom Gu", 3), ("Grass Sword Gu", 3),
        ("Wood Path Gu", 5), ("Life Tree Gu", 6), ("Divine Wood Gu", 6), ("Eternal Spring Gu", 6),
    ],
    [
        ("Soul Light Gu", 1), ("Soul Searching Gu", 2), ("Soul Armor Gu", 3),
        ("Soul Pacifying Gu", 3), ("Soul Howl Gu", 4), ("Soul Shaking Gu", 4),
        ("Soul Beast Summoning Gu", 5), ("Guts Gu", 5), ("Change Soul Gu", 6), ("Cleanse Soul Gu", 6),
        ("Soul Worm", 1), ("Soul Shield Gu", 2), ("Soul Fire Gu", 3), ("Soul Suppression Gu", 4),
        ("Soul Refinement Gu", 5), ("Soul Cultivation Gu", 3), ("Split Soul Gu", 5),
        ("Soul Shift Gu", 4), ("Soul Trap Gu", 3), ("Soul Devour Gu", 5), ("Phantom Soul Gu", 4),
        ("Strong Soul Gu", 2), ("Soul Spear Gu", 3), ("Soul Chain Gu", 4), ("Soul Mark Gu", 5),
        ("Wandering Soul Gu", 5), ("Suppress Soul Gu", 6), ("Soul Sound Gu", 4),
        ("Soul Mirror Gu", 5), ("Soul Unity Gu", 6),
    ],
    [
        ("Good Luck Gu", 1), ("Bad Luck Gu", 2), ("Notice Luck Gu", 2), ("Inspect Luck Gu", 3),
        ("Break Luck Gu", 4), ("Transfer Luck Gu", 5), ("Dog Shit Luck Gu", 6),
        ("Connect Luck Gu", 6), ("Rivaling Heaven Gu", 6), ("Calamity Beckoning Gu", 6),
        ("Small Luck Gu", 1), ("Fortune Gu", 2), ("Misfortune Gu", 2), ("Luck Shield Gu", 3),
        ("Luck Inspection Gu", 3), ("Gather Luck Gu", 4), ("Scatter Luck Gu", 4),
        ("Suppress Luck Gu", 5), ("Borrow Luck Gu", 5), ("Steal Luck Gu", 5), ("Plunder Luck Gu", 5),
        ("Luck Suppression Gu", 4), ("Fortunate Star Gu", 3), ("Calamity Suppression Gu", 5),
        ("Luck Light Gu", 2), ("Qi Luck Gu", 4), ("Immortal Luck Gu", 6), ("Heaven's Luck Gu", 6),
        ("Earth's Luck Gu", 6), ("All Living Beings Luck Gu", 6),
    ],
    [
        ("Blood Dripping Gu", 1), ("Blood Stain Gu", 2), ("Blood Blade Gu", 2), ("Blood Shield Gu", 3),
        ("Blood Wing Gu", 3), ("Blood Moon Gu", 4), ("Blood Skull Gu", 5), ("Blood Asset Gu", 5),
        ("Blood Deity Gu", 6), ("Blood Torrent Gu", 6), ("Blood Qi Gu", 1), ("Blood Flow Gu", 2),
        ("Blood Arrow Gu", 2), ("Blood Spear Gu", 3), ("Blood Armor Gu", 3), ("Blood Siphon Gu", 4),
        ("Blood Boil Gu", 4), ("Blood Sea Gu", 5), ("Blood Purification Gu", 4),
        ("Blood Stasis Gu", 3), ("Blood Lineage Gu", 5), ("Blood Shadow Gu", 4), ("Blood Wave Gu", 3),
        ("Blood Essence Gu", 5), ("Blood Clone Gu", 5), ("Blood Contract Gu", 4),
        ("Blood Demon Gu", 6), ("Blood River Gu", 6), ("Blood Path Gu", 6), ("Blood Soul Gu", 5),
    ],
    [
        ("Rice Gu", 1), ("Liquor Worm", 1), ("Snack Gu", 2), ("Wine Essence Gu", 3),
        ("Big Belly Gu", 3), ("Satiation Gu", 4), ("Iron Stomach Gu", 5), ("Gourmet Gu", 5),
        ("Cook Gu", 6), ("Eat Strength Gu", 6), ("Water Wine Gu", 1), ("Food Gathering Gu", 2),
        ("Appetite Gu", 2), ("Delicious Gu", 3), ("Meat Gu", 2), ("Wine Cup Gu", 3),
        ("Fasting Gu", 4), ("Hunger Gu", 3), ("Feast Gu", 5), ("Banquet Gu", 5),
        ("Food Storage Gu", 4), ("Digestion Gu", 3), ("Taste Gu", 2), ("Sweetness Gu", 1),
        ("Sourness Gu", 1), ("Bitterness Gu", 1), ("Spiciness Gu", 1), ("Food Refinement Gu", 5),
        ("Immortal Wine Gu", 6), ("Heavenly Feast Gu", 6),
    ],
    [
        ("Thief Hand Gu", 1), ("Sneak Attack Gu", 2), ("Pickpocket Gu", 3), ("Plunder Gu", 4),
        ("Concealment Gu", 5), ("Steal Heart Gu", 5), ("Steal Dao Gu", 6), ("Steal Life Gu", 6),
        ("Thievish Hand Gu", 6), ("Shadowless Thief Gu", 6), ("Steal Essence Gu", 1),
        ("Steal Primeval Gu", 2), ("Steal Thought Gu", 3), ("Steal Vision Gu", 3),
        ("Steal Sound Gu", 3), ("Steal Light Gu", 4), ("Steal Shadow Gu", 4), ("Steal Gu", 5),
        ("Thief Shadow Gu", 4), ("Concealed Track Gu", 3), ("Silent Footstep Gu", 2),
        ("Pocket Steal Gu", 2), ("Lift Luck Gu", 5), ("Steal Soul Gu", 5), ("Steal Form Gu", 5),
        ("Steal Space Gu", 6), ("Steal Time Gu", 6), ("Steal Sky Gu", 6), ("Steal Earth Gu", 6),
        ("Phantom Hand Gu", 5),
    ],
    [
        ("Refine Gu", 1), ("Nurture Gu", 2), ("Fire Refinement Gu", 3),
        ("Water Refinement Gu", 3), ("Quiet Refinement Gu", 4), ("Unify Form Gu", 4),
        ("Success Gu", 5), ("Advanced Refinement Gu", 5),
        ("Heavenly Essence Treasure Monarch Lotus", 6), ("Refine Sea Gu", 6),
        ("Refinement Fire Gu", 1), ("Refinement Water Gu", 2), ("Quick Refinement Gu", 3),
        ("Force Refinement Gu", 4), ("Natural Refinement Gu", 5), ("Refinement Aid Gu", 2),
        ("Refinement Light Gu", 3), ("Core Refinement Gu", 4), ("Refinement Mark Gu", 5),
        ("Batch Refinement Gu", 4), ("Auxiliary Refinement Gu", 3),
        ("Gentle Refinement Gu", 3), ("Ice Refinement Gu", 4), ("Lightning Refinement Gu", 4),
        ("Earth Refinement Gu", 4), ("Wind Refinement Gu", 4), ("Blood Refinement Gu", 5),
        ("Spirit Refinement Gu", 5), ("Heaven Refinement Gu", 6), ("Refine Dao Gu", 6),
    ],
    [
        ("Dream Leaf Gu", 1), ("Dream Butterfly Gu", 2), ("Dream Tapestry Gu", 3),
        ("Dream Realm Gu", 3), ("Dream Trap Gu", 4), ("Dream Armor Gu", 4),
        ("Unravel Dream Gu", 5), ("Dream Seeker Gu", 5), ("Dream Wings Gu", 6),
        ("Great Dream Gu", 6), ("Dream Chaser Gu", 1), ("Dream Vision Gu", 2),
        ("Dream Sound Gu", 2), ("Dream Light Gu", 3), ("Dream Form Gu", 3), ("Dream Soul Gu", 4),
        ("Dream Mind Gu", 4), ("Dream Illusion Gu", 4), ("Dream Shield Gu", 4),
        ("Dream Blade Gu", 5), ("Dream Spear Gu", 5), ("Dream Escape Gu", 5),
        ("Dream Travel Gu", 5), ("Dream Domain Gu", 5), ("Nightmare Gu", 4),
        ("Sweet Dream Gu", 3), ("Dream Creation Gu", 6), ("Dream Control Gu", 6),
        ("Dream Lord Gu", 6), ("Dream World Gu", 6),
    ],
];

// signature attribute per path (rank-1 worms)
const ATTRS: [&str; 12] = [
    "Strength", "Vitality", "Intellect", "Vitality", "Vitality", "Intellect",
    "Luck", "Strength", "Vitality", "Agility", "Intellect", "Agility",
];
const ARCHES: [&str; 12] = [
    "Blade", "Oddity", "Staff", "Axe", "Spear", "Staff",
    "Dagger", "Blade", "Maul", "Bow", "Maul", "Bow",
];

fn f2(v: f32) -> String {
    format!("{:.2}", (v * 100.0).round() / 100.0)
}

// effect expression + human desc, per (path, rank, occurrence-in-rank)
fn effects(path: usize, rank: u8, occ: usize) -> (Vec<String>, Vec<String>) {
    let mut fx = Vec::new();
    let mut ds = Vec::new();
    let mut push = |e: String, d: String| {
        fx.push(e);
        ds.push(d);
    };
    let stat = || format!("E::Stat(AttrKind::{}, 1)", ATTRS[path]);
    match rank {
        1 => {
            if occ == 0 {
                push(stat(), format!("+1 {}.", ATTRS[path].to_uppercase()));
            } else {
                push(format!("E::Power({})", f2(1.03 + 0.01 * occ as f32)),
                    format!("+{}% damage.", (3 + occ) as u32));
            }
        }
        2 => {
            if occ % 2 == 0 {
                push(format!("E::Power({})", f2(1.05 + 0.01 * occ as f32)),
                    format!("+{}% damage.", 5 + occ as u32));
            } else {
                match path {
                    0 => push("E::Range(1.06)".into(), "+6% range.".into()),
                    1 => push("E::Thorns(0.15)".into(), "Reflect bites.".into()),
                    2 => push("E::Xp(1.08)".into(), "+8% XP.".into()),
                    3 => push("E::Guard(0.97)".into(), "Take 3% less.".into()),
                    4 => push("E::Regen(0.40)".into(), "Regrow steadily.".into()),
                    5 | 7 => push("E::Lifesteal(0.02)".into(), "Drink a little.".into()),
                    6 => push("E::Crit(0.02)".into(), "+2% crit.".into()),
                    8 => push("E::Potion(1.10)".into(), "Potions +10%.".into()),
                    9 => push("E::Gold(1.06)".into(), "+6% stones.".into()),
                    10 => push("E::Power(1.07)".into(), "+7% damage.".into()),
                    _ => push("E::Dodge(0.02)".into(), "+2% dodge.".into()),
                }
            }
        }
        3 => match path {
            0 | 7 => push("E::Power(1.08)".into(), "+8% damage.".into()),
            1 => push("E::Thorns(0.25)".into(), "Teeth everywhere.".into()),
            2 | 6 => push("E::Crit(0.04)".into(), "+4% crit.".into()),
            3 => push("E::Guard(0.92)".into(), "Take 8% less.".into()),
            4 => push("E::Regen(0.80)".into(), "Bark knits fast.".into()),
            5 => push("E::Lifesteal(0.03)".into(), "Sip spirits.".into()),
            8 => push("E::Potion(1.20)".into(), "Potions +20%.".into()),
            9 => push("E::Gold(1.12)".into(), "+12% stones.".into()),
            10 => push("E::Power(1.10)".into(), "+10% damage.".into()),
            _ => push("E::Dodge(0.04)".into(), "+4% dodge.".into()),
        },
        4 => match path {
            0 => push("E::Arrows(1)".into(), "+1 projectile.".into()),
            1 => push("E::ShieldKill(4.0)".into(), "Kills ward you.".into()),
            2 => push("E::Xp(1.20)".into(), "+20% XP.".into()),
            3 => push("E::Guard(0.90)".into(), "Take 10% less.".into()),
            4 => push("E::Regen(1.20)".into(), "Sap runs hot.".into()),
            5 => push("E::Lifesteal(0.05)".into(), "Feast on falls.".into()),
            6 => push("E::Gold(1.20)".into(), "+20% stones.".into()),
            7 => push("E::Power(1.12)".into(), "+12% damage.".into()),
            8 => push("E::Potion(1.30)".into(), "Potions +30%.".into()),
            9 => push("E::Gold(1.15)".into(), "+15% stones.".into()),
            10 => push("E::Power(1.15)".into(), "+15% damage.".into()),
            _ => push("E::Dodge(0.06)".into(), "+6% dodge.".into()),
        },
        5 => {
            let sig: (String, String) = match path {
                0 => ("E::Arrows(1)".into(), "+1 projectile.".into()),
                1 => ("E::ShieldKill(5.0)".into(), "Thick kill-wards.".into()),
                2 => ("E::Xp(1.25)".into(), "+25% XP.".into()),
                3 => ("E::Guard(0.90)".into(), "Take 10% less.".into()),
                4 => ("E::Regen(1.60)".into(), "Old-growth vigor.".into()),
                5 => ("E::Lifesteal(0.06)".into(), "Deep drafts.".into()),
                6 => ("E::Crit(0.05)".into(), "+5% crit.".into()),
                7 => ("E::Lifesteal(0.04)".into(), "Red communion.".into()),
                8 => ("E::Potion(1.40)".into(), "Potions +40%.".into()),
                9 => ("E::Gold(1.25)".into(), "+25% stones.".into()),
                10 => ("E::Power(1.20)".into(), "+20% damage.".into()),
                _ => ("E::Dodge(0.07)".into(), "+7% dodge.".into()),
            };
            let pw = if path == 10 { 1.20 } else { 1.15 };
            push(format!("E::Power({})", f2(pw)), format!("+{}% damage.", ((pw - 1.0) * 100.0) as u32));
            push(sig.0, sig.1);
            if path == 2 {
                // deduction worms guide the hand: shots steer home
                push("E::Homing(1.50)".into(), "Shots steer true.".into());
            }
        }
        _ => {
            let sig: (String, String) = match path {
                0 => ("E::Arrows(2)".into(), "+2 projectiles.".into()),
                1 => ("E::ShieldKill(8.0)".into(), "A wall of the fallen.".into()),
                2 => ("E::Xp(1.60)".into(), "+60% XP.".into()),
                3 => ("E::Guard(0.82)".into(), "Take 18% less.".into()),
                4 => ("E::Regen(2.50)".into(), "Undying grove.".into()),
                5 => ("E::Lifesteal(0.10)".into(), "Soul tithe.".into()),
                6 => ("E::Crit(0.08)".into(), "+8% crit.".into()),
                7 => ("E::Power(1.30)".into(), "+30% damage.".into()),
                8 => ("E::Potion(1.60)".into(), "Potions +60%.".into()),
                9 => ("E::Gold(1.40)".into(), "+40% stones.".into()),
                10 => ("E::Power(1.35)".into(), "+35% damage.".into()),
                _ => ("E::Dodge(0.10)".into(), "+10% dodge.".into()),
            };
            let pw = match path {
                7 | 10 => 1.30,
                3 => 1.15,
                _ => 1.25,
            };
            if path == 7 || path == 10 {
                push(sig.0.clone(), sig.1.clone());
            } else {
                push(format!("E::Power({})", f2(pw)), format!("+{}% damage.", ((pw - 1.0) * 100.0) as u32));
                push(sig.0, sig.1);
            }
            if path == 3 {
                push("E::Speed(1.06)".into(), "+6% speed.".into());
            }
        }
    }
    (fx, ds)
}

fn main() {
    // validate the canon input before emitting a line
    let mut seen = std::collections::HashSet::new();
    for (p, path) in NAMES.iter().enumerate() {
        assert_eq!(path.len(), 30, "path {p} must list 30 Gu");
        assert_eq!(path[0].1, 1, "path {p} must open on rank 1");
        assert!(path.iter().any(|g| g.1 == 6), "path {p} needs rank 6");
        let mut ranks = [false; 7];
        for (name, rank) in path.iter() {
            assert!((1..=6).contains(rank), "bad rank for {name}");
            ranks[*rank as usize] = true;
            assert!(seen.insert(name.to_string()), "duplicate Gu: {name}");
        }
    }
    let out = env::var("OUT_DIR").unwrap();
    let mut s = String::new();
    s.push_str("/// Generated by build.rs: 360 canon Gu (12 paths x 30).\n");
    s.push_str("pub const GU: [GuDef; 360] = [\n");
    for (p, path) in NAMES.iter().enumerate() {
        writeln!(s, "    // ---- {} Path ({}) ----", PATHS[p], p).unwrap();
        let mut rank_occ = [0usize; 7];
        for (name, rank) in path.iter() {
            let occ = rank_occ[*rank as usize];
            rank_occ[*rank as usize] += 1;
            let (fx, ds) = effects(p, *rank, occ);
            let fx_list = fx.join(", ");
            let desc = ds.join(" ");
            // escape for rust string literal
            let nm = name.replace('"', "'");
            writeln!(
                s,
                "    GuDef {{ name: \"{nm}\", path: {p}, rank: {rank}, arch: WeaponArch::{arch}, desc: \"{desc}\", effects: &[{fx_list}] }},",
                arch = ARCHES[p],
            )
            .unwrap();
        }
    }
    s.push_str("];\n");
    fs::write(format!("{out}/gu_table.rs"), s).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
