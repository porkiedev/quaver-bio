//
// Contains the interface code for the Quaver API
//

use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};

/// The Quaver API URI
const API_URL: &str = "https://api.quavergame.com";

/// Queries the Quaver API for a user's profile
pub async fn get_user(id: u64) -> Result<User> {

    // Query the API
    let mut response = reqwest::get(format!("{API_URL}/v2/user/{id}"))
    .await
    .map_err(Error::Request)?
    .json::<Value>()
    .await
    .map_err(Error::JsonParse)?;

    // Deserialize the json body into a user
    let user = serde_json::from_value::<User>(response["user"].take())
    .map_err(Error::Deserialize)?;

    Ok(user)
}

// TODO: Use arraystrings to minimize allocations for short strings?
/// A Quaver user, returned from the API
#[derive(Debug, Default, Deserialize)]
pub struct User {
    pub id: u64,
    pub steam_id: String,
    pub username: String,
    pub time_registered: String,
    pub allowed: bool,
    pub privileges: u64,
    pub usergroups: u64,
    pub mute_end_time: String,
    pub latest_activity: String,
    pub country: String,
    pub avatar_url: String,
    pub twitter: Option<String>,
    pub title: Option<String>,
    pub twitch_username: Option<String>,
    pub donator_end_time: String,
    pub discord_id: Option<String>,
    pub misc_information: Option<UserMiscInformation>,
    pub clan_id: Option<u64>,
    pub clan_leave_time: String,
    pub clan_status: Option<String>,
    pub stats_keys4: Stats,
    pub stats_keys7: Stats,
}

// TODO: Are there additional fields that I'm unaware of?
/// A user's miscellaneous information, returned from the API
#[derive(Debug, Default, Deserialize)]
pub struct UserMiscInformation {
    pub discord: Option<String>,
    pub default_mode: u64
}

/// A user's 4K stats, returned from the API
#[derive(Debug, Default, Deserialize)]
pub struct Stats {
    pub ranks: Ranks,
    pub total_score: u64,
    pub ranked_score: u64,
    pub overall_accuracy: f64,
    pub overall_performance_rating: f64,
    pub play_count: u64,
    pub fail_count: u64,
    pub max_combo: u64,
    pub total_marvelous: u64,
    pub total_perfect: u64,
    pub total_great: u64,
    pub total_good: u64,
    pub total_okay: u64,
    pub total_miss: u64,
    pub count_grade_x: u64,
    pub count_grade_ss: u64,
    pub count_grade_s: u64,
    pub count_grade_a: u64,
    pub count_grade_b: u64,
    pub count_grade_c: u64,
    pub count_grade_d: u64,
}

/// A user's ranks, returned from the API
#[derive(Debug, Default, Deserialize)]
pub struct Ranks {
    pub global: u64,
    pub country: u64,
    pub total_hits: u64
}

// ===== ENUMS ===== //

/// Errors that can occur when working with the Quaver API
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to query the Quaver API: {0}")]
    Request(reqwest::Error),
    #[error("Failed to parse the API response into valid JSON: {0}")]
    JsonParse(reqwest::Error),
    #[error("Failed to deserialize the API response: {0}")]
    Deserialize(serde_json::Error),
}
