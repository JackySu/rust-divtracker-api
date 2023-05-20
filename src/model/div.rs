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
    pub main_story: String,
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
    pub total_playtime: u64,
    pub level: u64,
    pub pvp_kills: u64,
    pub npc_kills: u64,
    pub headshots: u64,
    pub headshot_kills: u64,
    pub shotgun_kills: u64,
    pub smg_kills: u64,
    pub pistol_kills: u64,
    pub rifle_kills: u64,
    pub player_kills: u64,
    pub xp_total: u64,
    pub pve_xp: u64,
    pub pvp_xp: u64,
    pub clan_xp: u64,
    pub sharpshooter_kills: u64,
    pub survivalist_kills: u64,
    pub demolitionist_kills: u64,
    pub e_credit: u64,
    pub commendation_count: u64,
    pub commendation_score: u64,
    pub gear_score: u64,
    pub dz_rank: u64,
    pub dz_playtime: u64,
    pub rogues_killed: u64,
    pub rogue_playtime: u64,
    pub longest_rogue: u64,
    pub conflict_rank: u64,
    pub conflict_playtime: u64,
    pub all_names: Vec<String>,
}

