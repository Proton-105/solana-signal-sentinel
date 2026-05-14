use crate::signal::types::{Signal, Source};
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

const PUMP_FUN_PROGRAM: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const RAYDIUM_CPMM:     &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
const POLL_INTERVAL_SECS: u64 = 15;

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<Vec<SignatureInfo>>,
}

#[derive(Debug, Deserialize)]
struct SignatureInfo {
    signature: String,
    #[serde(rename = "blockTime")]
    block_time: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TxResponse {
    result: Option<TxResult>,
}

#[derive(Debug, Deserialize)]
struct TxResult {
    meta: Option<TxMeta>,
    transaction: Option<TxData>,
}

#[derive(Debug, Deserialize)]
struct TxMeta {
    err: Option<serde_json::Value>,
    #[serde(rename = "preBalances")]
    pre_balances: Vec<u64>,
    #[serde(rename = "postBalances")]
    post_balances: Vec<u64>,
    #[serde(rename = "innerInstructions")]
    inner_instructions: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct TxData {
    message: Option<TxMessage>,
}

#[derive(Debug, Deserialize)]
struct TxMessage {
    #[serde(rename = "accountKeys")]
    account_keys: Vec<AccountKey>,
}

#[derive(Debug, Deserialize)]
struct AccountKey {
    pubkey: String,
}

pub struct RpcPoller {
    client:      Client,
    rpc_url:     String,
    tx:          UnboundedSender<Signal>,
    last_sig:    std::sync::Mutex<Option<String>>,
}

impl RpcPoller {
    pub fn new(rpc_url: String, tx: UnboundedSender<Signal>) -> Self {
        Self {
            client:   Client::new(),
            rpc_url,
            tx,
            last_sig: std::sync::Mutex::new(None),
        }
    }

    pub async fn run(&self) {
        info!("🔄 RPC Poller started — polling every {}s", POLL_INTERVAL_SECS);

        loop {
            // Опрашиваем Pump.fun
            if let Err(e) = self.poll_program(PUMP_FUN_PROGRAM, Source::PumpFun).await {
                error!("Poll error (Pump.fun): {e:#}");
            }

            // Опрашиваем Raydium CPMM
            if let Err(e) = self.poll_program(RAYDIUM_CPMM, Source::Raydium).await {
                error!("Poll error (Raydium): {e:#}");
            }

            sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    }

    async fn poll_program(
        &self,
        program: &str,
        source: Source,
    ) -> anyhow::Result<()> {
        // Получаем последние подписи транзакций
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSignaturesForAddress",
            "params": [
                program,
                { "limit": 10, "commitment": "confirmed" }
            ]
        });

        info!("🔍 Polling {}", program);
        let resp: RpcResponse = self.client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let sigs = match resp.result {
            Some(s) if !s.is_empty() => s,
            _ => return Ok(()),
        };

        // Берём только новые подписи
        let last = self.last_sig.lock().unwrap().clone();
        let new_sigs: Vec<_> = match &last {
            None => sigs.into_iter().take(1).collect(),
            Some(l) => sigs.into_iter()
                .take_while(|s| &s.signature != l)
                .collect(),
        };

        info!("📥 Got {} new sigs for {}", new_sigs.len(), program);

        if new_sigs.is_empty() {
            return Ok(());
        }

        // Обновляем last_sig
        *self.last_sig.lock().unwrap() = Some(new_sigs[0].signature.clone());

        // Обрабатываем каждую новую транзакцию
        for sig_info in new_sigs {
            if let Ok(Some(signal)) = self
                .fetch_and_parse(&sig_info.signature, source.clone())
                .await
            {
                info!("📡 New signal from RPC: mint={}", signal.mint);
                let _ = self.tx.send(signal);
            }
        }

        Ok(())
    }

    async fn fetch_and_parse(
        &self,
        signature: &str,
        source: Source,
    ) -> anyhow::Result<Option<Signal>> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                signature,
                { "encoding": "jsonParsed", "commitment": "confirmed", "maxSupportedTransactionVersion": 0 }
            ]
        });

        let resp: TxResponse = self.client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let result = match resp.result {
            Some(r) => r,
            None => {
                info!("⚠️ getTransaction returned null for {}", signature);
                return Ok(None);
            }
        };

        let meta = match result.meta {
            Some(m) if m.err.is_none() => m,
            Some(_) => {
                info!("⚠️ Transaction failed, skipping {}", signature);
                return Ok(None);
            }
            None => {
                info!("⚠️ No meta for {}", signature);
                return Ok(None);
            }
        };

        let account_keys: Vec<String> = result.transaction
            .and_then(|t| t.message)
            .map(|m| m.account_keys.into_iter().map(|k| k.pubkey).collect())
            .unwrap_or_default();

        if account_keys.is_empty() {
            return Ok(None);
        }

        let creator = account_keys.first().cloned().unwrap_or_default();
        let mint    = account_keys.get(1).cloned().unwrap_or_default();
        let pool    = account_keys.get(2).cloned().unwrap_or_default();

        // Ликвидность
        let liquidity_sol = meta.post_balances.iter()
            .zip(meta.pre_balances.iter())
            .map(|(post, pre)| *post as i64 - *pre as i64)
            .filter(|&d| d > 0)
            .max()
            .unwrap_or(0) as f64 / 1_000_000_000.0;

        let buy_count = meta.inner_instructions.len() as u32;

        info!("✅ Parsed signal: mint={} liq={:.2} buys={}", mint, liquidity_sol, buy_count);
        Ok(Some(Signal {
            mint,
            pool,
            source,
            liquidity_sol,
            buy_count,
            smart_money: false,
            dev_sold_pct: 0.0,
            creator,
            detected_at: Utc::now(),
        }))
    }
}
