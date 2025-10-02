use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};

declare_id!("DLMM1111111111111111111111111111111111111");

pub mod errors;
pub mod instructions;
pub mod state;
pub mod events;

use instructions::*;
use state::*;
use events::*;

#[program]
pub mod kyo_dlmm {
    use super::*;

    /// Initialize the honorary fee position for quote-only fee accrual
    pub fn initialize_honorary_position(
        ctx: Context<InitializeHonoraryPosition>,
        pool_config: PoolConfig,
    ) -> Result<()> {
        instructions::initialize_honorary_position(ctx, pool_config)
    }

    /// Execute the 24h distribution crank with pagination support
    pub fn execute_distribution_crank(
        ctx: Context<ExecuteDistributionCrank>,
        investor_data: Vec<InvestorData>,
        page_size: u8,
    ) -> Result<()> {
        instructions::execute_distribution_crank(ctx, investor_data, page_size)
    }

    /// Update policy configuration
    pub fn update_policy(
        ctx: Context<UpdatePolicy>,
        new_policy: PolicyConfig,
    ) -> Result<()> {
        instructions::update_policy(ctx, new_policy)
    }
}
