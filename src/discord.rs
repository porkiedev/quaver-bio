//
// Contains the interface code for the Discord API
//

use anyhow::Result;
use serde::Serialize;
use tracing::{debug, error, trace};

/// The Discord user API URI
const API_URL: &str = "https://discord.com/api/v9/users/@me/profile";
/// The user agent used when making requests to the Discord API
/// 
/// - NOTE: Spoofing user agents is generally a bad idea, but I am doing it to reduce the chances of being blocked by Discord
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:129.0) Gecko/20100101 Firefox/129.0";

/// The Discord API interface
#[derive(Debug)]
pub struct Api {
    /// The http client
    client: reqwest::Client,
    /// The discord account token
    token: String,
}
impl Api {
    /// Create a new API instance
    pub async fn new(token: &str) -> Result<Self> {
        // Create the http client
        let client  = reqwest::ClientBuilder::default()
        .user_agent(USER_AGENT)
        .build()
        .map_err(Error::ClientBuilder)?;

        Ok(Self {
            client,
            token: token.to_owned()
        })
    }

    /// Set the user's bio
    pub async fn set_bio(&self, body: &ProfileRequest) -> Result<()> {

        // Query the API
        let resp = self.client.patch(API_URL)
        .header("Authorization", &self.token)
        .json(body)
        .send().await
        .map_err(Error::Request)?;

        // The bio update was successful
        if resp.status().is_success() {
            // Get the response body
            let body = resp.text().await?;

            // Print the response from the API
            trace!("Successfully set the user's bio
            API Response: {body}");
            Ok(())
        }
        // Failed to set the bio, print the error
        else {
            // Get the response body
            let body = resp.text().await?;

            // Print and return the rror
            error!("Failed to set the user's bio: {body}");
            Err(Error::Unknown(body))?
        }
    }
}

// ===== STRUCTS ===== //

/// The request body sent to the Discord API to set the user's bio
#[derive(Debug, Default, Serialize)]
pub struct ProfileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_effect: Option<u64>
}

// ===== ENUMS ===== //

/// Errors that can occur when working with the Discord API
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create a client to interface with the discord API (this should never happen): {0}")]
    ClientBuilder(reqwest::Error),
    #[error("Failed to query the discord API: {0}")]
    Request(reqwest::Error),
    #[error("Failed to query the discord API: {0}")]
    Unknown(String)
}
