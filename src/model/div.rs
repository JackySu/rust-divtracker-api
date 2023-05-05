use rocket::serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct D1PlayerStats {
    #[serde(skip_serializing)]
    pub id: String,
    pub name: String,
    pub level: u64,
    pub dz_rank: u64,
    pub ug_rank: u64,
    pub playtime: u64,
    pub main_story: f32,
    pub total_kills: u64,
    pub rogue_kills: u64,
    pub items_extracted: u64,
    pub skill_kills: u64,
    pub gear_score: u64,
    pub all_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct D2PlayerStats {
    #[serde(skip_serializing)]
    pub id: String,
    pub name: String,
    pub pvp_kills: u64,
    pub npc_kills: u64,
    pub headshots: u64,
    pub skill_kills: u64,
    pub items_looted: u64,
    pub longest_rogue: u64,
    pub level: u64,
    pub dz_rank: u64,
    pub white_zone_xp: u64,
    pub dark_zone_xp: u64,
    pub pvp_xp: u64,
    pub clan_xp: u64,
    pub commendation_score: u64,
    pub e_credit: u64,
    pub total_playtime: u64,
    pub dz_playtime: u64,
    pub rogue_playtime: u64,
    pub white_zone_pve_kills: u64,
    pub dark_zone_pve_kills: u64,
    pub total_hits: u64,
    pub crit_hits: u64,
    pub gear_score: u64,
    pub world_tier: String,
    pub conflict_rank: u64,
    pub headshots_hits_ratio: f32,
    pub all_names: Vec<String>,
}

