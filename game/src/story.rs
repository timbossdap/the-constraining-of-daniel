//! Story engine: a cultivation saga in three columns (Special / Repeatable /
//! Incomplete), Reverend-Insanity-inspired. Sects, long branches, endings.
//! Combat stays twin-stick arena runs; the story is the spine that launches,
//! rewards, and ends them.

use std::collections::{BTreeMap, BTreeSet};

/// Persistent saga state. Lives in GameInner so death/reincarnate can scope it.
#[derive(Debug, Clone, Default)]
pub struct StoryState {
    pub node: String,
    pub stamina: f32,
    pub max_stamina: u32,
    pub flags: BTreeSet<String>,
    pub counters: BTreeMap<String, u32>,
    pub sect: Option<String>,
    pub consumed: BTreeSet<String>,
    pub log: Vec<String>,
    pub run_active: bool,
    /// Bedded down via Sleep: stamina regens fast until another choice
    /// interrupts (or the bar fills and you wake on your own).
    pub sleeping: bool,
}

impl StoryState {
    pub fn new() -> Self {
        let mut s = Self {
            node: "ashes".to_string(),
            stamina: 10.0,
            max_stamina: 12,
            ..Default::default()
        };
        s.slog("The village is ash. You are what's left.".to_string());
        s
    }

    pub fn slog(&mut self, line: String) {
        self.log.push(line);
        while self.log.len() > 40 {
            self.log.remove(0);
        }
    }

    pub fn contrib(&self) -> u32 {
        self.counters.get("contrib").copied().unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceKind {
    Special,
    Repeatable,
}

#[derive(Debug, Clone)]
pub struct Req {
    pub min_level: u32,
    pub path: Option<u8>,
    pub sect: Option<&'static str>,
    pub no_sect: bool,
    pub flag: Option<&'static str>,
    pub noflag: Option<&'static str>,
    pub stamina: u32,
    pub stones: u32,
    pub kills: u32,
    pub gu_rank: u8,
    pub contrib: u32,
}

impl Req {
    pub fn free() -> Self {
        Self {
            min_level: 0,
            path: None,
            sect: None,
            no_sect: false,
            flag: None,
            noflag: None,
            stamina: 0,
            stones: 0,
            kills: 0,
            gu_rank: 0,
            contrib: 0,
        }
    }

    /// Flat view of the run for requirement checks (no engine types).
    pub fn check(
        &self,
        level: u32,
        path: Option<usize>,
        sect: Option<&str>,
        flags: &BTreeSet<String>,
        counters: &BTreeMap<String, u32>,
        stamina: f32,
        stones: u32,
        kills: u32,
        best_gu_rank: u8,
    ) -> Option<String> {
        if level < self.min_level {
            return Some(format!("Lv {}", self.min_level));
        }
        if let Some(p) = self.path {
            if path != Some(p as usize) {
                return Some("sealed fate".to_string());
            }
        }
        if let Some(s) = self.sect {
            if sect != Some(s) {
                return Some("other sect".to_string());
            }
        }
        if self.no_sect && sect.is_some() {
            return Some("sectless only".to_string());
        }
        if let Some(f) = self.flag {
            if !flags.contains(f) {
                return Some("unproven".to_string());
            }
        }
        if let Some(f) = self.noflag {
            if flags.contains(f) {
                return Some("already done".to_string());
            }
        }
        if stamina < self.stamina as f32 {
            return Some("no stamina".to_string());
        }
        if stones < self.stones {
            return Some(format!("{} stones", self.stones));
        }
        if kills < self.kills {
            return Some(format!("{} kills", self.kills));
        }
        if best_gu_rank < self.gu_rank {
            return Some(format!("rank {} Gu", self.gu_rank));
        }
        let have = counters.get("contrib").copied().unwrap_or(0);
        if have < self.contrib {
            return Some(format!("{} contrib", self.contrib));
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum StoryEffect {
    Stones(i32),
    Xp(u32),
    HealFull,
    Flag(String),
    Sect(Option<String>),
    Contrib(u32),
    GrantGuRank(u8),
    GrantPathGu { path: u8, rank: u8 },
    /// Bed down: passive stamina regen until another choice interrupts.
    Sleep,
    /// Step into the ring: one elite duel in the arena, purse on victory.
    Spar { foe: &'static str, stones: u32, xp: u32, rank: u8 },
    /// Burn three worms of a rank into one worm a rank higher.
    Refine { rank: u8 },
    StartArena,
    Reincarnate { bonus: bool },
    Log(String),
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub label: &'static str,
    pub kind: ChoiceKind,
    pub once: bool,
    pub req: Req,
    pub effects: Vec<StoryEffect>,
    pub goto: &'static str,
}

#[derive(Debug, Clone)]
pub struct StoryNode {
    pub id: &'static str,
    pub title: &'static str,
    pub body: &'static str,
    pub ending: bool,
    pub choices: Vec<Choice>,
}

fn r(
    min_level: u32,
    stamina: u32,
    stones: u32,
    kills: u32,
    gu_rank: u8,
    contrib: u32,
) -> Req {
    Req {
        min_level,
        stamina,
        stones,
        kills,
        gu_rank,
        contrib,
        ..Req::free()
    }
}

fn c(
    label: &'static str,
    kind: ChoiceKind,
    once: bool,
    req: Req,
    effects: Vec<StoryEffect>,
    goto: &'static str,
) -> Choice {
    Choice { label, kind, once, req, effects, goto }
}

fn fx_flag(f: &str) -> StoryEffect {
    StoryEffect::Flag(f.to_string())
}

fn fx_log(s: String) -> StoryEffect {
    StoryEffect::Log(s)
}

/// Visible choice for UI + input: (column 0=special 1=repeatable 2=incomplete,
/// display label, lock reason or cost note, node index, choice index).
#[derive(Debug, Clone)]
pub struct VisibleChoice {
    pub col: usize,
    pub label: String,
    pub note: String,
    pub node_idx: usize,
    pub choice_idx: usize,
}

pub fn visible_choices(
    node: &StoryNode,
    node_idx: usize,
    level: u32,
    path: Option<usize>,
    sect: Option<&str>,
    flags: &BTreeSet<String>,
    counters: &BTreeMap<String, u32>,
    stamina: f32,
    stones: u32,
    kills: u32,
    best_gu_rank: u8,
) -> Vec<VisibleChoice> {
    let mut out = Vec::new();
    for (ci, ch) in node.choices.iter().enumerate() {
        if ch.once && flags.contains(&format!("done:{}:{}", node.id, ch.label)) {
            continue;
        }
        let lock = ch.req.check(level, path, sect, flags, counters, stamina, stones, kills, best_gu_rank);
        let (col, note) = match (&ch.kind, lock) {
            (_, Some(reason)) => (2, reason),
            (ChoiceKind::Special, None) => {
                let cost = if ch.req.stamina > 0 {
                    format!("{} STAM", ch.req.stamina)
                } else {
                    String::new()
                };
                (0, cost)
            }
            (ChoiceKind::Repeatable, None) => {
                let cost = if ch.req.stamina > 0 {
                    format!("{} STAM", ch.req.stamina)
                } else {
                    String::new()
                };
                (1, cost)
            }
        };
        out.push(VisibleChoice {
            col,
            label: ch.label.to_string(),
            note,
            node_idx,
            choice_idx: ci,
        });
    }
    out
}

/// Column-major flattened choice indices (max 5 per column), shared by the
/// sync (draw) and the update branch (keyboard/mouse) so both agree on order.
pub fn flat_choices(items: &[VisibleChoice]) -> Vec<usize> {
    let mut out = Vec::new();
    for c in 0..3 {
        let mut n = 0;
        for (idx, it) in items.iter().enumerate() {
            if it.col == c {
                if n >= 5 {
                    break;
                }
                n += 1;
                out.push(idx);
            }
        }
    }
    out
}

pub fn story_nodes() -> Vec<StoryNode> {
    vec![
        StoryNode {
            id: "ashes",
            title: "Ashes of Qing Mao",
            body: "You wake under a collapsed roof-beam with the taste of smoke in your mouth. Gu Yue village — your village — is a bowl of fire. Through the burning lanes move cultivators in red-threaded robes, pulling primeval stones out of the clan storehouse while the village Gu stores crack open in the heat.\n\nSomething small and warm wriggles against your ankle. A rank-1 Gu of your own path, fled from the shattered pens, has chosen you the way drowning men choose driftwood. Behind you the granary collapses in a gout of sparks. Ahead, only smoke and decisions.",
            ending: false,
            choices: vec![
                c("Bury the dead", ChoiceKind::Special, true, Req::free(),
                    vec![fx_flag("buried"), StoryEffect::Stones(10), fx_log("You buried who you could find. The mountain remembers.".to_string())],
                    "clearing"),
                c("Follow the smoke", ChoiceKind::Special, true, Req::free(),
                    vec![fx_flag("smoke"), fx_log("You follow the raiders' trail east, memorizing faces.".to_string())],
                    "cinder_out"),
                // ---- path communions: one worm-calling per Gu path, gated by
                // fate — yours answers, the other eleven show as sealed ----
                c("Breathe with the dawn", ChoiceKind::Special, true, Req { path: Some(0), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 0, rank: 2 }, StoryEffect::Xp(40), fx_log("Your breath and the worm's breath fall into one rhythm.".to_string())],
                    "ashes"),
                c("Leash a wild dog-spirit", ChoiceKind::Special, true, Req { path: Some(1), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 1, rank: 2 }, StoryEffect::Stones(20), fx_log("It snarls, then kneels. Everything kneels eventually.".to_string())],
                    "ashes"),
                c("Read the cinder-patterns", ChoiceKind::Special, true, Req { path: Some(2), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 2, rank: 2 }, StoryEffect::Xp(60), fx_log("The ash spells out tomorrow, if you squint right.".to_string())],
                    "ashes"),
                c("Shed yesterday's skin", ChoiceKind::Special, true, Req { path: Some(3), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 3, rank: 2 }, StoryEffect::HealFull, fx_log("It itches. Then you are stronger than the itch.".to_string())],
                    "ashes"),
                c("Root beneath the ash", ChoiceKind::Special, true, Req { path: Some(4), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 4, rank: 2 }, StoryEffect::Stones(12), StoryEffect::Xp(20), fx_log("Small green defiance under the cinders.".to_string())],
                    "ashes"),
                c("Drink a lingering ghost", ChoiceKind::Special, true, Req { path: Some(5), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 5, rank: 2 }, StoryEffect::Xp(50), fx_log("It tastes like somebody's unfinished business. Useful.".to_string())],
                    "ashes"),
                c("Bet on a green beetle", ChoiceKind::Special, true, Req { path: Some(6), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 6, rank: 2 }, StoryEffect::Stones(30), fx_log("The beetle wins. The beetle always wins for you.".to_string())],
                    "ashes"),
                c("Tithe your own blood", ChoiceKind::Special, true, Req { path: Some(7), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 7, rank: 2 }, StoryEffect::Xp(55), fx_log("A bowl of red for a worm of red. Fair trade.".to_string())],
                    "ashes"),
                c("Roast a fat rabbit", ChoiceKind::Special, true, Req { path: Some(8), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 8, rank: 2 }, StoryEffect::HealFull, StoryEffect::Stones(10), fx_log("Crackling fat. The worm gets the eyes; you get the rest.".to_string())],
                    "ashes"),
                c("Palm the quartermaster's purse", ChoiceKind::Special, true, Req { path: Some(9), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 9, rank: 2 }, StoryEffect::Stones(35), fx_log("Heavier on your side, lighter on his. Nobody saw.".to_string())],
                    "ashes"),
                c("Smelt cinder-iron", ChoiceKind::Special, true, Req { path: Some(10), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 10, rank: 2 }, StoryEffect::Xp(45), StoryEffect::Stones(10), fx_log("The slag hides a worm that appreciates craft.".to_string())],
                    "ashes"),
                c("Nap inside the dream", ChoiceKind::Special, true, Req { path: Some(11), stamina: 2, ..Req::free() },
                    vec![StoryEffect::GrantPathGu { path: 11, rank: 2 }, StoryEffect::Xp(35), StoryEffect::HealFull, fx_log("You wake with a worm you don't remember catching.".to_string())],
                    "ashes"),
                c("Sift the rubble", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Stones(6), fx_log("Ash, a bent nail, and six primeval stones.".to_string())],
                    "ashes"),
                c("Sleep by the embers", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull, fx_log("You sleep one eye open. The fire keeps watch.".to_string())],
                    "ashes"),
            ],
        },
        StoryNode {
            id: "clearing",
            title: "The Clearing",
            body: "Beyond the ridge the smoke thins into a pine clearing where Old Chen — mortal, sixty, missing two fingers — boils bark tea over a stone stove like the world isn't ending. Refugees huddle under oiled tarps. He looks at the Gu riding your shoulder and goes very still.\n\n\"So the worms choose now,\" he mutters. \"Listen, child. Three banners were seen on Stonebridge road this morning. Heavenly Sword takes disciples. Blood Sea takes... volunteers. The Refinement Hall takes anyone who can carry coal. Choose later. First, eat.\"",
            ending: false,
            choices: vec![
                c("Ask of the three sects", ChoiceKind::Special, true, Req::free(),
                    vec![fx_log("Chen maps the sects in the dirt with a stick.".to_string())],
                    "sects_gate"),
                c("Practice forms", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(30), fx_log("Stances until your legs give out.".to_string())],
                    "clearing"),
                c("Hunt rabbits", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(45), StoryEffect::Stones(4)],
                    "clearing"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "clearing"),
            ],
        },
        StoryNode {
            id: "cinder_out",
            title: "The Scorched Trail",
            body: "The raiders' trail runs east through blackened millet — bootprints, cart ruts, and here and there a dropped primeval stone some nervous recruit fumbled in the dark. You gather what they were too hurried to miss. Far ahead, beyond the ash fields, Stonebridge's lanterns burn like a second, smaller fire.\n\nThe smoke has thinned. There is nothing more to learn from footprints.",
            ending: false,
            choices: vec![
                c("Gather dropped stones", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Stones(7)],
                    "cinder_out"),
                c("Turn back to the clearing", ChoiceKind::Special, true, Req::free(),
                    vec![fx_log("You memorize the trail, then walk home to the living.".to_string())],
                    "clearing"),
            ],
        },
        StoryNode {
            id: "sects_gate",
            title: "Three Banners Over Stonebridge",
            body: "Stonebridge at dusk: three recruiting pavilions around the dry fountain, three kinds of silence. Heavenly Sword's disciples stand in white rows, swords sheathed, eyes forward. Blood Sea's recruiter smiles too much beside a cage of wailing mortal 'volunteers'. The Refinement Hall's furnace-master just shouts for strong backs and pays in stones on the spot.\n\nOld Chen was right. Everyone is hiring. Nobody is kind.",
            ending: false,
            choices: vec![
                c("Join Heavenly Sword", ChoiceKind::Special, true, Req::free(),
                    vec![StoryEffect::Sect(Some("Heavenly Sword".to_string())), fx_flag("sect_joined"), fx_log("White robes. Cold mornings. A sword with your name coming.".to_string())],
                    "sword1"),
                c("Join Blood Sea", ChoiceKind::Special, true,
                    Req { kills: 30, ..Req::free() },
                    vec![StoryEffect::Sect(Some("Blood Sea".to_string())), fx_flag("sect_joined"), fx_log("The recruiter's smile widens. The cage has room.".to_string())],
                    "blood1"),
                c("Join Blood Sea (blood calls to blood)", ChoiceKind::Special, true,
                    Req { path: Some(7), ..Req::free() },
                    vec![StoryEffect::Sect(Some("Blood Sea".to_string())), fx_flag("sect_joined"), fx_log("Your Gu hums. The recruiter bows, actually bows.".to_string())],
                    "blood1"),
                c("Join Refinement Hall", ChoiceKind::Special, true,
                    r(4, 0, 0, 0, 0, 0),
                    vec![StoryEffect::Sect(Some("Refinement Hall".to_string())), fx_flag("sect_joined"), fx_log("Coal dust, furnace heat, and honest pay.".to_string())],
                    "refine1"),
                c("Remain sectless", ChoiceKind::Special, true, Req::free(),
                    vec![fx_flag("rogue"), fx_log("No robes. No masters. No leash.".to_string())],
                    "rogue1"),
            ],
        },
        // ---- Heavenly Sword chain ----
        StoryNode {
            id: "sword1",
            title: "Outer Disciple",
            body: "Dawn drills. Cold rice. A senior sister named Lin who corrects your stance by hitting it with a stick. Heavenly Sword does not coddle: the outer courtyard is fifty mortals and twenty slots, and the mountain only feeds those who climb.\n\n\"Sweep the courtyard,\" Lin says, handing you a broom the size of a spear. \"The sect notices the diligent. It also notices the talented. Be both.\"",
            ending: false,
            choices: vec![
                c("Sweep the courtyard", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Contrib(1), StoryEffect::Stones(4), fx_log("Clean stones, noticed diligence.".to_string())],
                    "sword1"),
                c("Spar at dawn (5 STAM — one foe, victor's purse)", ChoiceKind::Repeatable, false, r(0, 5, 0, 0, 0, 0),
                    vec![StoryEffect::Spar { foe: "Senior Sister Lin", stones: 40, xp: 90, rank: 2 }],
                    "sword1"),
                c("Enter the testing ground", ChoiceKind::Special, true, r(5, 0, 0, 0, 0, 0),
                    vec![fx_flag("trial_sword"), StoryEffect::StartArena, fx_log("The testing ground is a real wilds, stocked with real teeth.".to_string())],
                    "sword2"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "sword1"),
            ],
        },
        StoryNode {
            id: "sword2",
            title: "Sword Testing Ground",
            body: "You come back from the wilds with someone else's blood drying on your sleeve and your own Gu fat and humming. Lin looks at you for a long moment, then nods once — the highest praise the outer sect offers.\n\n\"The inner sect examines candidates at the full moon,\" she says. \"Show them a Gu worth feeding, and cultivation worth funding.\"",
            ending: false,
            choices: vec![
                c("Drill forms", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(70)],
                    "sword2"),
                c("Report for examination", ChoiceKind::Special, true, r(8, 0, 0, 0, 0, 0),
                    vec![StoryEffect::GrantGuRank(2), fx_flag("inner"), fx_log("The elders nod. An inner robe, and a Gu from the vault.".to_string())],
                    "sword3"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "sword2"),
            ],
        },
        StoryNode {
            id: "sword3",
            title: "Inner Sect",
            body: "Inner robes are heavier than they look — mostly with other people's expectations. You have a stipend now, a scripture shelf, and a standing invitation to bleed in the sect tournament every season. The elders speak of war coming down from the north like weather.\n\nLin, now your martial aunt in all but name, sharpens her sword each night and says nothing.",
            ending: false,
            choices: vec![
                c("Read scripture", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(90)],
                    "sword3"),
                c("Serve the sect", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Contrib(1), StoryEffect::Stones(6)],
                    "sword3"),
                c("Enter the tournament", ChoiceKind::Special, true, r(12, 2, 0, 0, 0, 0),
                    vec![StoryEffect::GrantGuRank(3), StoryEffect::StartArena, fx_log("The tournament ring is just the wilds with witnesses.".to_string())],
                    "sword3"),
                c("Take the core oath", ChoiceKind::Special, true,
                    Req { min_level: 15, flag: Some("inner"), ..Req::free() },
                    vec![fx_flag("sword_oath"), fx_log("You swear on your Gu. It listens. That is the frightening part.".to_string())],
                    "sword3"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "sword3"),
            ],
        },
        // ---- Blood Sea chain ----
        StoryNode {
            id: "blood1",
            title: "The Red Baptism",
            body: "Blood Sea does not recruit. It collects. Your first night, they lock you in a cellar with a starving prisoner and one knife, and open the door at dawn to see which of you walks out. You walk out. Nobody congratulates you. That, you slowly understand, IS the congratulations.\n\nYour handler is a smiling woman called Red Aunt. She grades your work in nods.",
            ending: false,
            choices: vec![
                c("Bleed the captives", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(55), StoryEffect::Stones(6), fx_log("The cellar learns your footsteps.".to_string())],
                    "blood1"),
                c("Fight in the cellar pit (5 STAM — one foe, victor's purse)", ChoiceKind::Repeatable, false, r(0, 5, 0, 0, 0, 0),
                    vec![StoryEffect::Spar { foe: "Starved Veteran", stones: 40, xp: 90, rank: 2 }],
                    "blood1"),
                c("Drink with Red Aunt", ChoiceKind::Special, true, r(6, 0, 0, 0, 0, 0),
                    vec![fx_flag("aunt"), StoryEffect::GrantGuRank(2), fx_log("She laughs, pours, and slides a Gu across the table. 'Grow up hungry.'".to_string())],
                    "blood2"),
                c("Sleep (one eye open)", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "blood1"),
            ],
        },
        StoryNode {
            id: "blood2",
            title: "Blood Pool",
            body: "The pool is exactly what it sounds like, and it works exactly the way the songs say: you go in hollow and come out humming with stolen vitality. Red Aunt watches you climb out and stops smiling, which from her is practically a bow.\n\n\"The Asura rite needs a hundred kills under your belt and a spine that doesn't bend,\" she says. \"You have one of those.\"",
            ending: false,
            choices: vec![
                c("Bathe again", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::HealFull, StoryEffect::Xp(40)],
                    "blood2"),
                c("Take the Asura rite", ChoiceKind::Special, true,
                    Req { min_level: 12, kills: 100, ..Req::free() },
                    vec![fx_flag("asura"), StoryEffect::GrantGuRank(3), StoryEffect::StartArena, fx_log("The rite is a hunt with witnesses.".to_string())],
                    "blood3"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "blood2"),
            ],
        },
        StoryNode {
            id: "blood3",
            title: "Asura Candidate",
            body: "They paint your face with the blood of something that almost killed you, and the sect hall goes quiet when you enter now. Asura candidacy means two things: everyone weaker fears you, and everyone stronger is measuring you for a coffin.\n\nRed Aunt leaves a northern war-map on your table, weighted with a dagger. An invitation, or a warning. With her, always both.",
            ending: false,
            choices: vec![
                c("Harvest the border villages", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(90), StoryEffect::Stones(10)],
                    "blood3"),
                c("Serve the sect", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Contrib(1), StoryEffect::Stones(6)],
                    "blood3"),
                c("Study the war-map", ChoiceKind::Special, true, r(15, 0, 0, 0, 0, 0),
                    vec![fx_flag("knows_war"), fx_log("Troop marks, supply lines, a date circled in red.".to_string())],
                    "war"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "blood3"),
            ],
        },
        // ---- Refinement Hall chain ----
        StoryNode {
            id: "refine1",
            title: "Apprentice",
            body: "The Thousand Refinement Hall smells of coal smoke, hot metal, and money. No oaths, no baptisms — the furnace-master, a woman built like a bellows named Granny Iron, points at a coal pile the size of a house and says: 'Shovel. We pay by the barrow. cultivators who can't work don't eat.'\n\nIt is, you realize with something like relief, honest.",
            ending: false,
            choices: vec![
                c("Shovel coal", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Contrib(1), StoryEffect::Stones(8), fx_log("Honest soot, honest pay.".to_string())],
                    "refine1"),
                c("Watch the refiners", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(50)],
                    "refine1"),
                c("Ask for journeyman work", ChoiceKind::Special, true, r(5, 0, 0, 0, 0, 0),
                    vec![fx_flag("journeyman"), fx_log("Granny Iron looks you up and down and grunts. Promoted.".to_string())],
                    "refine2"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "refine1"),
            ],
        },
        StoryNode {
            id: "refine2",
            title: "Journeyman",
            body: "Journeyman means your own corner of furnace-floor, your own tongs, and the right to bid on dud Gu the masters won't waste essence finishing. Most are junk. Some just need a steadier hand than their refiner had.\n\nGranny Iron leaves failed commissions on your bench 'by accident' with increasing frequency. It is the closest thing to affection she owns.",
            ending: false,
            choices: vec![
                c("Finish a dud commission", ChoiceKind::Repeatable, false, r(0, 2, 0, 20, 0, 0),
                    vec![StoryEffect::Xp(70), StoryEffect::Stones(12), fx_log("It holds. It actually holds.".to_string())],
                    "refine2"),
                c("Commission a rank-3 Gu", ChoiceKind::Special, true,
                    Req { min_level: 8, stones: 100, ..Req::free() },
                    vec![StoryEffect::Stones(-100), StoryEffect::GrantGuRank(3), fx_log("Three weeks of furnace light. Worth it.".to_string())],
                    "refine3"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "refine2"),
            ],
        },
        StoryNode {
            id: "refine3",
            title: "Master",
            body: "Master of the third furnace. Your name is chalked on the commission board in Granny Iron's own hand, and merchants ask for you by reputation. The Hall pays in stones, in favors, and — for masters — in first refusal on wild Gu the caravans bring down from the north.\n\nWar, the caravans say, is coming down from the north like weather.",
            ending: false,
            choices: vec![
                c("Take commissions", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(90), StoryEffect::Stones(14)],
                    "refine3"),
                c("Refine three rank-1 worms (~15 stones)", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Refine { rank: 1 }],
                    "refine3"),
                c("Refine three rank-2 worms (~30 stones)", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Refine { rank: 2 }],
                    "refine3"),
                c("Refine three rank-3 worms (~45 stones)", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Refine { rank: 3 }],
                    "refine3"),
                c("Refine three rank-4 worms (~60 stones)", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Refine { rank: 4 }],
                    "refine3"),
                c("Refine three rank-5 worms (~75 stones)", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Refine { rank: 5 }],
                    "refine3"),
                c("Serve the hall", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Contrib(1), StoryEffect::Stones(6)],
                    "refine3"),
                c("Read the caravan news", ChoiceKind::Special, true, r(15, 0, 0, 0, 0, 0),
                    vec![fx_flag("knows_war"), fx_log("Troop marks, supply lines, a date circled in red.".to_string())],
                    "war"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "refine3"),
            ],
        },
        // ---- Rogue chain ----
        StoryNode {
            id: "rogue1",
            title: "No Sect, No Master",
            body: "No robes. No stipend. No one to bow to and no one watching your back — the accounts balance exactly the way rogues like. You sleep in haylofts and eat what the road forgets to guard, and your Gu grows fat on scraps and spite.\n\nIn Stonebridge's back alleys, a fence with one milky eye sells information by weight. His first free sample: 'War's coming. Rogues who aren't ready get harvested with the wheat.'",
            ending: false,
            choices: vec![
                c("Run messages", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Stones(6), StoryEffect::Xp(30)],
                    "rogue1"),
                c("Rob the tax cart", ChoiceKind::Repeatable, false, r(4, 2, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(70), StoryEffect::Stones(16), fx_log("The guards never even saw the cart rock.".to_string())],
                    "rogue1"),
                c("Knife-duel behind the tannery (5 STAM — one foe, victor's purse)", ChoiceKind::Repeatable, false, r(0, 5, 0, 0, 0, 0),
                    vec![StoryEffect::Spar { foe: "Tannery Knife", stones: 45, xp: 85, rank: 2 }],
                    "rogue1"),
                c("Ask about the black market", ChoiceKind::Special, true, r(5, 0, 0, 0, 0, 0),
                    vec![fx_flag("market"), fx_log("Down the dry well, knock twice, mention milky-eye.".to_string())],
                    "rogue2"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "rogue1"),
            ],
        },
        StoryNode {
            id: "rogue2",
            title: "The Black Market",
            body: "Down the dry well, past a door that isn't there, the black market hums like a second heart under Stonebridge. Everything is for sale: stolen Gu still warm from their owners, maps of sect patrols, poisons with handwritten apologies. The milky-eyed fence nods at you from behind a cage of dream butterflies.\n\n'Rogue discount,' he lies, pleasantly.",
            ending: false,
            choices: vec![
                c("Buy a caged Gu (80 stones)", ChoiceKind::Repeatable, false,
                    Req { stones: 80, ..Req::free() },
                    vec![StoryEffect::Stones(-80), StoryEffect::GrantGuRank(3), fx_log("It bites the cage, then your finger, then settles. Yours.".to_string())],
                    "rogue2"),
                c("Sell patrol schedules", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::Stones(18), StoryEffect::Xp(50)],
                    "rogue2"),
                c("Melt three rank-1 worms in a stolen cauldron (~15 stones)", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Refine { rank: 1 }],
                    "rogue2"),
                c("Melt three rank-2 worms in a stolen cauldron (~30 stones)", ChoiceKind::Repeatable, false, r(0, 1, 0, 0, 0, 0),
                    vec![StoryEffect::Refine { rank: 2 }],
                    "rogue2"),
                c("Hear the war talk", ChoiceKind::Special, true, r(15, 0, 0, 0, 0, 0),
                    vec![fx_flag("knows_war"), fx_log("Everyone is buying weapons. Everyone.".to_string())],
                    "war"),
                c("Become nobody", ChoiceKind::Special, true, r(10, 0, 0, 0, 0, 0),
                    vec![fx_flag("nameless"), fx_log("You stop giving your name. The alleys start giving you room.".to_string())],
                    "rogue3"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "rogue2"),
            ],
        },
        StoryNode {
            id: "rogue3",
            title: "Nameless",
            body: "Somewhere along the road you stopped giving your name, because names are handles and handles are for grabbing. The alleys call you Nameless now, and mean it as respect. You have survived every sect's attention by being exactly unimportant enough to ignore and exactly dangerous enough to regret.\n\nIt is, all things considered, an excellent way to live. It cannot last.",
            ending: false,
            choices: vec![
                c("Take quiet contracts", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(90), StoryEffect::Stones(14)],
                    "rogue3"),
                c("Howl at the moon", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::HealFull, fx_log("The moon, professional that it is, says nothing.".to_string())],
                    "rogue3"),
            ],
        },
        // ---- Convergence: war ----
        StoryNode {
            id: "war",
            title: "The Righteous-Demonic War",
            body: "It starts the way these things always start: with a border village that has a name on no map that matters, burning on a Tuesday. Within a month both hosts are in the field, and Stonebridge sits directly between them like a coin on a chopping block.\n\nEvery banner calls. Every debt comes due. The choices you made getting here decide which doors are open — and which are barricaded.",
            ending: false,
            choices: vec![
                c("March with your sect", ChoiceKind::Special, true,
                    Req { min_level: 16, flag: Some("sect_joined"), ..Req::free() },
                    vec![fx_flag("war_side"), StoryEffect::StartArena, fx_log("Banners up. Drums. The field smells of rain and iron.".to_string())],
                    "war2"),
                c("Sell your sword (rogue)", ChoiceKind::Special, true,
                    Req { min_level: 16, flag: Some("rogue"), ..Req::free() },
                    vec![fx_flag("war_side"), StoryEffect::Stones(60), StoryEffect::StartArena, fx_log("Highest bidder, shortest leash. The rogue way.".to_string())],
                    "war2"),
                c("Drill for the campaign", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(110)],
                    "war"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "war"),
            ],
        },
        StoryNode {
            id: "war2",
            title: "Siege of Stonebridge",
            body: "Stonebridge burns politely, district by district, while both armies pretend the granaries are the objective and everyone knows the objective is the primeval spring underneath. You fight in streets you once begged in. The fountain plaza runs red, then runs clear, then runs red again.\n\nWhen it ends — when it pauses, wars never end — the survivors look at each other like strangers who share a secret.",
            ending: false,
            choices: vec![
                c("Charge the gates", ChoiceKind::Repeatable, false, r(0, 2, 0, 0, 0, 0),
                    vec![StoryEffect::StartArena, StoryEffect::Xp(60), fx_log("Another push. Another street.".to_string())],
                    "war2"),
                c("Carry the summit invitation", ChoiceKind::Special, true, r(20, 0, 0, 0, 0, 0),
                    vec![StoryEffect::GrantGuRank(4), fx_flag("summoned"), fx_log("A lotus-sealed letter, addressed to you by name. The Heavenly Gathering wants witnesses.".to_string())],
                    "summit"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "war2"),
            ],
        },
        StoryNode {
            id: "summit",
            title: "Heavenly Gathering",
            body: "Above the clouds, on a platform of white jade that has no business existing, the great powers gather to decide what the next century tastes like. You are here because you survived everything they threw at the world below — that, it turns out, is the entire entrance exam.\n\nAncient eyes measure you. Four futures unfold like fans. The mountain holds its breath.",
            ending: false,
            choices: vec![
                c("Ascend (Lv 35 + Immortal Gu)", ChoiceKind::Special, true,
                    Req { min_level: 35, gu_rank: 6, ..Req::free() },
                    vec![],
                    "end_ascend"),
                c("Claim the demon throne (Blood + 250 kills)", ChoiceKind::Special, true,
                    Req { sect: Some("Blood Sea"), kills: 250, ..Req::free() },
                    vec![],
                    "end_demon"),
                c("Dream the eternal dream", ChoiceKind::Special, true,
                    Req { path: Some(11), min_level: 25, ..Req::free() },
                    vec![],
                    "end_dream"),
                c("Become sect patriarch", ChoiceKind::Special, true,
                    Req { min_level: 20, contrib: 10, ..Req::free() },
                    vec![],
                    "end_patriarch"),
                c("Meditate on the clouds", ChoiceKind::Repeatable, false, r(0, 3, 0, 0, 0, 0),
                    vec![StoryEffect::Xp(150)],
                    "summit"),
                c("Sleep", ChoiceKind::Repeatable, false, Req::free(),
                    vec![StoryEffect::Sleep, StoryEffect::HealFull],
                    "summit"),
            ],
        },
        // ---- Endings ----
        StoryNode {
            id: "end_ascend",
            title: "ENDING: Ascension",
            body: "The tribulation lightning comes down like a judgment you already appealed — and lost, and appealed again, for five hundred years of mornings exactly like this one. Your Immortal Gu unfolds around you, rank-six wings beating once, twice, and the sky simply... accepts it.\n\nBelow, Qing Mao mountain greens over the ash. Someone will dig up your story and get it wrong in all the interesting ways. You find, to your surprise, that you don't mind. The heavens are large. There is so much left to steal from them.",
            ending: true,
            choices: vec![
                c("Reincarnate", ChoiceKind::Special, true, Req::free(),
                    vec![StoryEffect::Reincarnate { bonus: true }, fx_log("The wheel turns. You keep the lessons.".to_string())],
                    "ashes"),
            ],
        },
        StoryNode {
            id: "end_demon",
            title: "ENDING: Demon Sovereign",
            body: "Two hundred and fifty names. You remember every one — that is the point, that is the price, that is the throne. The Blood Sea Sect kneels in a red tide, and Red Aunt, smiling that terrible smile, is the first to bend the knee.\n\nRighteous cultivators will tell stories about you to frighten children for a thousand years. The children, being children, will want to grow up just like you.",
            ending: true,
            choices: vec![
                c("Reincarnate", ChoiceKind::Special, true, Req::free(),
                    vec![StoryEffect::Reincarnate { bonus: true }, fx_log("Even sovereigns get bored. Again.".to_string())],
                    "ashes"),
            ],
        },
        StoryNode {
            id: "end_dream",
            title: "ENDING: Eternal Dream",
            body: "One morning you simply stop waking up — and discover you never needed to. The dream is larger than the waking world and kinder in the ways that matter: here the village never burned, the Gu sing instead of bite, and every choice you didn't make is a door you can still open.\n\nReality knocks sometimes. You never answer. Dream Path cultivators call this the only honest victory.",
            ending: true,
            choices: vec![
                c("Reincarnate", ChoiceKind::Special, true, Req::free(),
                    vec![StoryEffect::Reincarnate { bonus: true }, fx_log("Even dreams end. Especially dreams.".to_string())],
                    "ashes"),
            ],
        },
        StoryNode {
            id: "end_patriarch",
            title: "ENDING: Sect Patriarch",
            body: "They give you the seal, the mountain, and the headaches that come with both. Under your hand the sect prospers: granaries full, disciples disciplined, furnaces never cold. Old Chen — impossibly, stubbornly alive — visits once a year to drink your tea and tell you you're doing it wrong.\n\nPower, it turns out, was never the tribulation. Paperwork was.",
            ending: true,
            choices: vec![
                c("Reincarnate", ChoiceKind::Special, true, Req::free(),
                    vec![StoryEffect::Reincarnate { bonus: true }, fx_log("The seal passes on. The wheel turns.".to_string())],
                    "ashes"),
            ],
        },
    ]
}

pub fn find_node<'a>(nodes: &'a [StoryNode], id: &str) -> Option<&'a StoryNode> {
    nodes.iter().find(|n| n.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_graph_holds() {
        let nodes = story_nodes();
        assert!(nodes.len() >= 20, "want a long saga, have {}", nodes.len());
        // every goto lands somewhere; every non-ending node offers a way on
        for n in nodes.iter() {
            assert!(!n.choices.is_empty(), "dead node: {}", n.id);
            for ch in n.choices.iter() {
                assert!(
                    nodes.iter().any(|m| m.id == ch.goto),
                    "dangling choice '{}' -> '{}'",
                    ch.label,
                    ch.goto
                );
            }
        }
        // start exists, four endings exist and only reincarnate out
        let start = find_node(&nodes, "ashes").expect("no start node");
        assert!(!start.ending);
        let mut endings = 0;
        for n in nodes.iter() {
            if n.ending {
                endings += 1;
                assert_eq!(n.choices.len(), 1, "ending {} must funnel", n.id);
            }
        }
        assert_eq!(endings, 4, "want exactly 4 endings");
        // every node reachable from the start (BFS over all gotos)
        let mut seen = BTreeSet::new();
        let mut stack = vec!["ashes"];
        while let Some(id) = stack.pop() {
            if !seen.insert(id.to_string()) {
                continue;
            }
            if let Some(n) = find_node(&nodes, id) {
                for ch in n.choices.iter() {
                    stack.push(ch.goto);
                }
            }
        }
        for n in nodes.iter() {
            assert!(seen.contains(n.id), "unreachable node: {}", n.id);
        }
    }
}
