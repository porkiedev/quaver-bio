#![allow(unused)]

mod quaver;
mod discord;
use std::{collections::HashMap, env::{self, current_exe}, fs::File, io::ErrorKind, path::{Path, PathBuf}, process, time::Duration};
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, level_filters::LevelFilter, trace, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt, Layer};

/// The character limit for the discord bio. I couldn't find the exact number so this is a conservative estimate
const BIO_CHAR_LIMIT: usize = 190;
/// The name of the application
const APP_NAME: &str = env!("CARGO_PKG_NAME");

// The environment variable configs
lazy_static! {
    /// The path to the config file. This is `config.json` in the same directory as the executable by default
    static ref CONFIG_PATH: String = get_env_var("QB_CONFIG_PATH").unwrap_or_else(|| get_exe_dir().unwrap().join("config.json").to_string_lossy().to_string());
    /// The stdout log level for the application. This is `TRACE` by default
    static ref LOG_LEVEL: LevelFilter = {
        let level = get_env_var("QB_LOG_LEVEL").unwrap_or_else(|| "TRACE".to_string());
        level.parse()
        .expect("Invalid log level specified. Valid options are TRACE, DEBUG, INFO, WARN, and ERROR")
    };
    /// The Loki log level for the application. This is `WARN` by default
    static ref LOKI_LOG_LEVEL: LevelFilter = {
        let level = get_env_var("QB_LOKI_LOG_LEVEL").unwrap_or_else(|| "WARN".to_string());
        level.parse()
        .expect("Invalid log level specified. Valid options are TRACE, DEBUG, INFO, WARN, and ERROR")
    };
    /// The endpoint for the Loki log server
    static ref LOKI_URL: Option<String> = get_env_var("QB_LOKI_URL");
    /// The user's Discord token. This can be provided directly or via a file path (useful for docker secret support, amongst other things)
    static ref DISCORD_TOKEN: String = get_env_var_file("QB_DISCORD_TOKEN").unwrap().expect("The QB_DISCORD_TOKEN environment variable was not set");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create the loki log layer if an endpoint was provided
    let loki_layer = match LOKI_URL.as_ref() {
        Some(url) => {
            let (loki_layer, loki_task) = tracing_loki::builder()
                .label("application", APP_NAME)?
                .extra_field("pid", process::id().to_string())?
                .build_url(Url::parse(url)?)?;

            // Set the Loki log level to ERROR (i.e. only log errors to Loki)
            let loki_layer = loki_layer.with_filter(*LOKI_LOG_LEVEL);

            // Spawn the Loki logging task in the background
            tokio::spawn(loki_task);

            Some(loki_layer)
        }
        None => None,
    };
    // Create the standard log layer
    let standard_layer = tracing_subscriber::fmt::layer().with_filter(*LOG_LEVEL);
    // Initialize the log system
    tracing_subscriber::registry()
        .with(loki_layer)
        .with(standard_layer)
        .init();
    
    // Get the application config
    let config = Config::new(&CONFIG_PATH).unwrap();
    // Create a discord API handle
    let discord_api = discord::Api::new(&DISCORD_TOKEN).await?;

    // Create the regex for formatting the bio string
    let bio_regex = Regex::new(r"\{(\w+)\}").unwrap();

    // Has the loop started yet?
    let mut started = false;

    // Update the user's bio ~forever~
    loop {

        // Sleep for for the configured interval. This doesn't apply to the first loop iteration
        if started {
            tokio::time::sleep(Duration::from_secs(config.update_interval)).await;
        }
        // Update the started variable now that we've started our first iteration
        started = true;

        // Get the user's quaver account data
        let user = match quaver::get_user(config.quaver_user_id).await {
            Ok(u) => u,
            Err(err) => {
                error!("Failed to get the user's quaver account data: {err}");
                continue;
            }
        };

        // Format the new bio string with the user's data
        let bio_str = bio_regex.replace_all(&config.bio_schema, |caps: &Captures| {
            // Get the schema key
            let key = &caps[1];
            // Get the value associated with the key
            map_key_to_user_value(key, &user)
        });

        // Ensure the bio string isn't too long
        if bio_str.chars().count() > BIO_CHAR_LIMIT {
            error!("The bio string exceeds the character limit of {BIO_CHAR_LIMIT}. Skipping...");
            continue;
        }

        // Set the user's bio
        if let Err(e) = discord_api.set_bio(&discord::ProfileRequest {
            bio: Some(bio_str.to_string()),
            ..Default::default()
        }).await {
            error!("Failed to set the user's bio: {e}");
            continue;
        };

        info!("Successfully set the user's bio");
    }

    Ok(())
}

// ===== UTILITY FUNCTIONS ===== //
/// Takes the input key and user struct, then returns the value associated with it
/// 
/// - NOTE: If an invalid key is provided, a blank string will be returned
/// 
/// Supported keys:
/// - `{username}` - The player's username
/// - `{country}` - The player's country
/// - `{4k_rank}` - The player's 4K global rank
/// - `{4k_rank_country}` - The player's 4K country rank
/// - `{4k_total_score}` - The player's 4K total score
/// - `{4k_ranked_score}` - The player's 4K ranked score
/// - `{4k_accuracy}` - The player's 4K accuracy
/// - `{4k_performance_rating}` - The player's 4K performance rating
/// - `{4k_play_count}` - The player's 4K play count
/// - `{4k_fail_count}` - The player's 4K fail count
/// - `{4k_max_combo}` - The player's 4K max combo
/// - `{7k_rank}` - The player's global rank for the 7K mode
/// - `{7k_rank_country}` - The player's country rank for the 7K mode
/// - `{7k_total_score}` - The player's 7K total score
/// - `{7k_ranked_score}` - The player's 7K ranked score
/// - `{7k_accuracy}` - The player's 7K accuracy
/// - `{7k_performance_rating}` - The player's 7K performance rating
/// - `{7k_play_count}` - The player's 7K play count
/// - `{7k_fail_count}` - The player's 7K fail count
/// - `{7k_max_combo}` - The player's 7K max combo
fn map_key_to_user_value(key: &str, user: &quaver::User) -> String {
    match key {
        "username" => user.username.to_string(),
        "country" => user.country.to_string(),
        "4k_rank" => user.stats_keys4.ranks.global.to_string(),
        "4k_rank_country" => user.stats_keys4.ranks.country.to_string(),
        "4k_total_score" => user.stats_keys4.total_score.to_string(),
        "4k_ranked_score" => user.stats_keys4.ranked_score.to_string(),
        "4k_accuracy" => format!("{:.2}", user.stats_keys4.overall_accuracy),
        "4k_performance_rating" => format!("{:.2}", user.stats_keys4.overall_performance_rating),
        "4k_play_count" => user.stats_keys4.play_count.to_string(),
        "4k_fail_count" => user.stats_keys4.fail_count.to_string(),
        "4k_max_combo" => user.stats_keys4.max_combo.to_string(),
        "7k_rank" => user.stats_keys7.ranks.global.to_string(),
        "7k_rank_country" => user.stats_keys7.ranks.country.to_string(),
        "7k_total_score" => user.stats_keys7.total_score.to_string(),
        "7k_ranked_score" => user.stats_keys7.ranked_score.to_string(),
        "7k_accuracy" => format!("{:.2}", user.stats_keys7.overall_accuracy),
        "7k_performance_rating" => format!("{:.2}", user.stats_keys7.overall_performance_rating),
        "7k_play_count" => user.stats_keys7.play_count.to_string(),
        "7k_fail_count" => user.stats_keys7.fail_count.to_string(),
        "7k_max_combo" => user.stats_keys7.max_combo.to_string(),
        _ => String::new()
    }
}

/// Returns the value of the specified environment variable, and if the value resolves to a file path, the file is read and the content is returned instead
/// 
/// - NOTE: Returns an error if the environment variable is not found or if the file cannot be read
// TODO: Check for permission errors when working with env var files
fn get_env_var_file(key: &str) -> Result<Option<String>> {
    match env::var(key) {
        Ok(mut v) => {
            // Remove any trailing whitespace
            v = v.trim_end_matches(' ').to_string();

            // Try to convert the value to a file path
            let path = Path::new(&v);

            trace!("Is file ({}), Exists ({})", path.is_file(), path.exists());

            // The value is a file path, read the file and return its content
            if path.exists() && path.is_file() {
                trace!("Using discord key from file ({path:?}): {:?}", std::fs::read_to_string(path)?);
                Ok(Some(std::fs::read_to_string(path)?))
            }
            // The value is not a file path, so return it as is
            else {
                trace!("Using discord key directly: {v}");
                Ok(Some(v))
            }
        },
        Err(e) => Ok(None)
    }
}

/// Returns the value of the specified environment variable if it exists
/// 
/// - NOTE: Unlike `get_env_var_file`, this function will not read a file if the value is a file path
/// - NOTE: Returns an error if the environment variable is not found or if the file cannot be read
fn get_env_var(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(v) => {
            // Remove any trailing whitespace
            Some(v.trim_end_matches(' ').to_string())
        },
        Err(e) => None
    }
}

/// Gets the parent directory of the current executable
fn get_exe_dir() -> Result<PathBuf> {
    Ok(current_exe()?.parent().unwrap().to_path_buf())
}

// ===== STRUCTS ===== //

/// A persistent configuration for the application
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct Config {
    /// The user's quaver user id
    quaver_user_id: u64,
    /// The schema string for the user's bio
    bio_schema: String,
    /// The bio update interval in seconds (30 minutes by default)
    update_interval: u64
}
impl Default for Config {
    fn default() -> Self {
        Self {
            quaver_user_id: 0,
            bio_schema: "Hello, {username}! Your 4K rank is {4k_rank}.".to_string(),
            update_interval: 1_800
        }
    }
}
impl Config {
    /// Returns the existing config file, creating a new one and panicking if it doesn't exist (allowing the user to populate the config)
    fn new(path: &str) -> Result<Self> {
        trace!("Trying to load the config file at `{path}`");

        // Open the config file
        let file = match File::open(path) {
            Ok(f) => f,
            Err(err) => {
                // The config file wasn't found, create a new one
                if ErrorKind::NotFound == err.kind() {
                    Self::default().save(path)?;
                    Err(Error::ConfigNotFound)?
                }
                // The config file couldn't be opened
                else {
                    Err(Error::IoRead(err))?
                }
            }
        };

        // Read the config file
        Ok(serde_json::from_reader(file)
        .map_err(Error::Serde)?)
    }

    fn save(&self, path: &str) -> Result<()> {
        trace!("Trying to save the config file to `{path}`");

        // Create the config file
        let file = File::create(path)
        .map_err(Error::IoWrite)?;

        // Write self to the config file
        Ok(serde_json::to_writer_pretty(file, &self)
        .map_err(Error::Serde)?)
    }
}

// ===== ENUMS ===== //
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to write to the config file: {0}")]
    IoWrite(std::io::Error),
    #[error("Failed to read the config file: {0}")]
    IoRead(std::io::Error),
    #[error("Failed to parse the config file: {0}")]
    Serde(serde_json::Error),
    #[error("Config file not found. A new one has been created for you, please edit it and restart the program")]
    ConfigNotFound,
}
