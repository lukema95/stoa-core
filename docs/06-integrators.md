# 06 Integrator Guide

[English](06-integrators.md) | [简体中文](06-integrators.zh.md)

This guide is for clients, indexers, and front-ends that build Stoa transactions or read Stoa state. It documents the canonical program ID, PDA derivation, instruction accounts, events, and error codes.

## 1. Program artifacts

After `anchor build`:

```text
target/idl/stoa.json       Anchor IDL
target/types/stoa.ts       TypeScript types
target/deploy/stoa.so      SBF program
```

Set up the client:

```ts
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Stoa } from "./target/types/stoa";
import IDL from "./target/idl/stoa.json";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = new Program<Stoa>(IDL as Stoa, provider);
```

The canonical program ID is:

```text
7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM
```

Keep the ID in `Anchor.toml`, `declare_id!`, and the IDL in sync. Run `./scripts/check-program-id.sh` after changing any of them.

## 2. PDA derivation

```ts
import { PublicKey } from "@solana/web3.js";

const [curveConfiguration] = PublicKey.findProgramAddressSync(
  [Buffer.from("CurveConfiguration")],
  program.programId,
);

const [pool] = PublicKey.findProgramAddressSync(
  [Buffer.from("liquidity_pool"), mint.toBuffer()],
  program.programId,
);

const [poolSolVault] = PublicKey.findProgramAddressSync(
  [Buffer.from("liquidity_sol_vault"), mint.toBuffer()],
  program.programId,
);

const [poolSolFeeVault] = PublicKey.findProgramAddressSync(
  [Buffer.from("liquidity_sol_fee_vault"), mint.toBuffer()],
  program.programId,
);

const poolTokenAccount = getAssociatedTokenAddressSync(
  mint,
  pool,
  true, // allowOwnerOffCurve: the pool is a PDA
  TOKEN_2022_PROGRAM_ID,
);
```

`curveConfiguration` is global: one per program deployment. `pool`, both vaults, and the pool ATA are per mint.

## 3. Account names and conventions

- IDL/Rust names use snake_case; generated TypeScript types use camelCase.
- Every `mut` account must be writable.
- `user` is the signer for buys, sells, and claims.
- `payer` is the signer for `create_pool`.
- `protocol_account` must equal `CurveConfiguration.admin`.
- `pool_creator` must equal `pool.creator`; `distribution_pool_creator` must equal `distribution_pool.creator`.
- Token amounts are **base units** (with the mint's decimals), not whole tokens. For a 9-decimal mint, 1 token = `1_000_000_000`.

## 4. Instruction account tables

### `initialize`

| Account | Mut | Signer | Notes |
|---|---|---|---|
| `curve_configuration_account` | ✅ | | PDA `["CurveConfiguration"]` |
| `admin` | ✅ | ✅ | Becomes `admin` and protocol fee recipient |
| `rent` | | | `SYSVAR_RENT_PUBKEY` |
| `system_program` | | | `SystemProgram.programId` |

Arguments: `protocol_fees: u64`, `creator_premint: u64`, `creator_fees: u64`. Each fee must be <= 100 and their sum must be <= 100. `initialize` succeeds only once.

### `create_pool`

| Account | Mut | Signer | Notes |
|---|---|---|---|
| `curve_configuration` | ✅ | | Global config |
| `pool` | ✅ | | PDA `["liquidity_pool", mint]` |
| `token_mint` | ✅ | | Token-2022 mint |
| `pool_token_account` | ✅ | | ATA(mint, authority = pool) |
| `pool_sol_vault` | ✅ | | PDA `["liquidity_sol_vault", mint]` |
| `pool_sol_fee_vault` | ✅ | | PDA `["liquidity_sol_fee_vault", mint]` |
| `user_token_account` | ✅ | | Creator ATA, receives premint |
| `payer` | ✅ | ✅ | Creates accounts and becomes `pool.creator` |
| `token_program` | | | Token-2022 program |
| `associated_token_program` | | | Associated Token program |
| `rent` | | | Rent sysvar |
| `system_program` | | | System program |

The payer must currently be the mint authority. `create_pool` transfers mint authority to the pool PDA and mints `creator_premint` to `user_token_account`.

### `buy`

| Account | Mut | Signer | Notes |
|---|---|---|---|
| `curve_configuration_account` | ✅ | | Global config |
| `pool` | ✅ | | Must match the mint and stored bump |
| `token_mint` | ✅ | | Token-2022 mint |
| `pool_token_account` | ✅ | | ATA(mint, authority = pool) |
| `pool_sol_vault` | ✅ | | Curve reserve |
| `pool_sol_fee_vault` | ✅ | | Fee vault |
| `user_token_account` | ✅ | | Buyer ATA, receives shares |
| `protocol_account` | ✅ | | Must equal `config.admin` |
| `pool_creator` | ✅ | | Must equal `pool.creator` |
| `user` | ✅ | ✅ | Pays SOL |
| `rent` | | | Rent sysvar |
| `system_program` | | | System program |
| `token_program` | | | Token-2022 program |
| `associated_token_program` | | | Associated Token program |

Argument: `amount_tokens: u64` (base units). Fees are paid on top of the curve price; quote with the on-chain helper logic or reproduce `get_buy_price_after_fees` off-chain.

### `sell`

Same accounts as `buy`, except `user` is the seller and `user_token_account` must hold the shares. Argument: `amount_tokens: u64`. Fees are deducted from the curve proceeds.

### `distribute_user_fee`

| Account | Mut | Signer | Notes |
|---|---|---|---|
| `pool` | ✅ | | Must be owned by `user` (creator) |
| `token_mint` | ✅ | | Used for the pool PDA |
| `recipient` | ✅ | | User wallet that can later claim |
| `user` | ✅ | ✅ | Must equal `pool.creator` |
| `rent` | | | Rent sysvar |
| `system_program` | | | System program |

Argument: `fee_share_percentage: u8`.

### `distribute_pool_fee`

| Account | Mut | Signer | Notes |
|---|---|---|---|
| `pool` | ✅ | | Source pool |
| `token_mint` | ✅ | | Source mint |
| `distribute_fee_pool` | ✅ | | Target Pool PDA |
| `distribute_fee_pool_token_mint` | ✅ | | Target mint |
| `user` | ✅ | ✅ | Must equal `pool.creator` |
| `rent` | | | Rent sysvar |
| `system_program` | | | System program |

Argument: `fee_share_percentage: u8`. The target pool must not be the source pool.

### `claim_user_fees`

| Account | Mut | Signer | Notes |
|---|---|---|---|
| `pool` | ✅ | | Pool holding the fee ledger |
| `token_mint` | ✅ | | Pool mint |
| `pool_sol_fee_vault` | ✅ | | Source of the SOL |
| `fee_recipient` | ✅ | | Must equal `user` |
| `user` | ✅ | ✅ | Claims to this wallet |
| `rent` | | | Rent sysvar |
| `system_program` | | | System program |
| `token_program` | | | Token-2022 program |
| `associated_token_program` | | | Associated Token program |

### `claim_pool_fees`

| Account | Mut | Signer | Notes |
|---|---|---|---|
| `pool` | ✅ | | Source pool (pays the fee) |
| `pool_sol_fee_vault` | ✅ | | Source fee vault |
| `token_mint` | ✅ | | Source mint |
| `curve_configuration_account` | ✅ | | Global config |
| `distribution_pool` | ✅ | | Target pool (receives the buy) |
| `distribution_pool_token_mint` | ✅ | | Target mint |
| `distribution_pool_token_account` | ✅ | | Target pool ATA |
| `distribution_pool_sol_vault` | ✅ | | Target SOL vault |
| `distribution_pool_sol_fee_vault` | ✅ | | Target fee vault |
| `distribution_pool_user_token_account` | ✅ | | Claimant ATA for target shares |
| `protocol_account` | ✅ | | Must equal `config.admin` |
| `distribution_pool_creator` | ✅ | | Must equal `distribution_pool.creator` |
| `user` | ✅ | ✅ | Must equal `distribution_pool.creator` |
| `rent` | | | Rent sysvar |
| `system_program` | | | System program |
| `token_program` | | | Token-2022 program |
| `associated_token_program` | | | Associated Token program |

The claim converts the source pool's unclaimed fee into target-pool shares and mints them to `distribution_pool_user_token_account`.

## 5. Events

Anchor event names become camelCase in TypeScript. Subscribe with `program.addEventListener("buyTokensEvent", ...)`.

| Event | Fields |
|---|---|
| `PoolCreated` | `creator`, `token_mint`, `pool`, `pool_sol_vault`, `pool_sol_fee_vault`, `pool_token_account` |
| `BuyTokensEvent` | `pool`, `user`, `user_token_account`, `amount_tokens`, `price` |
| `SellTokensEvent` | `pool`, `user`, `user_token_account`, `amount_tokens`, `price` |
| `DistributeFeeEvent` | `pool`, `fee_share_percentage`, `recipient_type`, `recipient` |
| `CleanZeroFeeRecipientEvent` | `pool`, `fee_recipient` |
| `ClaimFeesEvent` | `pool`, `fee_recipient_type`, `fee_recipient`, `sender`, `claim_amount` |

`price` in buy/sell events is the fee-inclusive price for that trade, calculated before the supply update. For exact lamport accounting, also inspect the transfer instructions.

## 6. Error codes

| Code | Error | Typical cause |
|---|---|---|
| 6000 | `InvalidFee` | Fee > 100, or `protocol_fees + creator_fees > 100` |
| 6001 | `InvalidAmount` | Zero amount, or a curve price of zero |
| 6002 | `InsufficientBalance` | Buyer does not have enough lamports for the curve price |
| 6003 | `InsufficientFunds` | Fee vault cannot cover the claim |
| 6004 | `InvalidOwner` | Caller is not the pool creator or fee recipient |
| 6005 | `InvalidProtocolAccount` | `protocol_account != config.admin` |
| 6006 | `SupplyNotEnough` | Sell would drop supply below the premint floor |
| 6007 | `PoolInsufficientFunds` | Reserve cannot cover the sell |
| 6008 | `InvalidRecipient` | Self-referencing pool fee distribution, or missing recipient |
| 6009 | `InvalidFeeSharePercentage` | Share > 100 or total shares > 100 |
| 6010 | `MaxFeeRecipientReached` | Adding an 11th recipient |

## 7. Token-2022 requirements

- Create the mint with the Token-2022 program.
- If you use the Metadata Pointer extension, initialize metadata in the same transaction as mint creation or before `create_pool`.
- Create the creator ATA before `create_pool`.
- The pool ATA is created by `create_pool`; the user ATA for buys/sells must exist (or be created by the client).
- Mints with transfer hooks or transfer fees may require extra accounts and are not covered by the current instruction set. Test the exact extension combination you plan to support.

## 8. Common pitfalls

1. **Wrong signer for `create_pool`.** The payer must be the current mint authority.
2. **Wrong `pool_creator`.** It must exactly equal `pool.creator`; otherwise the transaction fails the `address` constraint.
3. **Wrong `protocol_account`.** It must equal `config.admin`.
4. **Whole tokens instead of base units.** `amount_tokens` uses mint decimals.
5. **No slippage check.** Compute a client-side maximum SOL input; the program currently has no `max_sol_in`.
6. **Claiming with the wrong recipient.** `claim_user_fees` requires the recipient to sign; `claim_pool_fees` requires the target pool creator.
7. **Stale IDL.** Regenerate types after any instruction or account change.

## 9. Versioning

The IDL and TypeScript types are versioned with the program. Until a published `@stoa/idl` package exists, pin integrations to a release tag and regenerate artifacts from that tag. Account layout changes are breaking changes.
