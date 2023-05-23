use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use reqwest::{self, header::HeaderValue};
use thirtyfour::prelude::*;
use serde_json::{from_str, Value};
use sqlx::{Pool, Sqlite};
use std::sync::Mutex;

use futures::{future::join_all, StreamExt};

use crate::db::user::{create_user, get_user_id_by_name, get_user_names_by_id, store_user_name};
use crate::model::div::{D1PlayerStats, D2PlayerStats};
use crate::model::ubi::{ProfileDTO, StatsDTO};
use crate::util;

lazy_static! {
    static ref UBI_TICKET: Mutex<String> = Mutex::new("".to_string());
    static ref UBI_SESSION_ID: Mutex<String> = Mutex::new("".to_string());
    static ref UBI_EXPIRATION: Mutex<String> =
        Mutex::new("2015-11-12T00:00:00.0000000Z".to_string());
}

pub static UBI_LOGIN_URL: &str = "https://public-ubiservices.ubi.com/v3/profiles/sessions";
pub async fn login_ubi() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = util::header::get_common_header().await;

    let userpass = format!(
        "{}:{}",
        std::env::var("UBI_USERNAME").expect("UBI_USERNAME not set"),
        std::env::var("UBI_PASSWORD").expect("UBI_PASSWORD not set")
    );
    let auth = base64::encode(userpass.as_bytes()).as_str().to_owned();
    headers.insert("Authorization", format!("Basic {}", auth).parse().unwrap());

    let resp = reqwest::Client::new()
        .post(UBI_LOGIN_URL)
        .headers(headers)
        .send()
        .await?
        .json::<Value>()
        .await?;

    if !resp["errorCode"].is_null() {
        println!("{:#?}", resp);
        return Err("Failed to login to Ubi".into());
    }

    let mut ticket = UBI_TICKET.lock().unwrap();
    *ticket = resp["ticket"].as_str().unwrap().to_string();

    let mut session_id = UBI_SESSION_ID.lock().unwrap();
    *session_id = resp["sessionId"].as_str().unwrap().to_string();

    let mut expiration = UBI_EXPIRATION.lock().unwrap();
    *expiration = resp["expiration"].as_str().unwrap().to_string();

    Ok(())
}

pub async fn find_player_id_by_db(
    pool: &Pool<Sqlite>,
    name: &str,
) -> Result<Vec<ProfileDTO>, Box<dyn std::error::Error>> {
    let ids = match get_user_id_by_name(pool, name).await {
        Ok(id) => id,
        Err(_) => return Err(format!("Failed to find player {} in db", name).into()),
    };
    let mut profiles = vec![];
    for id in ids {
        profiles.push(ProfileDTO { id: id, name: None });
    }
    Ok(profiles)
}

pub async fn find_player_id_by_api(
    name: &str,
) -> Result<Vec<ProfileDTO>, Box<dyn std::error::Error>> {
    let expiration = UBI_EXPIRATION.lock().unwrap().clone();
    let mut exp = DateTime::parse_from_rfc3339(&expiration)
        .unwrap()
        .with_timezone(&Utc);
    let mut now = Utc::now();

    let mut login_counts = 0;
    while exp < now && login_counts < 5 {
        login_ubi().await?;
        login_counts += 1;
        println!("Renewed Ubi ticket at {}", now.to_rfc3339());
        let expiration = UBI_EXPIRATION.lock().unwrap().clone();
        exp = DateTime::parse_from_rfc3339(&expiration)
            .unwrap()
            .with_timezone(&Utc);
        now = Utc::now();
    }
    if login_counts >= 5 {
        return Err("Failed to login after 5 trials".into());
    }

    let ticket = UBI_TICKET.lock().unwrap().clone();
    let mut headers = util::header::get_common_header().await;
    headers.insert(
        "Authorization",
        format!("Ubi_v1 t={}", &*ticket).parse().unwrap(),
    );

    let session_id = UBI_SESSION_ID.lock().unwrap().clone();
    headers.insert(
        "Ubi-SessionId",
        (*session_id).parse::<HeaderValue>().unwrap(),
    );

    let url = format!(
        "https://public-ubiservices.ubi.com/v2/profiles?nameOnPlatform={}&platformType=uplay",
        name
    );
    let resp = reqwest::Client::new()
        .get(&url)
        .headers(headers)
        .send()
        .await?
        .json::<Value>()
        .await?;

    let profiles = &resp["profiles"];
    if profiles.is_array() && profiles.as_array().unwrap().is_empty() {
        return Err(format!("Failed to find player {} by api", name).into());
    }

    Ok(profiles
        .as_array()
        .unwrap()
        .into_iter()
        .map(|p| ProfileDTO {
            id: p["profileId"].as_str().unwrap().to_string(),
            name: Some(p["nameOnPlatform"].as_str().unwrap().to_string()),
        })
        .collect::<Vec<ProfileDTO>>())
}

pub async fn get_player_profiles_by_name(
    pool: &Pool<Sqlite>,
    name: &str,
) -> Result<Vec<ProfileDTO>, Box<dyn std::error::Error>> {
    let mut profiles = match find_player_id_by_api(name).await {
        Ok(profiles) => profiles,
        Err(_) => vec![],
    };
    if profiles.is_empty() {
        profiles = find_player_id_by_db(pool, name).await?;
    }
    Ok(profiles)
}

pub async fn get_player_stats_by_name(
    pool: &Pool<Sqlite>,
    name: &str,
    game_space_id: &str,
) -> Result<Vec<StatsDTO>, Box<dyn std::error::Error>> {
    let mut headers = util::header::get_common_header().await;
    let ticket = UBI_TICKET.lock().unwrap().clone();
    headers.insert(
        "Authorization",
        format!("Ubi_v1 t={}", &ticket).parse().unwrap(),
    );

    let session_id = UBI_SESSION_ID.lock().unwrap().clone();
    headers.insert(
        "Ubi-SessionId",
        (*session_id).parse::<HeaderValue>().unwrap(),
    );

    let mut profiles = get_player_profiles_by_name(pool, name).await?;

    let mut results: Vec<StatsDTO> = vec![];
    let urls = profiles
        .iter()
        .map(|p| {
            let url = format!(
                "https://public-ubiservices.ubi.com/v1/profiles/{}/statscard?spaceId={}",
                p.id, game_space_id
            );
            url
        })
        .collect::<Vec<String>>();

    let client = reqwest::Client::new();
    let stream =
        futures::stream::iter(urls).map(|url| client.get(&url).headers(headers.clone()).send());

    let mut stream = stream.buffered(5);

    let mut i = 0;
    while let Some(result) = stream.next().await {
        let resp = result?.json::<Value>().await?;
        if !resp["errorCode"].is_null() {
            println!("{:#?}", resp);
            return Err("Failed to login to Ubi".into());
        }
        let profile = &mut profiles[i];
        match create_user(&pool, &profile.id).await {
            Ok(_) => println!("Created or update user {}", &profile.id),
            Err(e) => {
                println!("Failed to create or update user {}: {:?}", &profile.id, e)
            }
        }
        let name = match &profile.name {
            Some(n) => (*n).clone(),
            None => {
                println!("Failed to get name for user {}", &profile.id);
                let url = format!(
                    "https://public-ubiservices.ubi.com/v2/profiles?userId={}&platformType=uplay",
                    &profile.id
                );
                let res = reqwest::Client::new()
                    .get(&url)
                    .headers(headers.clone())
                    .send()
                    .await?
                    .json::<Value>()
                        .await?["profiles"][0]["nameOnPlatform"]
                        .as_str()
                        .unwrap()
                        .to_string();
                profile.name = Some(res.clone());
                res
            }
        };

        match store_user_name(&pool, &profile.id, &name).await {
            Ok(_) => println!("Stored name {} for user {}", &name, &profile.id),
            Err(e) => {
                println!(
                    "Failed to store name {} for user {}: {:?}",
                    &name, &profile.id, e
                );
            }
        }
        results.push(StatsDTO {
            stats: resp["Statscards"].as_array().unwrap().clone(),
            profile: profile.clone(),
        });
        i += 1;
    }

    if results.is_empty() {
        return Err(format!("Failed to find player {} by either api or db", name).into());
    }
    Ok(results)
}

pub static DIV1_SPACE_ID: &str = "6edd234a-abff-4e90-9aab-b9b9c6e49ff7";
pub async fn get_div1_player_stats(
    pool: &Pool<Sqlite>,
    name: &str,
) -> Result<Vec<D1PlayerStats>, Box<dyn std::error::Error>> {
    let res = get_player_stats_by_name(pool, name, DIV1_SPACE_ID).await?;
    Ok(join_all(
        res.into_iter()
            .map(|r| async move {
                let p = r.profile;
                let s = r.stats;
                D1PlayerStats {
                    id: p.id.clone(),
                    name: p.name.unwrap_or("".to_string()),
                    level: s[0]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
                    dz_rank: s[1]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
                    ug_rank: s[2]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
                    playtime: s[3]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0) / 3600,
                    main_story: s[4]["value"].as_str().unwrap_or("0 %").to_string(),
                    rogue_kills: s[5]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
                    items_extracted: s[6]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
                    skill_kills: s[7]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
                    total_kills: s[8]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
                    gear_score: s[11]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
                    all_names: get_user_names_by_id(pool, p.id.clone().as_str())
                        .await
                        .unwrap_or(vec![]),
                }
            })
            .collect::<Vec<_>>(),
    )
    .await)
}

// pub static DIV2_SPACE_ID: &str = "60859c37-949d-49e2-8fc8-6d8dc40f1a9e";
pub static TRACKER_URL: &str = "https://api.tracker.gg/api/v2/division-2/standard/profile/uplay/";
pub async fn get_div2_player_stats(
    pool: &Pool<Sqlite>,
    name: &str,
) -> Result<Vec<D2PlayerStats>, Box<dyn std::error::Error>> {
    let mut profiles = match find_player_id_by_api(&name).await {
        Ok(profiles) => profiles,
        Err(_) => vec![],
    };

    if profiles.is_empty() {
        profiles = find_player_id_by_db(pool, &name).await?;
        if profiles.is_empty() {
            return Err(format!("Failed to find player {} by either api or db", name).into());
        }
    } else {
        match create_user(&pool, &profiles[0].id).await {
            Ok(_) => println!("Created or update user {}", &profiles[0].id),
            Err(e) => {
                println!("Failed to create or update user {}: {:?}", &profiles[0].id, e)
            }
        }
    }

    let p = &profiles[0];
    let p_name = p.name.clone().unwrap();
    match store_user_name(&pool, &p.id, &p_name).await {
        Ok(_) => println!("Stored name {} for user {}", &p_name, &p.id),
        Err(e) => {
            println!(
                "Failed to store name {} for user {}: {:?}",
                &name, &p.id, e
            );
        }
    }

    let driver = util::webdriver::get_webdriver().await?;
    driver.goto(format!("{}{}", TRACKER_URL, p.name.clone().unwrap_or("".to_string()))).await.unwrap();
    let data = driver.find(By::Css("body")).await?.text().await?;
    driver.quit().await?;

    let metadata: Value = from_str(&data)?;
    let stats = &metadata["data"]["segments"][0]["stats"];
    
    if stats.is_null() {
        return Err(format!("player {} exists but no profile for this game", name).into());
    }
    Ok(vec![D2PlayerStats {
        id: p.id.clone(),
        name: p.name.clone().unwrap_or("".to_string()),
        total_playtime: stats["timePlayed"]["value"].as_u64().unwrap_or(0) / 3600,
        level: stats["highestPlayerLevel"]["value"].as_u64().unwrap_or(0),
        pvp_kills: stats["killsPvP"]["value"].as_u64().unwrap_or(0),
        npc_kills: stats["killsNpc"]["value"].as_u64().unwrap_or(0),
        headshots: stats["headshots"]["value"].as_u64().unwrap_or(0),
        headshot_kills: stats["killsHeadshot"]["value"].as_u64().unwrap_or(0),
        shotgun_kills: stats["killsWeaponShotgun"]["value"].as_u64().unwrap_or(0),
        smg_kills: stats["killsWeaponSubMachinegun"]["value"].as_u64().unwrap_or(0),
        pistol_kills: stats["killsWeaponPistol"]["value"].as_u64().unwrap_or(0),
        rifle_kills: stats["killsWeaponRifle"]["value"].as_u64().unwrap_or(0),
        player_kills: stats["playersKilled"]["value"].as_u64().unwrap_or(0),
        xp_total: stats["xPTotal"]["value"].as_u64().unwrap_or(0),
        pve_xp: stats["xPPve"]["value"].as_u64().unwrap_or(0),
        pvp_xp: stats["xPPvp"]["value"].as_u64().unwrap_or(0),
        clan_xp: stats["xPClan"]["value"].as_u64().unwrap_or(0),
        sharpshooter_kills: stats["killsSpecializationSharpshooter"]["value"].as_u64().unwrap_or(0),
        survivalist_kills: stats["killsSpecializationSurvivalist"]["value"].as_u64().unwrap_or(0),
        demolitionist_kills: stats["killsSpecializationDemolitionist"]["value"].as_u64().unwrap_or(0),
        e_credit: stats["eCreditBalance"]["value"].as_u64().unwrap_or(0),
        commendation_count: stats["commendationCount"]["value"].as_u64().unwrap_or(0),
        commendation_score: stats["commendationScore"]["value"].as_u64().unwrap_or(0),
        gear_score: stats["latestGearScore"]["value"].as_u64().unwrap_or(0),
        dz_rank: stats["rankDZ"]["value"].as_u64().unwrap_or(0),
        dz_playtime: stats["timePlayedDarkZone"]["value"].as_u64().unwrap_or(0) / 3600,
        rogues_killed: stats["roguesKilled"]["value"].as_u64().unwrap_or(0),
        rogue_playtime: stats["timePlayedRogue"]["value"].as_u64().unwrap_or(0) / 3600,
        longest_rogue: stats["timePlayedRogueLongest"]["value"].as_u64().unwrap_or(0) / 60,
        conflict_rank: stats["latestConflictRank"]["value"].as_u64().unwrap_or(0),
        conflict_playtime: stats["timePlayedConflict"]["value"].as_u64().unwrap_or(0) / 3600,
        all_names: get_user_names_by_id(pool, profiles[0].id.clone().as_str())
            .await
            .unwrap_or(vec![])
    }])
}
