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
pub struct ProfileDTO {
    pub id: String,
    pub name: String,
}