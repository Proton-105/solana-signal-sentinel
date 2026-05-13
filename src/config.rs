#![allow(dead_code)]
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub detector: DetectorConfig,
    pub scorer:   ScorerConfig,
    pub stream:   StreamConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DetectorConfig {
    pub max_pool_age_secs: u64,
    pub min_liquidity_sol: f64,
    pub min_buy_count_30s: u32,
    pub max_dev_sell_pct:  f64,
}

#[derive(Debug, Deserialize)]
pub struct ScorerConfig {
    pub min_score: u8,
}

#[derive(Debug, Deserialize)]
pub struct StreamConfig {
    pub commitment: String,
}

pub struct Env {
    pub grpc_endpoint:       String,
    pub grpc_x_token:        Option<String>,
    pub telegram_bot_token:  String,
    pub telegram_channel_id: String,
}

pub fn load_env() -> Result<Env> {
    dotenvy::dotenv().ok();
    Ok(Env {
        grpc_endpoint:       std::env::var("GRPC_ENDPOINT")?,
        grpc_x_token:        std::env::var("GRPC_X_TOKEN").ok(),
        telegram_bot_token:  std::env::var("TELEGRAM_BOT_TOKEN")?,
        telegram_channel_id: std::env::var("TELEGRAM_CHANNEL_ID")?,
    })
}

pub fn load_config() -> Result<AppConfig> {
    let cfg = config::Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()?;
    Ok(cfg.try_deserialize()?)
}
