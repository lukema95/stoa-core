// SPDX-License-Identifier: MIT
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Stoa } from "../target/types/stoa";
import { BN } from "bn.js";
import { assert, expect } from "chai";

import {
  Connection,
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
  ComputeBudgetProgram,
  SYSVAR_RENT_PUBKEY,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";

import {
  ExtensionType,
  createMint,
  getMint,
  getMintLen,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddress,
  createInitializeMintInstruction,
  createAssociatedTokenAccountInstruction,
  createInitializeMetadataPointerInstruction,
  getMetadataPointerState,
  getTokenMetadata,
  TYPE_SIZE,
  LENGTH_SIZE,
  TOKEN_2022_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAccount,
} from "@solana/spl-token";

import {
  createInitializeInstruction,
  createUpdateFieldInstruction,
  createRemoveKeyInstruction,
  pack,
  TokenMetadata,
} from "@solana/spl-token-metadata";

const curveSeed = "CurveConfiguration";
const POOL_SEED_PREFIX = "liquidity_pool";
const SOL_VAULT_PREFIX = "liquidity_sol_vault";
const SOL_FEE_PREFIX = "liquidity_sol_fee_vault";
const tokenDecimals = 9;

// Configure the client to use the local cluster.
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.Stoa as Program<Stoa>;

describe("stoa", () => {
  const admin = provider.wallet.publicKey;
  console.log("admin public key", admin);

  // Test keypairs are generated at runtime. Never commit private keys.
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();

  const [curveConfiguration, curveConfigurationBump] = PublicKey.findProgramAddressSync(
    [Buffer.from(curveSeed)],
    program.programId,
  );
  console.log("curveConfiguration", curveConfiguration);

  const curveConfigurationPda = curveConfiguration;

  let tokenMintPda: PublicKey;
  let tokenMintPad2: PublicKey;
  let poolPda: PublicKey;
  let poolPda2: PublicKey;
  let poolTokenAccountPda: PublicKey;
  let poolTokenAccountPda2: PublicKey;
  let poolSolVaultPda: PublicKey;
  let poolSolVaultPda2: PublicKey;
  let poolSolFeeVaultPda: PublicKey;
  let poolSolFeeVaultPda2: PublicKey;

  it("Initialize Contract", async () => {
    await initializeContract(curveConfigurationPda, admin);
  });

  it("Airdrop SOL", async () => {
    await airdrop(provider.connection, user1.publicKey, 200);
    await airdrop(provider.connection, user2.publicKey, 200);

    // const user1Balance = await provider.connection.getBalance(user1.publicKey);
    // console.log("user1Balance", user1Balance);

    // const user2Balance = await provider.connection.getBalance(user2.publicKey);
    // console.log("user2Balance", user2Balance);

    // const adminBalance = await provider.connection.getBalance(admin);
    // console.log("adminBalance", adminBalance);
  });

  it("Create TokenMint, Metadata and PDA", async () => {
    try {
      const mint = await createTokenMint(provider.connection, user1);
      const mint2 = await createTokenMint(provider.connection, user2);

      const { pool, poolTokenAccount, poolSolVault, poolSolFeeVault } = getPda(
        curveSeed,
        mint,
        program.programId,
      );

      const {
        pool: pool2,
        poolTokenAccount: poolTokenAccount2,
        poolSolVault: poolSolVault2,
        poolSolFeeVault: poolSolFeeVault2,
      } = getPda(curveSeed, mint2, program.programId);

      tokenMintPda = mint;
      tokenMintPad2 = mint2;
      poolPda = pool;
      poolPda2 = pool2;
      poolTokenAccountPda = poolTokenAccount;
      poolTokenAccountPda2 = poolTokenAccount2;
      poolSolVaultPda = poolSolVault;
      poolSolVaultPda2 = poolSolVault2;
      poolSolFeeVaultPda = poolSolFeeVault;
      poolSolFeeVaultPda2 = poolSolFeeVault2;
    } catch (e) {
      console.error("Error Initializing PDA", e);
      assert.fail(`Error Initializing PDA: ${e}`);
    }
  });

  it("Create Pool", async () => {
    // Create Pool 1 by user 1
    console.log("===========Create Pool 1===========");
    await createPool(
      provider.connection,
      user1,
      curveConfigurationPda,
      poolPda,
      tokenMintPda,
      poolTokenAccountPda,
      poolSolVaultPda,
      poolSolFeeVaultPda,
    );

    // Create Pool 2 by user 2
    console.log("===========Create Pool 2===========");
    await createPool(
      provider.connection,
      user2,
      curveConfigurationPda,
      poolPda2,
      tokenMintPad2,
      poolTokenAccountPda2,
      poolSolVaultPda2,
      poolSolFeeVaultPda2,
    );
  });

  it("Distribute Fee", async () => {
    try {
      // Distribute User Fee
      await distributeUserFee(program, poolPda, tokenMintPda, user1, user2.publicKey);

      // Distribute Pool Fee
      await distributePoolFee(program, poolPda, tokenMintPda, user1, poolPda2, tokenMintPad2);

      const poolInfo = await program.account.pool.fetch(poolPda);
      console.log("Pool info after distribute pool fee", poolInfo);
    } catch (e) {
      console.error("Error distributing fee", e);
      assert.fail(`Error distributing fee: ${e}`);
    }
  });

  it("Update Fee Recipient At Capacity", async () => {
    // Fill the table to the maximum with zero-share recipients.
    for (let i = 0; i < 8; i += 1) {
      await distributeUserFee(
        program,
        poolPda,
        tokenMintPda,
        user1,
        Keypair.generate().publicKey,
        0,
      );
    }

    let poolInfo = await program.account.pool.fetch(poolPda);
    assert.equal(poolInfo.feeRecipients.length, 10);

    // Updating an existing recipient must still work when the table is full.
    await distributeUserFee(program, poolPda, tokenMintPda, user1, user2.publicKey, 15);

    poolInfo = await program.account.pool.fetch(poolPda);
    assert.equal(poolInfo.totalFeeSharePercentage, 25);
  });

  it("Buy Token", async () => {
    try {
      const poolSolVaultBalanceBefore = await provider.connection.getBalance(poolSolVaultPda);
      console.log("poolSolVaultBalanceBefore", poolSolVaultBalanceBefore);

      const poolSolFeeVaultBalanceBefore = await provider.connection.getBalance(poolSolFeeVaultPda);
      console.log("poolSolFeeVaultBalanceBefore", poolSolFeeVaultBalanceBefore);

      await buyToken(
        program,
        provider.connection,
        user2,
        curveConfigurationPda,
        poolPda,
        tokenMintPda,
        poolTokenAccountPda,
        poolSolVaultPda,
        poolSolFeeVaultPda,
        admin,
        user1.publicKey,
        100,
      );

      const poolInfo = await program.account.pool.fetch(poolPda);
      console.log("Pool info after buy", poolInfo);

      const poolSolVaultBalanceAfter = await provider.connection.getBalance(poolSolVaultPda);
      console.log("poolSolVaultBalanceAfter", poolSolVaultBalanceAfter);
      expect(poolSolVaultBalanceAfter).to.be.greaterThan(poolSolVaultBalanceBefore);

      const poolSolFeeVaultBalanceAfter = await provider.connection.getBalance(poolSolFeeVaultPda);
      console.log("poolSolFeeVaultBalanceAfter", poolSolFeeVaultBalanceAfter);
      expect(poolSolFeeVaultBalanceAfter).to.be.greaterThan(poolSolFeeVaultBalanceBefore);
    } catch (e) {
      console.error("Error buying token", e);
      assert.fail(`Error buying token: ${e}`);
    }
  });

  it("Sell Token", async () => {
    try {
      const poolSolVaultBalanceBefore = await provider.connection.getBalance(poolSolVaultPda);
      console.log("poolSolVaultBalanceBefore", poolSolVaultBalanceBefore);

      const poolSolFeeVaultBalanceBefore = await provider.connection.getBalance(poolSolFeeVaultPda);
      console.log("poolSolFeeVaultBalanceBefore", poolSolFeeVaultBalanceBefore);

      await sellToken(
        program,
        provider.connection,
        user2,
        curveConfigurationPda,
        poolPda,
        tokenMintPda,
        poolTokenAccountPda,
        poolSolVaultPda,
        poolSolFeeVaultPda,
        admin,
        user1.publicKey,
        50,
      );

      const poolInfo = await program.account.pool.fetch(poolPda);
      console.log("Pool info after sell", poolInfo);

      const poolSolVaultBalanceAfter = await provider.connection.getBalance(poolSolVaultPda);
      console.log("poolSolVaultBalanceAfter", poolSolVaultBalanceAfter);
      expect(poolSolVaultBalanceAfter).to.be.lessThan(poolSolVaultBalanceBefore);

      const poolSolFeeVaultBalanceAfter = await provider.connection.getBalance(poolSolFeeVaultPda);
      console.log("poolSolFeeVaultBalanceAfter", poolSolFeeVaultBalanceAfter);
      expect(poolSolFeeVaultBalanceAfter).to.be.greaterThan(poolSolFeeVaultBalanceBefore);
    } catch (e) {
      console.error("Error selling token", e);
      assert.fail(`Error selling token: ${e}`);
    }
  });

  it("Claim Fee", async () => {
    try {
      // Start Claim User Fee
      let poolSolFeeVaultBalanceBefore = await provider.connection.getBalance(poolSolFeeVaultPda);
      console.log("poolSolFeeVaultBalanceBefore", poolSolFeeVaultBalanceBefore);

      let user2SolBalanceBefore = await provider.connection.getBalance(user2.publicKey);
      console.log("user2SolBalanceBefore", user2SolBalanceBefore);

      await claimUserFee(
        program,
        user2,
        poolPda,
        tokenMintPda,
        poolSolFeeVaultPda,
        user2.publicKey,
      );

      const poolSolFeeVaultBalanceAfter = await provider.connection.getBalance(poolSolFeeVaultPda);
      console.log("poolSolFeeVaultBalanceAfter", poolSolFeeVaultBalanceAfter);
      expect(poolSolFeeVaultBalanceAfter).to.be.lessThan(poolSolFeeVaultBalanceBefore);

      const user2SolBalanceAfter = await provider.connection.getBalance(user2.publicKey);
      console.log("user2SolBalanceAfter", user2SolBalanceAfter);
      expect(user2SolBalanceAfter).to.be.greaterThan(user2SolBalanceBefore);

      // Start Claim Pool Fee
      let pool2Info = await program.account.pool.fetch(poolPda2);
      console.log("pool2Info before claim pool fee", pool2Info);

      await claimPoolFee(
        program,
        provider.connection,
        user2,
        poolPda,
        poolSolFeeVaultPda,
        tokenMintPda,
        curveConfigurationPda,
        poolPda2,
        tokenMintPad2,
        poolTokenAccountPda2,
        poolSolVaultPda2,
        poolSolFeeVaultPda2,
        admin,
        user2.publicKey,
      );

      pool2Info = await program.account.pool.fetch(poolPda2);
      console.log("pool2Info after claim pool fee", pool2Info);

      const claimedTokenAccount = getAssociatedTokenAddressSync(
        tokenMintPad2,
        user2.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
      );
      const claimedBalance = await provider.connection.getTokenAccountBalance(claimedTokenAccount);
      console.log("claimed token balance", claimedBalance.value.uiAmount);
      expect(claimedBalance.value.uiAmount).to.be.greaterThan(0);
    } catch (e) {
      console.error("Error claiming user fee", e);
      assert.fail(`Error claiming user fee: ${e}`);
    }
  });
});
async function claimPoolFee(
  program: Program<Stoa>,
  connection: Connection,
  user: Keypair,
  poolPda: PublicKey,
  poolSolFeeVault: PublicKey,
  tokenMint: PublicKey,
  curveConfigurationPda: PublicKey,
  distributionPool: PublicKey,
  distributionPoolTokenMint: PublicKey,
  distributionPoolTokenAccount: PublicKey,
  distributionPoolSolVault: PublicKey,
  distributionPoolSolFeeVault: PublicKey,
  protocolAccount: PublicKey,
  distributionPoolCreator: PublicKey,
) {
  console.log("===========Claim Pool Fee===========");
  try {
    // check if user has token account
    const distributionPoolUserTokenAccount = getAssociatedTokenAddressSync(
      distributionPoolTokenMint,
      user.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
    const accountInfo = await connection.getAccountInfo(distributionPoolUserTokenAccount);
    if (accountInfo === null) {
      console.log("distributionPoolUserTokenAccount does not exist, creating...");
      const createUserTokenAccountIx = createAssociatedTokenAccountInstruction(
        user.publicKey,
        distributionPoolUserTokenAccount,
        user.publicKey,
        distributionPoolTokenMint,
        TOKEN_2022_PROGRAM_ID,
      );
      const createAtaTx = new Transaction().add(createUserTokenAccountIx);
      await sendAndConfirmTransaction(provider.connection, createAtaTx, [user], {
        skipPreflight: true,
      });
    }

    await program.methods
      .claimPoolFees()
      .accounts({
        pool: poolPda,
        poolSolFeeVault: poolSolFeeVault,
        tokenMint: tokenMint,
        curveConfigurationAccount: curveConfigurationPda,
        distributionPool: distributionPool,
        distributionPoolTokenMint: distributionPoolTokenMint,
        distributionPoolTokenAccount: distributionPoolTokenAccount,
        distributionPoolSolVault: distributionPoolSolVault,
        distributionPoolSolFeeVault: distributionPoolSolFeeVault,
        distributionPoolUserTokenAccount: distributionPoolUserTokenAccount,
        protocolAccount: protocolAccount,
        distributionPoolCreator: distributionPoolCreator,
        user: user.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([user])
      .rpc();
  } catch (e) {
    console.error("Error claiming pool fee", e);
    assert.fail(`Error claiming pool fee: ${e}`);
  }
}

async function claimUserFee(
  program: Program<Stoa>,
  user: Keypair,
  poolPda: PublicKey,
  tokenMint: PublicKey,
  poolSolFeeVault: PublicKey,
  feeRecipient: PublicKey,
) {
  console.log("===========Claim User Fee===========");
  try {
    await program.methods
      .claimUserFees()
      .accounts({
        pool: poolPda,
        tokenMint: tokenMint,
        poolSolFeeVault: poolSolFeeVault,
        feeRecipient: feeRecipient,
        user: user.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([user])
      .rpc();
  } catch (e) {
    console.error("Error claiming user fee", e);
    assert.fail(`Error claiming user fee: ${e}`);
  }
}

async function sellToken(
  program: Program<Stoa>,
  connection: Connection,
  seller: Keypair,
  curveConfigurationPda: PublicKey,
  poolPda: PublicKey,
  tokenMint: PublicKey,
  poolTokenAccountPda: PublicKey,
  poolSolVaultPda: PublicKey,
  poolSolFeeVaultPda: PublicKey,
  protocolAccount: PublicKey,
  poolCreator: PublicKey,
  amount: number,
) {
  console.log("===========Sell Token===========");
  try {
    const sellerTokenAccount = getAssociatedTokenAddressSync(
      tokenMint,
      seller.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
    const accountInfo = await connection.getAccountInfo(sellerTokenAccount);
    if (accountInfo === null) {
      console.log("seller's token account does not exist");
      throw new Error("seller's token account does not exist");
    }

    await program.methods
      .sell(new BN(amount * LAMPORTS_PER_SOL))
      .accounts({
        curveConfigurationAccount: curveConfigurationPda,
        pool: poolPda,
        tokenMint: tokenMint,
        poolTokenAccount: poolTokenAccountPda,
        poolSolVault: poolSolVaultPda,
        poolSolFeeVault: poolSolFeeVaultPda,
        userTokenAccount: sellerTokenAccount,
        protocolAccount: protocolAccount,
        poolCreator: poolCreator,
        user: seller.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([seller])
      .rpc();
  } catch (e) {
    console.error("Error selling token", e);
    assert.fail(`Error selling token: ${e}`);
  }
}

async function buyToken(
  program: Program<Stoa>,
  connection: Connection,
  buyer: Keypair,
  curveConfigurationPda: PublicKey,
  poolPda: PublicKey,
  tokenMint: PublicKey,
  poolTokenAccountPda: PublicKey,
  poolSolVaultPda: PublicKey,
  poolSolFeeVaultPda: PublicKey,
  protocolAccount: PublicKey,
  poolCreator: PublicKey,
  amount: number,
) {
  console.log("===========Buy Token===========");
  try {
    // check if buyer has token account
    const buyerTokenAccount = getAssociatedTokenAddressSync(
      tokenMint,
      buyer.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
    const accountInfo = await connection.getAccountInfo(buyerTokenAccount);
    if (accountInfo === null) {
      console.log("buyer's token account does not exist, creating...");
      const createUserTokenAccountIx = createAssociatedTokenAccountInstruction(
        buyer.publicKey,
        buyerTokenAccount,
        buyer.publicKey,
        tokenMint,
        TOKEN_2022_PROGRAM_ID,
      );
      const createAtaTx = new Transaction().add(createUserTokenAccountIx);
      await sendAndConfirmTransaction(provider.connection, createAtaTx, [buyer], {
        skipPreflight: true,
      });
    }

    await program.methods
      .buy(new BN(amount * LAMPORTS_PER_SOL))
      .accounts({
        curveConfigurationAccount: curveConfigurationPda,
        pool: poolPda,
        tokenMint: tokenMint,
        poolTokenAccount: poolTokenAccountPda,
        poolSolVault: poolSolVaultPda,
        poolSolFeeVault: poolSolFeeVaultPda,
        userTokenAccount: buyerTokenAccount,
        protocolAccount: protocolAccount,
        poolCreator: poolCreator,
        user: buyer.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([buyer])
      .rpc();
  } catch (e) {
    console.error("Error buying token", e);
    assert.fail(`Error buying token: ${e}`);
  }
}

async function distributePoolFee(
  program: Program<Stoa>,
  poolPda: PublicKey,
  tokenMint: PublicKey,
  distributor: Keypair,
  distributePool: PublicKey,
  distributePoolTokenMint: PublicKey,
) {
  console.log("===========Distribute Pool Fee===========");
  try {
    await program.methods
      .distributePoolFee(10)
      .accounts({
        pool: poolPda,
        tokenMint: tokenMint,
        distributeFeePool: distributePool,
        distributeFeePoolTokenMint: distributePoolTokenMint,
        user: distributor.publicKey,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([distributor])
      .rpc();
  } catch (e) {
    console.error("Error distributing pool fee", e);
    assert.fail(`Error distributing pool fee: ${e}`);
  }
}

async function distributeUserFee(
  program: Program<Stoa>,
  poolPda: PublicKey,
  tokenMint: PublicKey,
  distributor: Keypair,
  recipient: PublicKey,
  sharePercentage = 10,
) {
  console.log("===========Distribute User Fee===========");
  try {
    await program.methods
      .distributeUserFee(sharePercentage)
      .accounts({
        pool: poolPda,
        tokenMint: tokenMint,
        recipient: recipient,
        user: distributor.publicKey,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([distributor])
      .rpc();
  } catch (e) {
    console.error("Error distributing user fee", e);
    assert.fail(`Error distributing user fee: ${e}`);
  }
}

async function createPool(
  connection: Connection,
  user: Keypair,
  curveConfigurationPda: PublicKey,
  poolPda: PublicKey,
  tokenMint: PublicKey,
  poolTokenAccountPda: PublicKey,
  poolSolVaultPda: PublicKey,
  poolSolFeeVaultPda: PublicKey,
) {
  console.log("===========Create Pool===========");
  try {
    const creatorTokenAccount = getAssociatedTokenAddressSync(
      tokenMint,
      user.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );

    const creatorTokenAccountInfo = await connection.getAccountInfo(creatorTokenAccount);
    if (creatorTokenAccountInfo === null) {
      console.log("creatorTokenAccount does not exist, creating...");
      const createUserTokenAccountIx = createAssociatedTokenAccountInstruction(
        user.publicKey,
        creatorTokenAccount,
        user.publicKey,
        tokenMint,
        TOKEN_2022_PROGRAM_ID,
      );
      const createAtaTx = new Transaction().add(createUserTokenAccountIx);
      await sendAndConfirmTransaction(connection, createAtaTx, [user], { skipPreflight: true });
    }

    let poolAccount = await connection.getAccountInfo(poolPda);
    if (poolAccount === null) {
      console.log("Pool PDA does not exist, initializing...");
      await program.methods
        .createPool()
        .accounts({
          curveConfiguration: curveConfigurationPda,
          pool: poolPda,
          tokenMint: tokenMint,
          poolTokenAccount: poolTokenAccountPda,
          poolSolVault: poolSolVaultPda,
          poolSolFeeVault: poolSolFeeVaultPda,
          userTokenAccount: creatorTokenAccount,
          payer: user.publicKey,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([user])
        .rpc();
    }

    const poolInfo = await program.account.pool.fetch(poolPda);
    console.log("Pool info", poolInfo);

    const poolSolVaultInfo = await connection.getAccountInfo(poolSolVaultPda);
    expect(poolSolVaultInfo).to.not.be.null;

    const poolSolFeeVaultInfo = await connection.getAccountInfo(poolSolFeeVaultPda);
    expect(poolSolFeeVaultInfo).to.not.be.null;
  } catch (e) {
    console.error("Error creating pool", e);
    assert.fail(`Error creating pool: ${e}`);
  }
}

async function airdrop(connection: Connection, user: PublicKey, amount: number) {
  console.log("===========Airdrop===========");
  try {
    const tx = await connection.requestAirdrop(user, amount * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(tx);
  } catch (e) {
    console.error("Error airdropping SOL", e);
    assert.fail(`Error airdropping SOL: ${e}`);
  }
}

async function initializeContract(curveConfigurationPda: PublicKey, admin: PublicKey) {
  console.log("===========Initialize Contract===========");
  try {
    let curveConfiguration = await provider.connection.getAccountInfo(curveConfigurationPda);
    if (curveConfiguration === null) {
      console.log("Program PDA does not exist, initializing...");
      const creatorPremint = new BN(1_000_000_000); // 1 token
      const creatorFees = new BN(5); // 5%
      const protocolFees = new BN(2); // 2%
      const tx = await program.methods
        .initialize(protocolFees, creatorPremint, creatorFees)
        .accounts({
          curveConfigurationAccount: curveConfigurationPda,
          admin: admin,
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
        } as any)
        .rpc();
      console.log("Initialization transaction signature", tx);
    }
  } catch (e) {
    console.error("Error initializing contract", e);
    assert.fail(`Error initializing contract: ${e}`);
  }
}

function getPda(seed: string, tokenMintPda: PublicKey, programId: PublicKey) {
  console.log("===========Get PDA===========");
  const [pool, poolBump] = PublicKey.findProgramAddressSync(
    [Buffer.from(POOL_SEED_PREFIX), tokenMintPda.toBuffer()],
    programId,
  );

  const [poolTokenAccount] = PublicKey.findProgramAddressSync(
    [pool.toBuffer(), TOKEN_2022_PROGRAM_ID.toBuffer(), tokenMintPda.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  const [poolSolVault, poolSolVaultBump] = PublicKey.findProgramAddressSync(
    [Buffer.from(SOL_VAULT_PREFIX), tokenMintPda.toBuffer()],
    programId,
  );

  const [poolSolFeeVault, poolSolFeeVaultBump] = PublicKey.findProgramAddressSync(
    [Buffer.from(SOL_FEE_PREFIX), tokenMintPda.toBuffer()],
    programId,
  );

  return {
    pool,
    poolTokenAccount,
    poolSolVault,
    poolSolFeeVault,
  };
}

async function createTokenMint(connection: Connection, user: Keypair) {
  try {
    console.log("===========Create Token Mint===========");
    // Generate new keypair for Mint Account
    const mintKeypair = Keypair.generate();

    // Address of Mint Account
    const mint = mintKeypair.publicKey;

    console.log("mint address", mint);

    // Metadata to store in Mint Account
    const metaData: TokenMetadata = {
      updateAuthority: user.publicKey,
      mint: mint,
      name: "Stoa" + Math.random().toString(36).substring(2, 15),
      symbol: "CATH" + Math.random().toString(36).substring(2, 15),
      uri: "https://res.cloudinary.com/drk2xlevs/image/upload/v1728099151/c8byp5uwcquh7qeisyvl.jpg",
      additionalMetadata: [
        ["description", "Only Possible On Solana"],
        ["github_name", "creator"],
        ["repository", "https://github.com/lukema95/stoa-core"],
        ["github_user", "admin"],
      ],
    };

    // Size of MetadataExtension 2 bytes for type, 2 bytes for length
    const metadataExtension = TYPE_SIZE + LENGTH_SIZE;
    // Size of metadata
    const metadataLen = pack(metaData).length;

    // Size of Mint Account with extension
    const mintLen = getMintLen([ExtensionType.MetadataPointer]);

    // Minimum lamports required for Mint Account
    const lamports = await connection.getMinimumBalanceForRentExemption(
      mintLen + metadataExtension + metadataLen,
    );
    // console.log("lamports", lamports);

    // Instruction to invoke System Program to create new account
    const createAccountInstruction = SystemProgram.createAccount({
      fromPubkey: user.publicKey, // Account that will transfer lamports to created account
      newAccountPubkey: mint, // Address of the account to create
      space: mintLen, // Amount of bytes to allocate to the created account
      lamports, // Amount of lamports transferred to created account
      programId: TOKEN_2022_PROGRAM_ID, // Program assigned as owner of created account
    });

    // Instruction to initialize the MetadataPointer Extension
    const initializeMetadataPointerInstruction = createInitializeMetadataPointerInstruction(
      mint, // Mint Account address
      user.publicKey, // Authority that can set the metadata address
      mint, // Account address that holds the metadata
      TOKEN_2022_PROGRAM_ID,
    );

    // Instruction to initialize Mint Account data
    const initializeMintInstruction = createInitializeMintInstruction(
      mint, // Mint Account Address
      tokenDecimals, // Decimals of Mint
      user.publicKey, // Designated Mint Authority
      null, // Optional Freeze Authority
      TOKEN_2022_PROGRAM_ID, // Token Extension Program ID
    );

    // Instruction to initialize Metadata Account data
    const initializeMetadataInstruction = createInitializeInstruction({
      programId: TOKEN_2022_PROGRAM_ID, // Token Extension Program as Metadata Program
      metadata: mint, // Account address that holds the metadata
      updateAuthority: user.publicKey, // Authority that can update the metadata
      mint: mint, // Mint Account address
      mintAuthority: user.publicKey, // Designated Mint Authority
      name: metaData.name,
      symbol: metaData.symbol,
      uri: metaData.uri,
    });

    // Instruction to update metadata, adding custom field
    const updateFieldInstruction = createUpdateFieldInstruction({
      programId: TOKEN_2022_PROGRAM_ID, // Token Extension Program as Metadata Program
      metadata: mint, // Account address that holds the metadata
      updateAuthority: user.publicKey, // Authority that can update the metadata
      field: metaData.additionalMetadata[0][0], // key
      value: metaData.additionalMetadata[0][1], // value
    });

    const updateFieldInstruction2 = createUpdateFieldInstruction({
      programId: TOKEN_2022_PROGRAM_ID, // Token Extension Program as Metadata Program
      metadata: mint, // Account address that holds the metadata
      updateAuthority: user.publicKey, // Authority that can update the metadata
      field: metaData.additionalMetadata[1][0], // key
      value: metaData.additionalMetadata[1][1], // value
    });

    const updateFieldInstruction3 = createUpdateFieldInstruction({
      programId: TOKEN_2022_PROGRAM_ID, // Token Extension Program as Metadata Program
      metadata: mint, // Account address that holds the metadata
      updateAuthority: user.publicKey, // Authority that can update the metadata
      field: metaData.additionalMetadata[2][0], // key
      value: metaData.additionalMetadata[2][1], // value
    });

    const updateFieldInstruction4 = createUpdateFieldInstruction({
      programId: TOKEN_2022_PROGRAM_ID, // Token Extension Program as Metadata Program
      metadata: mint, // Account address that holds the metadata
      updateAuthority: user.publicKey, // Authority that can update the metadata
      field: metaData.additionalMetadata[3][0], // key
      value: metaData.additionalMetadata[3][1], // value
    });

    // Add instructions to new transaction
    const transaction = new Transaction().add(
      createAccountInstruction,
      initializeMetadataPointerInstruction,
      initializeMintInstruction,
      initializeMetadataInstruction,
      updateFieldInstruction,
      updateFieldInstruction2,
      updateFieldInstruction3,
      updateFieldInstruction4,
    );

    // Send transaction
    const transactionSignature = await sendAndConfirmTransaction(
      connection,
      transaction,
      [user, mintKeypair], // Signers
    );

    console.log(
      "\nCreate Mint Account:",
      `https://solana.fm/tx/${transactionSignature}?cluster=devnet-solana`,
    );

    // Create Token Account
    const tokenAccount = await createAccount(
      connection,
      user, // payer
      mint, // mint address
      user.publicKey, // owner
      undefined, // keypair (optional)
      undefined, // options
      TOKEN_2022_PROGRAM_ID,
    );
    console.log("Token account created:", tokenAccount.toBase58());

    // Retrieve mint information
    const mintInfo = await getMint(connection, mint, "confirmed", TOKEN_2022_PROGRAM_ID);
    console.log("mintInfo", mintInfo);

    // Retrieve and log the metadata pointer state
    const metadataPointer = getMetadataPointerState(mintInfo);
    console.log("\nMetadata Pointer:", JSON.stringify(metadataPointer, null, 2));

    // Retrieve and log the metadata state
    const metadata = await getTokenMetadata(
      connection,
      mint, // Mint Account address
    );
    console.log("\nMetadata:", JSON.stringify(metadata, null, 2));

    return mint;
  } catch (e) {
    console.error("Error creating token mint", e);
    assert.fail(`Error creating token mint: ${e}`);
  }
}
