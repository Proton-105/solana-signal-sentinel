use crate::config::DetectorConfig;
#![allow(dead_code, unused_variables)]
use crate::signal::types::{Signal, Source};
use chrono::Utc;
use yellowstone_grpc_proto::prelude::SubscribeUpdateTransactionInfo;

const PUMP_FUN_PROGRAM:  &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const RAYDIUM_CPMM:      &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";

pub fn detect(
    tx: &SubscribeUpdateTransactionInfo,
    config: &DetectorConfig,
) -> Option<Signal> {
    // Только успешные транзакции
    let meta = tx.meta.as_ref()?;
    if meta.err.is_some() {
        return None;
    }

    let inner_tx = tx.transaction.as_ref()?;
    let message  = inner_tx.message.as_ref()?;

    // Определяем источник по account_keys
    let account_keys: Vec<String> = message
        .account_keys
        .iter()
        .map(|k| bs58::encode(k).into_string())
        .collect();

    let source = if account_keys.contains(&PUMP_FUN_PROGRAM.to_string()) {
        Source::PumpFun
    } else if account_keys.contains(&RAYDIUM_CPMM.to_string()) {
        Source::Raydium
    } else {
        return None;
    };

    // Извлекаем creator (первый подписант)
    let creator = account_keys.first()?.clone();

    // Извлекаем mint (второй аккаунт после creator)
    let mint = account_keys.get(1)?.clone();

    // Ликвидность из postTokenBalances
    let liquidity_sol = extract_liquidity(meta);

    // Фильтр: минимальная ликвидность
    if liquidity_sol < config.min_liquidity_sol {
        return None;
    }

    // buy_count из inner instructions (упрощённо: кол-во inner ix)
    let buy_count = meta.inner_instructions.len() as u32;

    // Фильтр: минимум покупок
    if buy_count < config.min_buy_count_30s {
        return None;
    }

    // dev_sold_pct — пока 0.0, заполним в следующей итерации
    let dev_sold_pct = 0.0_f64;

    // smart_money — пока false, заполним в следующей итерации  
    let smart_money = false;

    Some(Signal {
        mint,
        pool: account_keys.get(2).cloned().unwrap_or_default(),
        source,
        liquidity_sol,
        buy_count,
        smart_money,
        dev_sold_pct,
        creator,
        detected_at: Utc::now(),
    })
}

fn extract_liquidity(
    meta: &yellowstone_grpc_proto::prelude::TransactionStatusMeta,
) -> f64 {
    // Ищем изменение SOL баланса в postBalances vs preBalances
    if meta.post_balances.is_empty() || meta.pre_balances.is_empty() {
        return 0.0;
    }

    // Максимальное изменение баланса = ликвидность добавленная в пул
    let max_delta = meta
        .post_balances
        .iter()
        .zip(meta.pre_balances.iter())
        .map(|(post, pre)| *post as i64 - *pre as i64)
        .filter(|&delta| delta > 0)
        .max()
        .unwrap_or(0);

    // Конвертируем lamports → SOL
    max_delta as f64 / 1_000_000_000.0
}
