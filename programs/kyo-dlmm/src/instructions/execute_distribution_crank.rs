use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use crate::state::*;
use crate::errors::*;
use crate::events::*;

#[derive(Accounts)]
pub struct ExecuteDistributionCrank<'info> {
    /// CP-AMM program
    /// CHECK: Validated by CP-AMM program
    pub cp_amm_program: AccountInfo<'info>,
    
    /// CP-AMM pool
    /// CHECK: Validated by CP-AMM program
    pub pool: AccountInfo<'info>,
    
    /// Honorary position
    #[account(
        mut,
        seeds = [b"honorary_position", pool.key().as_ref()],
        bump = honorary_position.bump
    )]
    pub honorary_position: Account<'info, HonoraryPosition>,
    
    /// Program quote treasury ATA
    #[account(mut)]
    pub program_quote_treasury: Account<'info, TokenAccount>,
    
    /// Creator quote ATA
    #[account(mut)]
    pub creator_quote_ata: Account<'info, TokenAccount>,
    
    /// Streamflow program
    /// CHECK: Validated by Streamflow program
    pub streamflow_program: AccountInfo<'info>,
    
    /// Policy configuration
    #[account(
        mut,
        seeds = [b"policy", pool.key().as_ref()],
        bump = policy.bump
    )]
    pub policy: Account<'info, PolicyConfig>,
    
    /// Distribution progress
    #[account(
        mut,
        seeds = [b"progress", pool.key().as_ref()],
        bump = progress.bump
    )]
    pub progress: Account<'info, DistributionProgress>,
    
    /// Token program
    pub token_program: Program<'info, Token>,
    
    /// System program
    pub system_program: Program<'info, System>,
}

pub fn execute_distribution_crank(
    ctx: Context<ExecuteDistributionCrank>,
    investor_data: Vec<InvestorData>,
    page_size: u8,
) -> Result<()> {
    let clock = Clock::get()?;
    let progress = &mut ctx.accounts.progress;
    let policy = &ctx.accounts.policy;
    
    // Check 24h gate
    let current_time = clock.unix_timestamp;
    let time_since_last = current_time - progress.last_distribution_ts;
    
    if progress.last_distribution_ts > 0 && time_since_last < 86400 {
        return Err(KyoDlmmError::DistributionTooEarly.into());
    }
    
    // Check if this is a new day
    let is_new_day = progress.last_distribution_ts == 0 || time_since_last >= 86400;
    
    if is_new_day {
        // Reset daily counters
        progress.daily_distributed = 0;
        progress.cursor = 0;
        progress.last_distribution_ts = current_time;
    }
    
    // Claim fees from honorary position
    let claimed_fees = claim_fees_from_position(&ctx)?;
    
    // Validate quote-only enforcement
    require!(
        claimed_fees.base_fees == 0,
        KyoDlmmError::BaseFeesDetected
    );
    
    emit!(QuoteFeesClaimed {
        position_key: ctx.accounts.honorary_position.key(),
        claimed_quote_fees: claimed_fees.quote_fees,
        claimed_base_fees: claimed_fees.base_fees,
        timestamp: current_time,
    });
    
    // Calculate investor share
    let total_locked = calculate_total_locked(&investor_data)?;
    let y0 = progress.total_investor_allocation;
    
    if y0 == 0 {
        // First time - set Y0 from current locked amounts
        progress.total_investor_allocation = total_locked;
    }
    
    let f_locked = if y0 > 0 {
        (total_locked as u128 * 10000) / y0 as u128
    } else {
        0
    };
    
    let eligible_investor_share_bps = std::cmp::min(
        policy.investor_fee_share_bps as u128,
        f_locked
    ) as u16;
    
    let investor_fee_quote = (claimed_fees.quote_fees as u128 * eligible_investor_share_bps as u128) / 10000;
    
    // Apply daily cap
    let remaining_cap = if policy.daily_cap_quote > progress.daily_distributed {
        policy.daily_cap_quote - progress.daily_distributed
    } else {
        0
    };
    
    let capped_investor_share = std::cmp::min(investor_fee_quote as u64, remaining_cap);
    
    // Process pagination
    let start_idx = progress.cursor as usize;
    let end_idx = std::cmp::min(
        start_idx + page_size as usize,
        investor_data.len()
    );
    
    let mut total_payout = 0u64;
    let mut processed_count = 0u32;
    
    for i in start_idx..end_idx {
        let investor = &investor_data[i];
        
        // Calculate weight for this investor
        let weight = if total_locked > 0 {
            (investor.locked_amount as u128 * 10000) / total_locked as u128
        } else {
            0
        };
        
        let payout = (capped_investor_share as u128 * weight) / 10000;
        
        if payout >= policy.min_payout_lamports as u128 {
            // Transfer to investor
            transfer_to_investor(&ctx, investor.investor_quote_ata, payout as u64)?;
            total_payout += payout as u64;
        }
        
        processed_count += 1;
    }
    
    // Update progress
    progress.cursor = end_idx as u32;
    progress.daily_distributed += total_payout;
    
    emit!(InvestorPayoutPage {
        page_number: (start_idx / page_size as usize) as u32,
        investors_processed: processed_count,
        total_payout,
        timestamp: current_time,
    });
    
    // Check if this is the final page of the day
    if end_idx >= investor_data.len() {
        // Route remainder to creator
        let creator_share = claimed_fees.quote_fees - total_payout;
        
        if creator_share > 0 {
            transfer_to_creator(&ctx, creator_share)?;
        }
        
        // Update carry over
        progress.carry_over = claimed_fees.quote_fees - total_payout - creator_share;
        
        emit!(CreatorPayoutDayClosed {
            daily_total_claimed: claimed_fees.quote_fees,
            investor_share: total_payout,
            creator_share,
            carry_over: progress.carry_over,
            timestamp: current_time,
        });
    }
    
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct ClaimedFees {
    quote_fees: u64,
    base_fees: u64,
}

fn claim_fees_from_position(ctx: &Context<ExecuteDistributionCrank>) -> Result<ClaimedFees> {
    // This is a placeholder - in practice, you would:
    // 1. Call the CP-AMM program to claim fees from the honorary position
    // 2. Transfer the claimed fees to the program treasury
    // 3. Return the amounts claimed
    
    // For now, return zero fees (this would be replaced with actual CP-AMM integration)
    Ok(ClaimedFees {
        quote_fees: 0,
        base_fees: 0,
    })
}

fn calculate_total_locked(investor_data: &[InvestorData]) -> Result<u64> {
    let mut total = 0u64;
    for investor in investor_data {
        total = total
            .checked_add(investor.locked_amount)
            .ok_or(KyoDlmmError::MathOverflow)?;
    }
    Ok(total)
}

fn transfer_to_investor(
    ctx: &Context<ExecuteDistributionCrank>,
    investor_ata: Pubkey,
    amount: u64,
) -> Result<()> {
    // This would implement the actual transfer logic
    // For now, it's a placeholder
    Ok(())
}

fn transfer_to_creator(
    ctx: &Context<ExecuteDistributionCrank>,
    amount: u64,
) -> Result<()> {
    // This would implement the actual transfer logic
    // For now, it's a placeholder
    Ok(())
}
