mod config;
mod signal;
mod stream;
mod detector;
mod scorer;
mod formatter;
mod poster;
mod utils;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::info;

use crate::signal::types::Signal;
use crate::stream::mock::MockStream;
use crate::poster::telegram_poster::TelegramPoster;
use crate::utils::dedup::DedupCache;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("🚀 Starting Solana Signal Sentinel [MOCK MODE]...");

    let env        = config::load_env()?;
    let app_config = config::load_config()?;

    info!("✅ Configuration loaded.");

    let (tx, mut rx) = mpsc::unbounded_channel::<Signal>();

    // Mock stream таск
    let mock = MockStream::new(tx);
    tokio::spawn(async move {
        mock.run().await;
    });

    // Telegram poster
    let poster = TelegramPoster::new(
        env.telegram_bot_token,
        env.telegram_channel_id,
    );

    // Dedup cache — TTL 5 минут
    let mut dedup = DedupCache::new(300);

    info!("📡 Listening for mock signals...");

    // Главный цикл
    while let Some(signal) = rx.recv().await {
        // Дедупликация
        if dedup.is_duplicate(&signal.mint) {
            info!("⏭ Duplicate skipped: {}", signal.mint);
            continue;
        }

        // Scorer
        if let Some(scored) = scorer::score(signal, &app_config.scorer) {
            info!(
                "🎯 SIGNAL | mint={} score={} liq={:.2} SOL buys={}",
                scored.signal.mint,
                scored.score,
                scored.signal.liquidity_sol,
                scored.signal.buy_count,
            );

            // Форматируем и постим в Telegram
            let text = formatter::telegram::format_signal(&scored);
            if let Err(e) = poster.send(text).await {
                tracing::error!("Failed to send Telegram message: {e:#}");
            }
        } else {
            info!("⏭ Signal below threshold, skipped.");
        }
    }

    Ok(())
}
