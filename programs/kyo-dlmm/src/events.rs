use anchor_lang::prelude::*;

#[event]
pub struct HonoraryPositionInitialized {
    pub position_key: Pubkey,
    pub pool_id: Pubkey,
    pub quote_mint: Pubkey,
    pub base_mint: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct QuoteFeesClaimed {
    pub position_key: Pubkey,
    pub claimed_quote_fees: u64,
    pub claimed_base_fees: u64,
    pub timestamp: i64,
}

#[event]
pub struct InvestorPayoutPage {
    pub page_number: u32,
    pub investors_processed: u32,
    pub total_payout: u64,
    pub timestamp: i64,
}

#[event]
pub struct CreatorPayoutDayClosed {
    pub daily_total_claimed: u64,
    pub investor_share: u64,
    pub creator_share: u64,
    pub carry_over: u64,
    pub timestamp: i64,
}

#[event]
pub struct PolicyUpdated {
    pub new_investor_fee_share_bps: u16,
    pub new_daily_cap_quote: u64,
    pub new_min_payout_lamports: u64,
    pub timestamp: i64,
}
