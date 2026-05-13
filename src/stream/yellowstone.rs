use anyhow::{Context, Result};
use futures::{sink::SinkExt, StreamExt};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use yellowstone_grpc_client::GeyserGrpcClient;
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
    SubscribeRequestFilterTransactions,
};
use tracing::{info, error};

pub struct YellowstoneStream {
    endpoint: String,
    x_token:  Option<String>,
}

impl YellowstoneStream {
    pub fn new(endpoint: String, x_token: Option<String>) -> Self {
        Self { endpoint, x_token }
    }

    pub async fn run(&self) -> Result<()> {
        let mut backoff = 1;
        loop {
            match self.connect_and_stream().await {
                Ok(_) => {
                    info!("Stream closed gracefully. Reconnecting...");
                    backoff = 1;
                }
                Err(e) => {
                    error!("Stream error: {}. Reconnecting in {}s...", e, backoff);
                    sleep(Duration::from_secs(backoff)).await;
                    backoff = (backoff * 2).min(32);
                }
            }
        }
    }

    async fn connect_and_stream(&self) -> Result<()> {
        let mut client = GeyserGrpcClient::build_from_shared(self.endpoint.clone())?
            .x_token(self.x_token.clone())?
            .connect()
            .await
            .context("Failed to connect to Yellowstone gRPC")?;

        let mut transactions = HashMap::new();
        
        // Pump.fun Program
        transactions.insert(
            "pump_fun".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string()],
                account_exclude: vec![],
                account_required: vec![],
            },
        );

        // Raydium Liquidity Pool V4
        transactions.insert(
            "raydium".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(false),
                failed: Some(false),
                signature: None,
                account_include: vec!["CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C".to_string()],
                account_exclude: vec![],
                account_required: vec![],
            },
        );

        let request = SubscribeRequest {
            transactions,
            commitment: Some(CommitmentLevel::Confirmed as i32),
            ..Default::default()
        };

        let (mut subscribe_tx, mut stream) = client.subscribe().await?;
        subscribe_tx.send(request).await?;

        info!("Subscribed to Yellowstone stream");

        while let Some(message) = stream.next().await {
            let message = message.context("Stream message error")?;
            
            if let Some(update) = message.update_oneof {
                match update {
                    UpdateOneof::Transaction(tx) => {
                        if let Some(tx_info) = tx.transaction {
                            let sig = bs58::encode(&tx_info.signature).into_string();
                            info!("New transaction: {}", sig);
                            // TODO: Process transaction for signals
                        }
                    }
                    UpdateOneof::Ping(_) => {
                        subscribe_tx.send(SubscribeRequest {
                            ping: Some(yellowstone_grpc_proto::prelude::SubscribeRequestPing { id: 1 }),
                            ..Default::default()
                        }).await?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
