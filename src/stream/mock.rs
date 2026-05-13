use crate::signal::types::{Signal, Source};
use chrono::Utc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{sleep, Duration};
use tracing::info;

pub struct MockStream {
    tx: UnboundedSender<Signal>,
}

impl MockStream {
    pub fn new(tx: UnboundedSender<Signal>) -> Self {
        Self { tx }
    }

    pub async fn run(&self) {
        info!("🧪 Mock stream started — generating fake signals every 10s");

        let mints = vec![
            "So11111111111111111111111111111111111111112",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            "7GCihgDB8fe6KNjn2MYtkzZcRjQy3t9GHdC8uHYmW2hr",
        ];

        let mut counter = 0usize;

        loop {
            sleep(Duration::from_secs(10)).await;

            let mint    = mints[counter % mints.len()].to_string();
            let source  = if counter % 2 == 0 { Source::PumpFun } else { Source::Raydium };
            let liq     = 8.0 + (counter as f64 * 1.5) % 20.0;
            let buys    = 18 + (counter as u32 * 3) % 30;
            let smart   = counter % 3 == 0;

            let signal = Signal {
                mint:          mint.clone(),
                pool:          format!("Pool{}", counter),
                source,
                liquidity_sol: liq,
                buy_count:     buys,
                smart_money:   smart,
                dev_sold_pct:  0.0,
                creator:       "CreatorWallet111111111111111111111111111111".to_string(),
                detected_at:   Utc::now(),
            };

            info!("🧪 Mock signal generated: mint={} liq={:.2} buys={}", mint, liq, buys);

            let _ = self.tx.send(signal);
            counter += 1;
        }
    }
}
