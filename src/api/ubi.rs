use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use reqwest::{self, header::HeaderValue};
use serde_json::Value;
use sqlx::{pool, Pool, Sqlite};
use std::sync::Mutex;

use futures::{future::join_all, stream, StreamExt, TryStreamExt};

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

pub async fn find_player_id(
    pool: &Pool<Sqlite>,
    name: &str,
) -> Result<Vec<ProfileDTO>, Box<dyn std::error::Error>> {
    let ids = get_user_id_by_name(pool, name).await;

    let mut results: Vec<ProfileDTO> = Vec::new();
    match ids {
        Ok(ids) => {
            if ids.len() > 0 {
                results = ids
                    .iter()
                    .map(|id| ProfileDTO {
                        id: id.to_owned(),
                        name: None,
                    })
                    .collect::<Vec<ProfileDTO>>();
            }
        }
        Err(e) => {
            println!(
                "Failed to get user id by name {} in database: {:?}",
                name, e
            );
        }
    }

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
    if profiles.is_array() && profiles.as_array().unwrap().is_empty() && results.is_empty() {
        return Err(format!("Failed to find player {}", name).into());
    }

    for p in profiles.as_array().unwrap() {
        results.push(ProfileDTO {
            id: p["profileId"].as_str().unwrap().to_string(),
            name: Some(p["nameOnPlatform"].as_str().unwrap().to_string()),
        });
    }
    Ok(results)
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

    let mut profiles = find_player_id(pool, name).await?;
    profiles.dedup_by(|a, b| a.id == b.id);

    let mut results: Vec<StatsDTO> = Vec::new();
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

    let mut stream = stream.buffer_unordered(10);

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
                println!(
                    "Failed to create or update user {}: {:?}", 
                    &profile.id, e
                )
            }
        }
        let name = match &profile.name {
            Some(n) => (*n).clone(),
            None => {
                println!(
                    "Failed to get name for user {}", 
                    &profile.id
                );
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
                    .await?
                        ["profiles"][0]["nameOnPlatform"]
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
                        .unwrap_or(Vec::new()),
                }
            })
            .collect::<Vec<_>>(),
    )
    .await)
}

pub static DIV2_SPACE_ID: &str = "60859c37-949d-49e2-8fc8-6d8dc40f1a9e";
pub async fn get_div2_player_stats(
    pool: &Pool<Sqlite>,
    name: &str,
) -> Result<Vec<D2PlayerStats>, Box<dyn std::error::Error>> {
    let res = get_player_stats_by_name(pool, name, DIV2_SPACE_ID).await?;

    Ok(join_all(res.into_iter().map(|r| async move {
        let s = r.stats;
        let p = r.profile;

        let hs = s[2]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0);
        let hits = s[19]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0);
        let ratio = if hits > 0 {
            hs as f32 / hits as f32
        } else {
            0f32
        };
        D2PlayerStats {
            id: p.id.clone(),
            name: p.name.unwrap_or("".to_string()),
            pvp_kills: s[0]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            npc_kills: s[1]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            headshots: hs,
            skill_kills: s[3]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            items_looted: s[4]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            longest_rogue: s[5]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0) / 60,
            level: s[6]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            dz_rank: s[7]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            white_zone_xp: s[8]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            dark_zone_xp: s[9]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            pvp_xp: s[10]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            clan_xp: s[11]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            commendation_score: s[12]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            e_credit: s[13]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            total_playtime: s[14]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0) / 3600,
            dz_playtime: s[15]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0) / 3600,
            rogue_playtime: s[16]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0) / 3600,
            white_zone_pve_kills: s[17]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            dark_zone_pve_kills: s[18]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            total_hits: hits,
            crit_hits: s[20]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            gear_score: s[21]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            world_tier: s[22]["value"]
                .as_str()
                .unwrap_or("No World Tier")
                .to_string(),
            conflict_rank: s[23]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
            headshots_hits_ratio: ratio,
            all_names: get_user_names_by_id(pool, p.id.clone().as_str())
                .await
                .unwrap_or(Vec::new()),
        }
    }))
    .await)
}
