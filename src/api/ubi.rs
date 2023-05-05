use reqwest::{self, header::HeaderValue};
use serde_json::Value;
use sqlx::{Pool, Sqlite};
use std::sync::Mutex;
use lazy_static::lazy_static;
use chrono::{DateTime, Utc};

use crate::model::div1::{D1PlayerStats, ProfileDTO};
use crate::db::user::{create_user, get_user_names_by_id, store_user_name};
use crate::util;

lazy_static! {
    static ref UBI_TICKET: Mutex<String> = Mutex::new("".to_string());
    static ref UBI_SESSION_ID: Mutex<String> = Mutex::new("".to_string());
    static ref UBI_EXPIRATION: Mutex<String> = Mutex::new("2015-11-12T00:00:00.0000000Z".to_string());
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

pub async fn find_player_id(name: &str) -> Result<ProfileDTO, Box<dyn std::error::Error>> {
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
    headers.insert("Ubi-SessionId", (*session_id).parse::<HeaderValue>().unwrap());

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
        return Err(format!("Failed to find player {}", name).into());
    }

    Ok(ProfileDTO { 
        id: profiles[0]["profileId"].as_str().unwrap().to_string(),
        name: profiles[0]["nameOnPlatform"].as_str().unwrap().to_string(),
    })
}

pub static DIV1_SPACE_ID: &str = "6edd234a-abff-4e90-9aab-b9b9c6e49ff7";
pub async fn get_div1_player_stats(
    pool: &Pool<Sqlite>,
    name: &str,
) -> Result<D1PlayerStats, Box<dyn std::error::Error>> {

    let mut headers = util::header::get_common_header().await;
    let ticket = UBI_TICKET.lock().unwrap().clone();
    headers.insert(
        "Authorization",
        format!("Ubi_v1 t={}", &ticket)
            .parse()
            .unwrap(),
    );

    let session_id = UBI_SESSION_ID.lock().unwrap().clone();
    headers.insert("Ubi-SessionId", (*session_id).parse::<HeaderValue>().unwrap());

    let profile = find_player_id(name).await?;

    let url = format!("https://public-ubiservices.ubi.com/v1/profiles/{}/statscard?spaceId={}", &profile.id, DIV1_SPACE_ID);

    match create_user(pool, &profile.id).await {
        Ok(_) => println!("Created or update user {}", &profile.id),
        Err(e) => return Err(e.0.into()),
    }
    match store_user_name(pool, &profile.id, &profile.name).await {
        Ok(_) => println!("Stored name {} for user {}", &profile.name, &profile.id),
        Err(e) => return Err(e.0.into()),
    }

    let resp = reqwest::Client::new()
        .get(&url)
        .headers(headers)
        .send()
        .await?
        .json::<Value>()
        .await?;

    if !resp["errorCode"].is_null() {
        println!("{:#?}", resp);
        return Err("Failed to get player stats".into());
    }
    let stats = resp["Statscards"].as_array().unwrap();
    if stats.len() != 12 {
        return Err("incomplete data".into());
    }

    let names = match get_user_names_by_id(pool, &profile.id).await {
        Ok(names) => names,
        Err(e) => return Err(e.0.into()),
    }; 

    Ok(D1PlayerStats {
        id: profile.id,
        name: profile.name,
        level: stats[0]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
        dz_rank: stats[1]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
        ug_rank: stats[2]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
        playtime: stats[3]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0) / 3600,
        main_story: stats[4]["value"].as_str().unwrap_or("0").parse::<f32>().unwrap_or(0f32) * 100f32,
        rogue_kills: stats[5]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
        items_extracted: stats[6]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
        skill_kills: stats[7]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
        total_kills: stats[8]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
        gear_score: stats[11]["value"].as_str().unwrap().parse::<u64>().unwrap_or(0),
        all_names: names,
    })
}
