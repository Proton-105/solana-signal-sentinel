use crate::config::ScorerConfig;
use crate::signal::types::{Signal, ScoredSignal};

/// Веса факторов (сумма = 100)
const WEIGHT_LIQUIDITY:   u8 = 25;
const WEIGHT_BUY_COUNT:   u8 = 20;
const WEIGHT_SMART_MONEY: u8 = 25;
const WEIGHT_DEV_BEHAVIOR: u8 = 15;
const WEIGHT_POOL_AGE:    u8 = 15;

pub fn score(signal: Signal, config: &ScorerConfig) -> Option<ScoredSignal> {
    let mut total: u8 = 0;

    // Ликвидность: 8 SOL = 50%, 20+ SOL = 100%
    let liq_score = normalize(signal.liquidity_sol, 8.0, 20.0);
    total = total.saturating_add(apply_weight(liq_score, WEIGHT_LIQUIDITY));

    // Покупки: 18 = 50%, 40+ = 100%
    let buy_score = normalize(signal.buy_count as f64, 18.0, 40.0);
    total = total.saturating_add(apply_weight(buy_score, WEIGHT_BUY_COUNT));

    // Smart money: присутствует = 100%, нет = 0%
    let sm_score = if signal.smart_money { 1.0 } else { 0.0 };
    total = total.saturating_add(apply_weight(sm_score, WEIGHT_SMART_MONEY));

    // Dev поведение: 0% слив = 100%, 4%+ слив = 0%
    let dev_score = normalize_inverse(signal.dev_sold_pct, 0.0, 4.0);
    total = total.saturating_add(apply_weight(dev_score, WEIGHT_DEV_BEHAVIOR));

    // Возраст пула: учитываем всегда как полный балл пока нет timestamp пула
    // TODO: заменить на реальный возраст в следующей итерации
    total = total.saturating_add(WEIGHT_POOL_AGE);

    // Порог
    if total < config.min_score {
        return None;
    }

    Some(ScoredSignal { signal, score: total })
}

/// Нормализует value в диапазон [min, max] → [0.0, 1.0]
fn normalize(value: f64, min: f64, max: f64) -> f64 {
    if value <= min { return 0.0; }
    if value >= max { return 1.0; }
    (value - min) / (max - min)
}

/// Инвертированная нормализация (меньше = лучше)
fn normalize_inverse(value: f64, min: f64, max: f64) -> f64 {
    1.0 - normalize(value, min, max)
}

/// Применяет вес к нормализованному score
fn apply_weight(normalized: f64, weight: u8) -> u8 {
    (normalized * weight as f64).round() as u8
}
