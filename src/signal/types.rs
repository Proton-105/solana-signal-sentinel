use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum Source {
    PumpFun,
    Raydium,
}

#[derive(Debug, Clone)]
pub struct Signal {
    pub mint:             String,
    pub pool:             String,
    pub source:           Source,
    pub liquidity_sol:    f64,
    pub buy_count:        u32,
    pub smart_money:      bool,
    pub dev_sold_pct:     f64,
    pub creator:          String,
    pub detected_at:      DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ScoredSignal {
    pub signal: Signal,
    pub score:  u8,
}
