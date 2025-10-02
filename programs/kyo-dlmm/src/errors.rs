use anchor_lang::prelude::*;

#[error_code]
pub enum KyoDlmmError {
    #[msg("Invalid pool configuration - position would accrue base fees")]
    InvalidPoolConfig,
    
    #[msg("Quote-only fee enforcement failed - base fees detected")]
    BaseFeesDetected,
    
    #[msg("24h gate not satisfied - too early for next distribution")]
    DistributionTooEarly,
    
    #[msg("Daily cap exceeded")]
    DailyCapExceeded,
    
    #[msg("Invalid investor data provided")]
    InvalidInvestorData,
    
    #[msg("Position not active")]
    PositionNotActive,
    
    #[msg("Invalid authority")]
    InvalidAuthority,
    
    #[msg("Math overflow in distribution calculation")]
    MathOverflow,
    
    #[msg("Invalid mint configuration")]
    InvalidMintConfig,
    
    #[msg("Distribution already completed for this day")]
    DistributionAlreadyCompleted,
    
    #[msg("Invalid pagination cursor")]
    InvalidCursor,
    
    #[msg("Insufficient quote fees to distribute")]
    InsufficientFees,
    
    #[msg("Streamflow data validation failed")]
    StreamflowValidationFailed,
}
