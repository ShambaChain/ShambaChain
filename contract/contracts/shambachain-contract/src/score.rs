use crate::types::CreditTier;

/// ShambaChain Credit Scoring Engine
///
/// Score = 100  (base — every registered farmer starts here)
///       + harvest_frequency   (max 200) — rewards consistent harvesting
///       + sales_consistency   (max 200) — rewards converting harvest → sales
///       + income_growth       (max 200) — rewards growing local currency income
///       + record_regularity   (max 150) — rewards frequent on-chain activity
///
/// Maximum possible score = 850  (mirrors FICO scale upper range)

pub fn calculate(
    total_harvests: u32,
    total_sales: u32,
    total_income_kes: u64,
    total_entries: u64,
) -> u32 {
    let freq = (total_harvests.saturating_mul(25)).min(200);
    let cons = (total_sales.saturating_mul(30)).min(200);
    // Divisor 500 is calibrated for KES; other currencies scale proportionally.
    // On the frontend. The contract stores raw integer
    let grow = (((total_income_kes / 500) as u32)).min(200);
    let reg  = (((total_entries.saturating_mul(10)) as u32)).min(150);

    (100_u32)
        .saturating_add(freq)
        .saturating_add(cons)
        .saturating_add(grow)
        .saturating_add(reg)
        .min(850)
}

pub fn to_tier(score: u32) -> CreditTier {
    match score {
        750..=850 => CreditTier::Platinum,
        650..=749 => CreditTier::Gold,
        500..=649 => CreditTier::Silver,
        300..=499 => CreditTier::Bronze,
        _ => CreditTier::Unranked,
    }
}

// Maximum USDC loan permitted per tier

pub fn max_loan_usdc(tier: &CreditTier) -> u64 {
    match tier {
        CreditTier::Platinum => 5_000,
        CreditTier::Gold => 2_000,
        CreditTier::Silver => 800,
        CreditTier::Bronze => 200,
        CreditTier::Unranked => 50,
    }
}

// Minimum score required to apply for any loan
pub const MIN_LOAN_SCORE: u32 = 200;