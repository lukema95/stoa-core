# 02 Accounts and PDAs

[English](02-accounts-pdas.md) | [简体中文](02-accounts-pdas.zh.md)

Stoa's account model: the configuration, the pool, and both vaults are PDAs; the token itself is a Token-2022 mint.

## 1. Program-owned accounts

### 1.1 `CurveConfiguration` (global singleton)

```rust
pub struct CurveConfiguration {
    pub admin: Pubkey,          // protocol fee recipient and initialize signer
    pub protocol_fees: u64,     // integer percentage, 0-100
    pub creator_premint: u64,   // premint amount, curve origin
    pub creator_fees: u64,      // integer percentage, 0-100
}
```

- Seeds: `["CurveConfiguration"]`
- Space: `8 + 32 + 8 + 8 + 8 = 64`
- `init` can succeed only once. There is no `update_config` instruction; changing fees requires a program upgrade or a new deployment.

`admin` has two roles: the deployer/protocol account and the address that `protocol_account` must equal in `buy` and `sell` (checked in the instruction).

### 1.2 `Pool` (one per mint)

```rust
pub struct Pool {
    pub creator: Pubkey,
    pub token: Pubkey,
    pub total_supply: u64,      // program-tracked supply: +amount on buy, -amount on sell
    pub reserve_sol: u64,       // program-tracked curve reserve in lamports
    pub bump: u8,
    pub fee_recipients: Vec<FeeRecipient>,
    pub total_fee_share_percentage: u16,
}
```

- Seeds: `["liquidity_pool", token_mint]`
- `MAX_FEE_RECIPIENTS = 10`

`FeeRecipient`:

```rust
pub enum FeeRecipientType { User, Pool }

pub struct FeeRecipient {
    pub recipient_type: FeeRecipientType,
    pub address: Pubkey,
    pub share_percentage: u8,
    pub total_fee: u64,         // cumulative fees credited
    pub unclaimed_fee: u64,     // fees not yet claimed
}
```

A `User` recipient claims SOL to `address`. A `Pool` recipient treats `address` as **another Pool PDA** and uses its unclaimed SOL to buy shares on that pool's curve.

## 2. PDA map

```mermaid
flowchart TB
  subgraph Global
    CFG["CurveConfiguration<br/>seeds: \"CurveConfiguration\""]
  end

  subgraph PerMint["Per Token Mint"]
    POOL["Pool<br/>seeds: \"liquidity_pool\" + mint"]
    SV["pool_sol_vault<br/>seeds: \"liquidity_sol_vault\" + mint<br/>owner = Stoa program"]
    FV["pool_sol_fee_vault<br/>seeds: \"liquidity_sol_fee_vault\" + mint<br/>owner = Stoa program"]
    PATA["Pool ATA<br/>authority = Pool<br/>Associated Token Program"]
  end

  MINT["Token-2022 Mint<br/>mint authority -> Pool"]

  CFG --- POOL
  MINT --> POOL
  MINT --> SV
  MINT --> FV
  POOL --> PATA
```

Derivation in TypeScript (the same logic used by clients and tests):

```ts
const [pool] = PublicKey.findProgramAddressSync(
  [Buffer.from("liquidity_pool"), mint.toBuffer()],
  programId,
);
const [solVault] = PublicKey.findProgramAddressSync(
  [Buffer.from("liquidity_sol_vault"), mint.toBuffer()],
  programId,
);
const [feeVault] = PublicKey.findProgramAddressSync(
  [Buffer.from("liquidity_sol_fee_vault"), mint.toBuffer()],
  programId,
);
```

**Why are the vaults PDAs?** On sell, the program debits lamports from the vault. Only an account whose owner is the program can be debited directly with `sub_lamports` / `add_lamports`. User wallets are owned by the System Program, so transfers from them must go through `system_program::transfer` CPI.

## 3. Two ways to move SOL

| Scenario | Method | Reason |
|---|---|---|
| User -> vault / protocol / creator | `system_program::transfer` CPI | `from` is a signer and system-owned |
| Vault -> user / protocol / creator / another vault | `from.sub_lamports` + `to.add_lamports` | `from` is a program-owned PDA; no System Program CPI needed |

Implementation: `transfer_sol_to_pool` / `transfer_sol_to_user` use CPI; `transfer_sol_from_pool` mutates lamports directly.

`buy_with_vault` (cross-pool claim) uses the direct path: the source is the source pool's fee vault and the destination is the target pool's SOL vault; both are program PDAs.

## 4. How mint authority moves to the pool

`create_pool` sequence:

1. `init` the Pool PDA (the address is now fixed; its bump is stored in `pool.bump`).
2. `set_authority(MintTokens, Some(pool.key()))`: the payer gives up mint authority.
3. `mint_to` the premint using CPI signer seeds:

```text
["liquidity_pool", token_mint, &[pool.bump]]
```

Every `buy` uses the same seeds to mint to the buyer. On `sell`, the user signs `burn` directly.

This is the standard "PDA as mint authority" pattern. Mint authority is not left with the creator because the creator could mint outside the curve and break the relationship between `total_supply` and the real Token-2022 supply.

## 5. Token-2022 instead of SPL Token

Pool creation and trading accounts all declare:

```rust
pub token_program: Program<'info, Token2022>
```

Clients typically initialize the following in the same transaction as `create_pool`:

- Metadata Pointer extension
- Token Metadata (name / symbol / uri)
- Custom fields such as `repository_name`, `repo_url`, and `github_user`

Product reason: metadata lives on the mint, so there is no parallel Metaplex Token Metadata account. Technical reason: `anchor_spl::token_interface` can abstract Token and Token-2022, and Stoa deliberately uses only Token-2022.

## 6. Reading account constraints

Taking `BuyTokens` as an example:

```text
curve_configuration_account  seeds=["CurveConfiguration"]      fee config
pool                         seeds=["liquidity_pool", mint]
                             bump = pool.bump                  prevents fake pools
token_mint                   mut                               minted on buy
pool_token_account           ATA(mint, authority=pool)         pool token account
pool_sol_vault               seeds=["liquidity_sol_vault", mint]
pool_sol_fee_vault           seeds=["liquidity_sol_fee_vault", mint]
user_token_account           ATA(mint, authority=user)
protocol_account             no constraint; checked in handler == admin
pool_creator                 address = pool.creator            prevents fee redirection
user                         Signer + mut                      pays
```

`has_one`, `constraint`, and `seeds` are Anchor's three validation layers. Stoa uses `seeds` for PDAs, `address` for `pool_creator` / `distribution_pool_creator`, and an explicit check for the protocol account.

## 7. Space and `Box<Account>`

The Solana stack is roughly 4 KB. Expanding multiple `Account<T>` values at once can overflow it, so instructions generally use:

```rust
pub pool: Box<Account<'info, Pool>>,
```

`Box` moves account data to the heap. This is an engineering necessity under the BPF stack limit, not business logic.

`FeeRecipient` serializes to `1 (enum tag) + 32 (address) + 1 (share) + 8 (total_fee) + 8 (unclaimed_fee) = 50` bytes. `Pool::ACCOUNT_SIZE` accounts for 50 bytes times 10 recipients plus the `u16` total share, and a unit test checks it against the real Borsh length. Update the constant when adding fields.

## 8. Events (for indexers, not on-chain logic)

| Event | When |
|---|---|
| `PoolCreated` | Pool creation completed |
| `BuyTokensEvent` | Successful buy |
| `SellTokensEvent` | Successful sell |
| `DistributeFeeEvent` | Fee share added or updated |
| `CleanZeroFeeRecipientEvent` | Zero-share / zero-unclaimed recipient removed |
| `ClaimFeesEvent` | User or pool fee claim |

Indexers replay these events into `pools`, `tokens`, and fee-recipient tables. The program itself never reads its own events.
