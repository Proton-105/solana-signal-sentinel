use crate::signal::types::{ScoredSignal, Source};

pub fn format_signal(scored: &ScoredSignal) -> String {
    let signal = &scored.signal;
    let score  = scored.score;

    let source_emoji = match signal.source {
        Source::PumpFun => "🟣 Pump.fun",
        Source::Raydium => "🔵 Raydium",
    };

    let score_bar = build_score_bar(score);

    let smart_money_line = if signal.smart_money {
        "💎 Smart money: ✅ присутствует"
    } else {
        "💎 Smart money: ➖ не обнаружен"
    };

    format!(
        "🎯 <b>HIGH-CONVICTION SIGNAL</b>\n\
         {source_emoji}\n\
         \n\
         <b>Score:</b> {score}/100 {score_bar}\n\
         \n\
         <b>Mint:</b> <code>{mint}</code>\n\
         <b>Pool:</b> <code>{pool}</code>\n\
         \n\
         💧 <b>Ликвидность:</b> {liq:.2} SOL\n\
         🛒 <b>Покупок за 30с:</b> {buys}\n\
         {smart_money_line}\n\
         👤 <b>Creator:</b> <code>{creator_short}</code>\n\
         \n\
         🔗 <a href=\"https://dexscreener.com/solana/{mint}\">Dexscreener</a> · \
         <a href=\"https://birdeye.so/token/{mint}\">Birdeye</a> · \
         <a href=\"https://solscan.io/token/{mint}\">Solscan</a>\n\
         \n\
         🕐 {detected_at} UTC",
        source_emoji   = source_emoji,
        score          = score,
        score_bar      = score_bar,
        mint           = signal.mint,
        pool           = signal.pool,
        liq            = signal.liquidity_sol,
        buys           = signal.buy_count,
        smart_money_line = smart_money_line,
        creator_short  = shorten(&signal.creator, 8),
        detected_at    = signal.detected_at.format("%H:%M:%S"),
    )
}

fn build_score_bar(score: u8) -> String {
    let filled = (score as usize) / 10;
    let empty  = 10 - filled.min(10);
    format!("{}{}", "🟩".repeat(filled), "⬜".repeat(empty))
}

fn shorten(s: &str, n: usize) -> String {
    if s.len() <= n * 2 {
        return s.to_string();
    }
    format!("{}...{}", &s[..n], &s[s.len() - n..])
}
