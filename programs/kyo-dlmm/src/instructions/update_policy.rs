use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

#[derive(Accounts)]
pub struct UpdatePolicy<'info> {
    /// Authority that can update policy
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// Policy configuration
    #[account(
        mut,
        seeds = [b"policy", policy.pool_id().as_ref()],
        bump = policy.bump,
        has_one = authority
    )]
    pub policy: Account<'info, PolicyConfig>,
}

pub fn update_policy(
    ctx: Context<UpdatePolicy>,
    new_policy: PolicyConfig,
) -> Result<()> {
    let clock = Clock::get()?;
    let policy = &mut ctx.accounts.policy;
    
    // Validate new policy values
    require!(
        new_policy.investor_fee_share_bps <= 10000,
        KyoDlmmError::InvalidAuthority
    );
    
    require!(
        new_policy.min_payout_lamports > 0,
        KyoDlmmError::InvalidAuthority
    );
    
    // Update policy
    let old_investor_share = policy.investor_fee_share_bps;
    let old_daily_cap = policy.daily_cap_quote;
    let old_min_payout = policy.min_payout_lamports;
    
    policy.investor_fee_share_bps = new_policy.investor_fee_share_bps;
    policy.daily_cap_quote = new_policy.daily_cap_quote;
    policy.min_payout_lamports = new_policy.min_payout_lamports;
    
    emit!(PolicyUpdated {
        new_investor_fee_share_bps: new_policy.investor_fee_share_bps,
        new_daily_cap_quote: new_policy.daily_cap_quote,
        new_min_payout_lamports: new_policy.min_payout_lamports,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}
