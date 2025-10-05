# 04 Core Flows

[English](04-core-flows.md) | [简体中文](04-core-flows.zh.md)

This chapter follows the actual call order. The "client" in the sequence diagrams includes the Token-2022 instructions that an integrator builds.

## 0. Lifecycle

```mermaid
stateDiagram-v2
  [*] --> Uninitialized
  Uninitialized --> Configured: initialize(admin, fees, premint)
  Configured --> PoolLive: create_pool(mint)
  PoolLive --> PoolLive: buy / sell
  PoolLive --> PoolLive: distribute_user_fee / distribute_pool_fee
  PoolLive --> PoolLive: claim_user_fees / claim_pool_fees
```

A cluster needs `initialize` once. After that, each mint gets one `create_pool`.

---

## 1. `initialize`

**Caller:** the protocol admin (the payer becomes the future `admin` and protocol fee recipient).

```mermaid
sequenceDiagram
  participant Admin
  participant Program
  participant CFG as CurveConfiguration PDA

  Admin->>Program: initialize(protocol_fees, creator_premint, creator_fees)
  Program->>Program: validate each fee <= 100 and their sum <= 100
  Program->>CFG: init + write admin/fees/premint
```

Test defaults: `protocol_fees=2`, `creator_fees=5`, `creator_premint=1e9`.

There is no update instruction. Fees are protocol-level constants.

Because the first caller becomes the admin, deploy and initialize as part of one controlled deployment job. See [08-deployment.md](./08-deployment.md).

---

## 2. `create_pool`

This is the on-chain step that turns a Token-2022 mint into a Stoa pool. The program does not inspect off-chain identity; it only checks that the mint exists and that the payer currently controls mint authority.

```mermaid
sequenceDiagram
  participant Creator
  participant Token2022
  participant Program
  participant Pool
  participant Vault as SOL Vault
  participant FeeVault as Fee Vault

  Note over Creator,Token2022: Earlier in the same transaction (client-built)
  Creator->>Token2022: create mint + Metadata Pointer + metadata fields
  Creator->>Token2022: create creator ATA

  Creator->>Program: create_pool()
  Program->>Vault: create_account if missing (owner=program)
  Program->>FeeVault: create_account if missing
  Program->>Token2022: set_authority(MintTokens -> Pool)
  Program->>Token2022: mint_to(creator ATA, premint) signed by Pool PDA
  Program->>Pool: creator / token / bump / total_supply=premint
  Program-->>Creator: emit PoolCreated
```

Details:

- The SOL vault and fee vault are created with `system_program::create_account` and signed with their own seeds. They have zero data and are funded rent-exempt. Their owner is the **Stoa program**, which is what allows the program to debit them directly on sell.
- Mint authority is transferred after the Pool PDA is initialized, because the new authority is the Pool address.
- `pool_token_account` is the Pool's ATA. The current buy/sell paths do not move tokens into it (buys mint directly to the user), but it is created for future integrations.

Integrators usually build "create mint + write metadata + create ATA + create_pool" into **one transaction**, so a failure cannot leave a mint without a pool.

---

## 3. `buy(amount_tokens)`

The user specifies **how many shares to buy**, not how much SOL to spend. The program integrates the curve at the current `total_supply`.

```mermaid
sequenceDiagram
  participant User
  participant Program
  participant Pool
  participant Vault as SOL Vault
  participant Token2022
  participant Protocol
  participant Creator

  User->>Program: buy(amount)
  Program->>Program: protocol_account == config.admin ?
  Program->>Pool: price = get_buy_price(supply, amount)
  alt price == 0 or user lamports < price
    Program-->>User: InvalidAmount / InsufficientBalance
  end
  User->>Vault: transfer price (curve price)
  User->>Protocol: transfer protocol_fees
  User->>Creator: transfer undistributed creator_fees
  User->>FeeVault: transfer distributed creator_fees
  Program->>Token2022: mint_to(user ATA, amount) signed by Pool PDA
  Program->>Pool: supply += amount, reserve_sol += price
  Program-->>User: emit BuyTokensEvent
```

Funding order: the full curve price enters the reserve first; fees are charged **on top** from the buyer's wallet. `reserve_sol += price` includes neither fee.

The event `price` is `get_buy_price_after_fees` computed **before** `total_supply += amount`, so it represents the fee-inclusive price of this trade. Indexers can use it directly; for exact lamports, cross-check the transfer instructions.

**There is no on-chain slippage limit.** The user signs only `amount_tokens`. If a searcher buys first, supply moves up and the user pays more SOL for the same number of shares. Production integrations should enforce a client-side `max_sol_in` check or wait for a future on-chain parameter.

---

## 4. `sell(amount_tokens)`

```mermaid
sequenceDiagram
  participant User
  participant Program
  participant Token2022
  participant Pool
  participant Vault as SOL Vault
  participant Protocol
  participant Creator

  User->>Program: sell(amount)
  Program->>Program: protocol_account == admin ?
  Program->>Program: supply - premint >= amount ?
  User->>Token2022: burn(amount) signed by user
  Program->>Pool: price = get_sell_price(supply, amount)
  alt price > reserve_sol
    Program-->>User: PoolInsufficientFunds
  end
  Vault->>User: price - protocol_fee - creator_fee
  Vault->>Protocol: protocol_fee
  Vault->>Creator: undistributed creator_fee
  Vault->>FeeVault: distributed creator_fee
  Program->>Pool: reserve_sol -= price, supply -= amount
  Program-->>User: emit SellTokensEvent
```

Three asymmetries compared with buy:

1. **Burn happens before pricing.** If a later step fails, the whole transaction reverts, including the burn.
2. Fees come **out of the reserve**; the user receives the net amount.
3. Floor: `total_supply` cannot be sold below `premint`.

There is no `min_sol_out`.

---

## 5. Invariants after trades

Ignoring the fee vault (fees are not part of the curve):

```text
pool.reserve_sol  ==  sum(buy curve prices) - sum(sell curve prices)
pool.total_supply ==  premint + net shares bought
SOL Vault lamports ~= rent-exempt + reserve_sol
```

On buy, fees do **not** enter the vault:

```text
vault increase = price
user decrease  = price + all fees
```

On sell:

```text
vault decrease = price
user increase  = price - all fees
fee recipients increase = all fees
```

The vault decreases by the full pre-fee `price`, and the fee portion leaves the vault to the recipients, so the accounting remains consistent.

---

## 6. Fee configuration and claims

Fee sharing does not change the curve; it only determines how the **creator fee** is split. The protocol fee always goes 100% to `admin`.

```mermaid
flowchart TB
  subgraph Config["creator-only writes"]
    DU[distribute_user_fee]
    DP[distribute_pool_fee]
  end

  subgraph Accrue["automatic on every buy/sell"]
    Acc[unclaimed_fee += creator_fee * share%]
    Vault[SOL transferred into Fee Vault]
  end

  subgraph Claim["claims"]
    CU[claim_user_fees: Fee Vault -> user wallet]
    CP[claim_pool_fees: Fee Vault -> buy target pool shares]
  end

  DU --> Accrue
  DP --> Accrue
  Acc --> CU
  Acc --> CP
```

Full rules are in [05-fee-system.md](./05-fee-system.md).

---

## 7. Instruction and account summary

| Instruction | Required signer | Key PDAs | External programs |
|---|---|---|---|
| `initialize` | admin | `CurveConfiguration` | System |
| `create_pool` | payer | `Pool`, both vaults | System, Token-2022, ATA |
| `buy` | user | pool, vaults, config | System, Token-2022, ATA |
| `sell` | user | pool, vaults | System, Token-2022, ATA |
| `distribute_user_fee` | `pool.creator` | `Pool` | System |
| `distribute_pool_fee` | `pool.creator` | source pool + target pool | System |
| `claim_user_fees` | fee recipient | `Pool` + fee vault | System |
| `claim_pool_fees` | target pool creator | both pools and vaults | System, Token-2022 |

Compute budget: when `create_pool` shares a transaction with Token-2022 metadata initialization, clients usually add a `ComputeBudgetProgram` instruction. Token-2022 plus metadata initialization is relatively expensive.
