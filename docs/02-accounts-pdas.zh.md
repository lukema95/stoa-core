# 02 账户模型与 PDA

[English](02-accounts-pdas.md) | [简体中文](02-accounts-pdas.zh.md)

Stoa 的账户模型：配置、池子、两个金库都是 PDA，代币走 Token-2022。

## 1. 两类程序拥有的账户

### 1.1 `CurveConfiguration`（全局单例）

```rust
pub struct CurveConfiguration {
    pub admin: Pubkey,          // 协议费收款地址，也是 initialize 的签名者
    pub protocol_fees: u64,     // 百分比整数，0–100
    pub creator_premint: u64,   // 预铸量，曲线原点
    pub creator_fees: u64,      // 百分比整数，0–100
}
```

- 种子：`["CurveConfiguration"]`
- 空间：`8 + 32 + 8 + 8 + 8 = 64`
- `init` 只能成功一次。没有 `update_config` 指令，改费率必须升级程序或另外部署。

`admin` 身兼两职：部署者，以及买卖时 `protocol_account` 必须相等的地址（`buy.rs` / `sell.rs` 里手写校验）。

### 1.2 `Pool`（每个 mint 一个）

```rust
pub struct Pool {
    pub creator: Pubkey,
    pub token: Pubkey,
    pub total_supply: u64,      // 程序自己记的供给，买 +amount、卖 -amount
    pub reserve_sol: u64,       // 程序自己记的曲线 SOL 储备
    pub bump: u8,
    pub fee_recipients: Vec<FeeRecipient>,
    pub total_fee_share_percentage: u16,
}
```

- 种子：`["liquidity_pool", token_mint]`
- `MAX_FEE_RECIPIENTS = 10`

`FeeRecipient`：

```rust
pub enum FeeRecipientType { User, Pool }

pub struct FeeRecipient {
    pub recipient_type: FeeRecipientType,
    pub address: Pubkey,
    pub share_percentage: u8,
    pub total_fee: u64,         // 累计入账
    pub unclaimed_fee: u64,     // 尚未领取
}
```

`User` 领取时把 SOL 转给 `address`；`Pool` 领取时把 `address` 当成**另一个 Pool PDA**，用待领 SOL 去那条曲线上买。

## 2. PDA 地图

```mermaid
flowchart TB
  subgraph Global
    CFG["CurveConfiguration<br/>seeds: \"CurveConfiguration\""]
  end

  subgraph PerMint["每个 Token Mint"]
    POOL["Pool<br/>seeds: \"liquidity_pool\" + mint"]
    SV["pool_sol_vault<br/>seeds: \"liquidity_sol_vault\" + mint<br/>owner = stoa program"]
    FV["pool_sol_fee_vault<br/>seeds: \"liquidity_sol_fee_vault\" + mint<br/>owner = stoa program"]
    PATA["Pool ATA<br/>authority = Pool<br/>Associated Token Program"]
  end

  MINT["Token-2022 Mint<br/>mint authority → Pool"]

  CFG --- POOL
  MINT --> POOL
  MINT --> SV
  MINT --> FV
  POOL --> PATA
```

推导代码（前端 `utils.ts` 与测试文件一致）：

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

**为什么金库也要 PDA？** 程序需要在卖出时从金库扣 lamports。只有 **owner = 本程序** 的账户，才能在程序里直接 `sub_lamports` / `add_lamports`。用户钱包 owner 是 System Program，必须走 `system_program::transfer` CPI。

## 3. 两种转 SOL 的方式

| 场景 | 方法 | 原因 |
|------|------|------|
| 用户 → 金库 / 协议 / 创建者 | `system_program::transfer` CPI | from 是 signer，system-owned |
| 金库 → 用户 / 协议 / 创建者 / 另一个金库 | `from.sub_lamports` + `to.add_lamports` | from 是程序拥有的 PDA，无需 System Program |

对应实现：`transfer_sol_to_pool` / `transfer_sol_to_user` vs `transfer_sol_from_pool`。

`buy_with_vault`（跨池领取）走后者：源是本池 fee vault，目标是对方 SOL vault，两边都是本程序 PDA。

## 4. 铸币权限如何交给 Pool

`create_pool` 顺序：

1. `init` Pool PDA（此时地址已确定，bump 写入 `pool.bump`）。
2. `set_authority(MintTokens, Some(pool.key()))`：payer 放弃 mint 权。
3. `mint_to` 预铸，**CPI signer seeds**：

```text
["liquidity_pool", token_mint, &[pool.bump]]
```

之后每次 `buy` 同样用这组 seeds 给用户铸币。`sell` 则由用户自己签名 `burn`。

这是标准的「PDA 作为 mint authority」模式。不把 mint authority 留给创建者，是因为创建者可以在曲线外无限增发，摧毁 `total_supply` 与真实 mint supply 的对应关系。

## 5. Token-2022 而不是 SPL Token

建池与买卖账户都声明：

```rust
pub token_program: Program<'info, Token2022>
```

前端同一笔交易里还会初始化：

- Metadata Pointer extension
- Token Metadata（name / symbol / uri）
- 自定义字段：`repository_name`、`repo_url`、`github_user`

选择 Token-2022 的产品原因：元数据可以直接挂在 mint 上，不必再上 Metaplex Token Metadata 那套并行账户。技术原因：`anchor_spl::token_interface` 可以同时抽象 Token 与 Token-2022，本项目锁死 2022。

## 6. 账户约束怎么读

以 `BuyTokens` 为例：

```text
curve_configuration_account  seeds=["CurveConfiguration"]     读费率
pool                         seeds=["liquidity_pool", mint]
                             bump = pool.bump                 防伪造 Pool
token_mint                   mut                              要 mint
pool_token_account           ATA(mint, authority=pool)        池子代币账户
pool_sol_vault               seeds=["liquidity_sol_vault", mint]
pool_sol_fee_vault           seeds=["liquidity_sol_fee_vault", mint]
user_token_account           ATA(mint, authority=user)
protocol_account             无约束，指令里手写 == admin
pool_creator                 address = pool.creator           防费用重定向
user                         Signer + mut                     付钱
```

`has_one`、`constraint`、`seeds` 是 Anchor 的三层防护。本项目对 PDA 用 seeds，对 `pool_creator` / `distribution_pool_creator` 用 `address` 约束，对协议地址在指令里手写校验。

## 7. 空间与 `Box<Account>`

Solana 栈约 4KB。多个 `Account<T>` 同时展开容易爆栈，所以指令里普遍：

```rust
pub pool: Box<Account<'info, Pool>>,
```

`Box` 把账户数据放到堆上。这不是业务逻辑，是 BPF 限制下的工程习惯。

`FeeRecipient` 的 Borsh 长度是 `1(enum tag) + 32(address) + 1(share) + 8(total_fee) + 8(unclaimed_fee) = 50` 字节。`Pool::ACCOUNT_SIZE` 已按 50 字节 × 最多 10 个接收者 + `u16` 总份额计算，并加了单元测试对账真实序列化长度；新增字段时应同步更新。

## 8. 事件（给索引器，不是给链上逻辑）

| 事件 | 何时 |
|------|------|
| `PoolCreated` | 建池结束 |
| `BuyTokensEvent` | 买入成功 |
| `SellTokensEvent` | 卖出成功 |
| `DistributeFeeEvent` | 新增/更新分润 |
| `CleanZeroFeeRecipientEvent` | 清掉 0 份额且 0 待领的接收者 |
| `ClaimFeesEvent` | 用户或池子领取 |

Backend 用事件回放填充 `pools` / `tokens` / 分润表。合约本身不读这些事件。
