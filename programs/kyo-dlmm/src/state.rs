use anchor_lang::prelude::*;

#[account]
pub struct PolicyConfig {
    /// Fee share for investors in basis points (0-10000)
    pub investor_fee_share_bps: u16,
    /// Daily cap for distributions in quote tokens
    pub daily_cap_quote: u64,
    /// Minimum payout threshold to avoid dust
    pub min_payout_lamports: u64,
    /// Authority that can update policy
    pub authority: Pubkey,
    /// Bump seed for the policy PDA
    pub bump: u8,
}

impl PolicyConfig {
    pub const LEN: usize = 8 + 2 + 8 + 8 + 32 + 1;
}

#[account]
pub struct DistributionProgress {
    /// Last distribution timestamp
    pub last_distribution_ts: i64,
    /// Cumulative amount distributed today
    pub daily_distributed: u64,
    /// Carry-over amount from previous day
    pub carry_over: u64,
    /// Current pagination cursor
    pub cursor: u32,
    /// Total investor allocation at TGE (Y0)
    pub total_investor_allocation: u64,
    /// Bump seed for the progress PDA
    pub bump: u8,
}

impl DistributionProgress {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 8 + 4 + 8 + 1;
}

#[account]
pub struct HonoraryPosition {
    /// The CP-AMM position key
    pub position_key: Pubkey,
    /// Pool configuration
    pub pool_config: PoolConfig,
    /// Quote mint (must be the fee accrual mint)
    pub quote_mint: Pubkey,
    /// Base mint
    pub base_mint: Pubkey,
    /// Whether position is active
    pub is_active: bool,
    /// Bump seed for the position PDA
    pub bump: u8,
}

impl HonoraryPosition {
    pub const LEN: usize = 8 + 32 + 8 + 32 + 32 + 1 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct PoolConfig {
    /// Pool ID from CP-AMM
    pub pool_id: Pubkey,
    /// Lower tick for the position
    pub lower_tick: i32,
    /// Upper tick for the position
    pub upper_tick: i32,
    /// Initial liquidity amount
    pub liquidity: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InvestorData {
    /// Investor's quote token ATA
    pub investor_quote_ata: Pubkey,
    /// Streamflow stream pubkey
    pub stream_pubkey: Pubkey,
    /// Still locked amount at current timestamp
    pub locked_amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StreamflowStreamData {
    /// Stream pubkey
    pub stream_pubkey: Pubkey,
    /// Total allocated amount
    pub total_allocated: u64,
    /// Amount still locked
    pub locked_amount: u64,
    /// Last update timestamp
    pub last_update_ts: i64,
}
