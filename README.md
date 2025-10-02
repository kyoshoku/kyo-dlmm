# DAMM v2 Honorary Fee Position + 24h Distribution Crank

A standalone Anchor-compatible module for managing honorary DAMM v2 LP positions that accrue fees exclusively in the quote mint, with automated 24-hour distribution to investors based on their still-locked amounts from Streamflow.

## Overview

This program creates and manages an "honorary" DAMM v2 LP position owned by a program PDA that accrues fees in the quote mint only. It provides a permissionless, once-per-24h crank that claims those quote fees and distributes them to investors pro-rata to still-locked amounts, with the remainder routed to the creator wallet.

## Features

- **Quote-only fee accrual**: Honorary position configured to accrue fees exclusively in the quote mint
- **Program ownership**: Fee position owned by program PDA for autonomous operation
- **24h distribution crank**: Permissionless, paginated distribution system with 24-hour gating
- **Pro-rata distribution**: Investors receive fees based on their still-locked Streamflow amounts
- **Dust handling**: Minimum payout thresholds and carry-over logic
- **Daily caps**: Configurable daily distribution limits
- **Idempotent pagination**: Safe to resume mid-day after partial success

## Program Architecture

### Core Accounts

#### Policy Configuration (`PolicyConfig`)
- `investor_fee_share_bps`: Fee share for investors (0-10000 basis points)
- `daily_cap_quote`: Daily distribution cap in quote tokens
- `min_payout_lamports`: Minimum payout threshold to avoid dust
- `authority`: Account that can update policy settings

#### Distribution Progress (`DistributionProgress`)
- `last_distribution_ts`: Timestamp of last distribution
- `daily_distributed`: Amount distributed in current day
- `carry_over`: Unused fees carried to next day
- `cursor`: Pagination cursor for investor processing
- `total_investor_allocation`: Total investor allocation at TGE (Y0)

#### Honorary Position (`HonoraryPosition`)
- `position_key`: CP-AMM position identifier
- `pool_config`: Pool configuration (ticks, liquidity)
- `quote_mint`: Quote token mint
- `base_mint`: Base token mint
- `is_active`: Whether position is active

### Instructions

#### 1. Initialize Honorary Position
```rust
pub fn initialize_honorary_position(
    ctx: Context<InitializeHonoraryPosition>,
    pool_config: PoolConfig,
) -> Result<()>
```

Creates an empty DAMM v2 position owned by the program PDA that accrues only quote-token fees.

**Required Accounts:**
- `payer`: Account paying for initialization
- `cp_amm_program`: CP-AMM program ID
- `pool`: CP-AMM pool account
- `quote_mint`: Quote token mint
- `base_mint`: Base token mint
- `quote_vault`: Pool's quote token vault
- `base_vault`: Pool's base token vault
- `honorary_position`: Program PDA for position
- `policy`: Policy configuration PDA
- `progress`: Distribution progress PDA

#### 2. Execute Distribution Crank
```rust
pub fn execute_distribution_crank(
    ctx: Context<ExecuteDistributionCrank>,
    investor_data: Vec<InvestorData>,
    page_size: u8,
) -> Result<()>
```

Claims fees from honorary position and distributes to investors with pagination support.

**Required Accounts:**
- `cp_amm_program`: CP-AMM program ID
- `pool`: CP-AMM pool account
- `honorary_position`: Honorary position PDA
- `program_quote_treasury`: Program's quote token treasury
- `creator_quote_ata`: Creator's quote token ATA
- `streamflow_program`: Streamflow program ID
- `policy`: Policy configuration PDA
- `progress`: Distribution progress PDA

#### 3. Update Policy
```rust
pub fn update_policy(
    ctx: Context<UpdatePolicy>,
    new_policy: PolicyConfig,
) -> Result<()>
```

Updates policy configuration (fee share, caps, thresholds).

## Integration Guide

### Prerequisites

1. **CP-AMM Integration**: Access to CP-AMM program and pool accounts
2. **Streamflow Integration**: Access to Streamflow program for locked amount queries
3. **Token Setup**: Quote and base token mints and ATAs

### Setup Steps

#### 1. Deploy the Program
```bash
anchor build
anchor deploy
```

#### 2. Initialize Honorary Position

```typescript
import { PublicKey } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";

// Get program instance
const program = anchor.workspace.KyoDlmm as Program<KyoDlmm>;

// Define pool configuration
const poolConfig = {
  poolId: cpAmmPoolPubkey,
  lowerTick: -1000,  // Configure for quote-only fees
  upperTick: 1000,
  liquidity: new BN(1000000),
};

// Derive PDAs
const [honoraryPositionPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("honorary_position"), poolPubkey.toBuffer()],
  program.programId
);

const [policyPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("policy"), poolPubkey.toBuffer()],
  program.programId
);

const [progressPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("progress"), poolPubkey.toBuffer()],
  program.programId
);

// Initialize
const tx = await program.methods
  .initializeHonoraryPosition(poolConfig)
  .accounts({
    payer: payer.publicKey,
    cpAmmProgram: cpAmmProgramId,
    pool: poolPubkey,
    quoteMint: quoteMintPubkey,
    baseMint: baseMintPubkey,
    quoteVault: quoteVaultPubkey,
    baseVault: baseVaultPubkey,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    rent: SYSVAR_RENT_PUBKEY,
    honoraryPosition: honoraryPositionPda,
    policy: policyPda,
    progress: progressPda,
  })
  .signers([payer])
  .rpc();
```

#### 3. Configure Policy

```typescript
// Update policy settings
const newPolicy = {
  investorFeeShareBps: 5000, // 50% to investors
  dailyCapQuote: new BN(1000000), // 1M quote tokens daily cap
  minPayoutLamports: new BN(1000), // 0.001 minimum payout
  authority: authorityPubkey,
  bump: 0, // Set by program
};

const tx = await program.methods
  .updatePolicy(newPolicy)
  .accounts({
    authority: authorityPubkey,
    policy: policyPda,
  })
  .signers([authority])
  .rpc();
```

#### 4. Execute Distribution Crank

```typescript
// Prepare investor data
const investorData = [
  {
    investorQuoteAta: investor1QuoteAta,
    streamPubkey: streamflowStream1Pubkey,
    lockedAmount: new BN(100000), // Still locked amount
  },
  {
    investorQuoteAta: investor2QuoteAta,
    streamPubkey: streamflowStream2Pubkey,
    lockedAmount: new BN(200000),
  },
  // ... more investors
];

// Execute crank with pagination
const pageSize = 10; // Process 10 investors per call

const tx = await program.methods
  .executeDistributionCrank(investorData, pageSize)
  .accounts({
    cpAmmProgram: cpAmmProgramId,
    pool: poolPubkey,
    honoraryPosition: honoraryPositionPda,
    programQuoteTreasury: programQuoteTreasuryPubkey,
    creatorQuoteAta: creatorQuoteAtaPubkey,
    streamflowProgram: streamflowProgramId,
    policy: policyPda,
    progress: progressPda,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### Account Tables

#### Required Accounts for Initialization

| Account | Type | Description |
|---------|------|-------------|
| `payer` | Signer | Account paying for initialization |
| `cp_amm_program` | Program | CP-AMM program ID |
| `pool` | Account | CP-AMM pool account |
| `quote_mint` | Mint | Quote token mint |
| `base_mint` | Mint | Base token mint |
| `quote_vault` | Account | Pool's quote token vault |
| `base_vault` | Account | Pool's base token vault |
| `honorary_position` | PDA | Honorary position account |
| `policy` | PDA | Policy configuration account |
| `progress` | PDA | Distribution progress account |

#### Required Accounts for Distribution Crank

| Account | Type | Description |
|---------|------|-------------|
| `cp_amm_program` | Program | CP-AMM program ID |
| `pool` | Account | CP-AMM pool account |
| `honorary_position` | PDA | Honorary position account |
| `program_quote_treasury` | TokenAccount | Program's quote treasury |
| `creator_quote_ata` | TokenAccount | Creator's quote ATA |
| `streamflow_program` | Program | Streamflow program ID |
| `policy` | PDA | Policy configuration account |
| `progress` | PDA | Distribution progress account |

### Error Codes

| Error | Code | Description |
|-------|------|-------------|
| `InvalidPoolConfig` | 6000 | Pool configuration would accrue base fees |
| `BaseFeesDetected` | 6001 | Base fees detected in honorary position |
| `DistributionTooEarly` | 6002 | 24h gate not satisfied |
| `DailyCapExceeded` | 6003 | Daily distribution cap exceeded |
| `InvalidInvestorData` | 6004 | Invalid investor data provided |
| `PositionNotActive` | 6005 | Honorary position not active |
| `InvalidAuthority` | 6006 | Invalid authority for operation |
| `MathOverflow` | 6007 | Math overflow in calculation |
| `InvalidMintConfig` | 6008 | Invalid mint configuration |
| `DistributionAlreadyCompleted` | 6009 | Distribution already completed for day |
| `InvalidCursor` | 6010 | Invalid pagination cursor |
| `InsufficientFees` | 6011 | Insufficient quote fees to distribute |
| `StreamflowValidationFailed` | 6012 | Streamflow data validation failed |

### Events

#### HonoraryPositionInitialized
Emitted when honorary position is successfully initialized.

```rust
pub struct HonoraryPositionInitialized {
    pub position_key: Pubkey,
    pub pool_id: Pubkey,
    pub quote_mint: Pubkey,
    pub base_mint: Pubkey,
    pub timestamp: i64,
}
```

#### QuoteFeesClaimed
Emitted when fees are claimed from honorary position.

```rust
pub struct QuoteFeesClaimed {
    pub position_key: Pubkey,
    pub claimed_quote_fees: u64,
    pub claimed_base_fees: u64,
    pub timestamp: i64,
}
```

#### InvestorPayoutPage
Emitted for each page of investor payouts.

```rust
pub struct InvestorPayoutPage {
    pub page_number: u32,
    pub investors_processed: u32,
    pub total_payout: u64,
    pub timestamp: i64,
}
```

#### CreatorPayoutDayClosed
Emitted when daily distribution is completed.

```rust
pub struct CreatorPayoutDayClosed {
    pub daily_total_claimed: u64,
    pub investor_share: u64,
    pub creator_share: u64,
    pub carry_over: u64,
    pub timestamp: i64,
}
```

### Day and Pagination Semantics

#### 24-Hour Gating
- First crank in a day requires `now >= last_distribution_ts + 86400`
- Subsequent pages within the same day share the same "day" window
- New day resets daily counters and cursor

#### Pagination
- `cursor` tracks current position in investor list
- `page_size` determines how many investors to process per call
- Safe to resume mid-day after partial success
- Final page routes remainder to creator

#### Math Formulas

**Investor Share Calculation:**
```
Y0 = total_investor_allocation (set at TGE)
locked_total(t) = sum of still_locked across investors at time t
f_locked(t) = locked_total(t) / Y0 in [0, 1]
eligible_investor_share_bps = min(investor_fee_share_bps, floor(f_locked(t) * 10000))
investor_fee_quote = floor(claimed_quote * eligible_investor_share_bps / 10000)
```

**Pro-rata Distribution:**
```
weight_i(t) = locked_i(t) / locked_total(t)
payout_i = floor(investor_fee_quote * weight_i(t))
```

### Testing

Run the test suite:

```bash
# Build the program
anchor build

# Run tests
anchor test

# Run tests with local validator
anchor test --skip-local-validator
```

### Security Considerations

1. **Quote-only enforcement**: Program validates that honorary position only accrues quote fees
2. **24h gating**: Prevents excessive distribution frequency
3. **Authority controls**: Policy updates require proper authority
4. **Math safety**: All calculations use checked arithmetic to prevent overflow
5. **Deterministic failure**: Base fee detection causes clean failure without distribution

### Deployment

1. **Local Development**:
   ```bash
   anchor build
   anchor test
   ```

2. **Devnet Deployment**:
   ```bash
   anchor build
   anchor deploy --provider.cluster devnet
   ```

3. **Mainnet Deployment**:
   ```bash
   anchor build
   anchor deploy --provider.cluster mainnet
   ```

### Integration Checklist

- [ ] CP-AMM program and pool accounts configured
- [ ] Streamflow program integration ready
- [ ] Quote and base token mints created
- [ ] Creator quote ATA created
- [ ] Program quote treasury ATA created
- [ ] Policy configuration set
- [ ] Investor data structure prepared
- [ ] Error handling implemented
- [ ] Event listeners configured
- [ ] Testing completed

## License

This project is licensed under the MIT License - see the LICENSE file for details.
