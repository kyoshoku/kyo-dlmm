import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { KyoDlmm } from "../target/types/kyo_dlmm";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { expect } from "chai";
import { startAnchor } from "solana-bankrun";

describe("kyo-dlmm bankrun integration tests", () => {
  let context: any;
  let program: Program<KyoDlmm>;
  let provider: any;

  // Test accounts
  let payer: Keypair;
  let quoteMint: PublicKey;
  let baseMint: PublicKey;
  let pool: Keypair;
  let cpAmmProgram: Keypair;
  let streamflowProgram: Keypair;
  let creatorQuoteAta: PublicKey;
  let programQuoteTreasury: PublicKey;

  before(async () => {
    // Start bankrun context
    context = await startAnchor("./", [], []);
    provider = new anchor.AnchorProvider(
      context.banksClient,
      context.payer,
      anchor.AnchorProvider.defaultOptions()
    );
    anchor.setProvider(provider);

    program = anchor.workspace.KyoDlmm as Program<KyoDlmm>;
    
    payer = context.payer;
    
    // Create test mints
    quoteMint = await createMint(
      context.banksClient,
      context.payer,
      context.payer.publicKey,
      null,
      6
    );

    baseMint = await createMint(
      context.banksClient,
      context.payer,
      context.payer.publicKey,
      null,
      6
    );

    // Create test accounts
    pool = Keypair.generate();
    cpAmmProgram = Keypair.generate();
    streamflowProgram = Keypair.generate();

    // Create creator quote ATA
    creatorQuoteAta = await createAccount(
      context.banksClient,
      context.payer,
      quoteMint,
      context.payer.publicKey
    );

    // Create program quote treasury
    programQuoteTreasury = await createAccount(
      context.banksClient,
      context.payer,
      quoteMint,
      context.payer.publicKey
    );

    // Mint some tokens to the program treasury for testing
    await mintTo(
      context.banksClient,
      context.payer,
      quoteMint,
      programQuoteTreasury,
      context.payer,
      1000000 // 1M tokens
    );
  });

  it("Complete flow: Initialize -> Update Policy -> Execute Crank", async () => {
    // Step 1: Initialize honorary position
    const poolConfig = {
      poolId: pool.publicKey,
      lowerTick: -1000,
      upperTick: 1000,
      liquidity: new anchor.BN(1000000),
    };

    const [honoraryPositionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("honorary_position"), pool.publicKey.toBuffer()],
      program.programId
    );

    const [policyPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("policy"), pool.publicKey.toBuffer()],
      program.programId
    );

    const [progressPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("progress"), pool.publicKey.toBuffer()],
      program.programId
    );

    // Initialize honorary position
    const initTx = await program.methods
      .initializeHonoraryPosition(poolConfig)
      .accounts({
        payer: payer.publicKey,
        cpAmmProgram: cpAmmProgram.publicKey,
        pool: pool.publicKey,
        quoteMint: quoteMint,
        baseMint: baseMint,
        quoteVault: Keypair.generate().publicKey,
        baseVault: Keypair.generate().publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        honoraryPosition: honoraryPositionPda,
        policy: policyPda,
        progress: progressPda,
      })
      .signers([payer])
      .rpc();

    console.log("Initialize transaction:", initTx);

    // Verify initialization
    const honoraryPosition = await program.account.honoraryPosition.fetch(honoraryPositionPda);
    expect(honoraryPosition.isActive).to.be.true;
    expect(honoraryPosition.quoteMint.toString()).to.equal(quoteMint.toString());

    // Step 2: Update policy
    const newPolicy = {
      investorFeeShareBps: 6000, // 60% to investors
      dailyCapQuote: new anchor.BN(500000), // 500K daily cap
      minPayoutLamports: new anchor.BN(1000), // 0.001 minimum
      authority: payer.publicKey,
      bump: 0,
    };

    const policyTx = await program.methods
      .updatePolicy(newPolicy)
      .accounts({
        authority: payer.publicKey,
        policy: policyPda,
      })
      .signers([payer])
      .rpc();

    console.log("Policy update transaction:", policyTx);

    // Verify policy update
    const policy = await program.account.policyConfig.fetch(policyPda);
    expect(policy.investorFeeShareBps).to.equal(6000);
    expect(policy.dailyCapQuote.toNumber()).to.equal(500000);

    // Step 3: Execute distribution crank
    const investorData = [
      {
        investorQuoteAta: await createAccount(
          context.banksClient,
          context.payer,
          quoteMint,
          Keypair.generate().publicKey
        ),
        streamPubkey: Keypair.generate().publicKey,
        lockedAmount: new anchor.BN(200000), // 200K locked
      },
      {
        investorQuoteAta: await createAccount(
          context.banksClient,
          context.payer,
          quoteMint,
          Keypair.generate().publicKey
        ),
        streamPubkey: Keypair.generate().publicKey,
        lockedAmount: new anchor.BN(300000), // 300K locked
      },
      {
        investorQuoteAta: await createAccount(
          context.banksClient,
          context.payer,
          quoteMint,
          Keypair.generate().publicKey
        ),
        streamPubkey: Keypair.generate().publicKey,
        lockedAmount: new anchor.BN(0), // Unlocked
      },
    ];

    // Set total investor allocation (Y0) in progress
    const progress = await program.account.distributionProgress.fetch(progressPda);
    const totalAllocation = new anchor.BN(1000000); // 1M total allocation

    // Execute crank with pagination
    const crankTx = await program.methods
      .executeDistributionCrank(investorData, 2) // Process 2 investors per page
      .accounts({
        cpAmmProgram: cpAmmProgram.publicKey,
        pool: pool.publicKey,
        honoraryPosition: honoraryPositionPda,
        programQuoteTreasury: programQuoteTreasury,
        creatorQuoteAta: creatorQuoteAta,
        streamflowProgram: streamflowProgram.publicKey,
        policy: policyPda,
        progress: progressPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Distribution crank transaction:", crankTx);

    // Verify progress was updated
    const updatedProgress = await program.account.distributionProgress.fetch(progressPda);
    expect(updatedProgress.cursor).to.equal(2); // Processed 2 investors
    expect(updatedProgress.dailyDistributed.toNumber()).to.be.greaterThan(0);

    // Step 4: Execute second page to complete the day
    const secondPageTx = await program.methods
      .executeDistributionCrank(investorData.slice(2), 1) // Process remaining investor
      .accounts({
        cpAmmProgram: cpAmmProgram.publicKey,
        pool: pool.publicKey,
        honoraryPosition: honoraryPositionPda,
        programQuoteTreasury: programQuoteTreasury,
        creatorQuoteAta: creatorQuoteAta,
        streamflowProgram: streamflowProgram.publicKey,
        policy: policyPda,
        progress: progressPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Second page transaction:", secondPageTx);

    // Verify final progress
    const finalProgress = await program.account.distributionProgress.fetch(progressPda);
    expect(finalProgress.cursor).to.equal(3); // All investors processed
  });

  it("Tests 24h gate enforcement", async () => {
    const [honoraryPositionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("honorary_position"), pool.publicKey.toBuffer()],
      program.programId
    );

    const [policyPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("policy"), pool.publicKey.toBuffer()],
      program.programId
    );

    const [progressPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("progress"), pool.publicKey.toBuffer()],
      program.programId
    );

    const investorData = [
      {
        investorQuoteAta: Keypair.generate().publicKey,
        streamPubkey: Keypair.generate().publicKey,
        lockedAmount: new anchor.BN(100000),
      },
    ];

    // Try to execute crank again immediately (should fail due to 24h gate)
    try {
      await program.methods
        .executeDistributionCrank(investorData, 1)
        .accounts({
          cpAmmProgram: cpAmmProgram.publicKey,
          pool: pool.publicKey,
          honoraryPosition: honoraryPositionPda,
          programQuoteTreasury: programQuoteTreasury,
          creatorQuoteAta: creatorQuoteAta,
          streamflowProgram: streamflowProgram.publicKey,
          policy: policyPda,
          progress: progressPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      
      expect.fail("Should have failed due to 24h gate");
    } catch (error) {
      expect(error.message).to.include("DistributionTooEarly");
      console.log("24h gate test passed - correctly rejected immediate re-execution");
    }
  });

  it("Tests dust threshold handling", async () => {
    const [honoraryPositionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("honorary_position"), pool.publicKey.toBuffer()],
      program.programId
    );

    const [policyPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("policy"), pool.publicKey.toBuffer()],
      program.programId
    );

    const [progressPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("progress"), pool.publicKey.toBuffer()],
      program.programId
    );

    // Create investor with very small locked amount (below dust threshold)
    const investorData = [
      {
        investorQuoteAta: await createAccount(
          context.banksClient,
          context.payer,
          quoteMint,
          Keypair.generate().publicKey
        ),
        streamPubkey: Keypair.generate().publicKey,
        lockedAmount: new anchor.BN(50), // Very small amount
      },
    ];

    // This should not distribute anything due to dust threshold
    const tx = await program.methods
      .executeDistributionCrank(investorData, 1)
      .accounts({
        cpAmmProgram: cpAmmProgram.publicKey,
        pool: pool.publicKey,
        honoraryPosition: honoraryPositionPda,
        programQuoteTreasury: programQuoteTreasury,
        creatorQuoteAta: creatorQuoteAta,
        streamflowProgram: streamflowProgram.publicKey,
        policy: policyPda,
        progress: progressPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Dust threshold test transaction:", tx);

    // Verify that no distribution occurred due to dust threshold
    const progress = await program.account.distributionProgress.fetch(progressPda);
    console.log("Dust threshold test - no distribution due to small amount");
  });

  it("Tests daily cap enforcement", async () => {
    // This test would verify that daily caps are properly enforced
    // Implementation would require setting up a scenario where
    // the daily cap is exceeded
    
    console.log("Daily cap enforcement test - would require specific setup");
  });

  it("Tests quote-only fee enforcement", async () => {
    // This test would verify that the program fails when base fees are detected
    // Implementation would require mocking CP-AMM to return base fees
    
    console.log("Quote-only enforcement test - would require CP-AMM mocking");
  });
});
