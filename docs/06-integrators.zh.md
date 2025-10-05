# 06 集成指南

[English](06-integrators.md) | [简体中文](06-integrators.zh.md)

本指南面向构造 Stoa 交易或读取 Stoa 状态的客户端、索引器和前端。内容包括 canonical
program ID、PDA 推导、指令账户、事件和错误码。

## 1. 程序产物

`anchor build` 之后：

```text
target/idl/stoa.json       Anchor IDL
target/types/stoa.ts       TypeScript 类型
target/deploy/stoa.so      SBF 程序
```

客户端初始化：

```ts
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Stoa } from "./target/types/stoa";
import IDL from "./target/idl/stoa.json";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = new Program<Stoa>(IDL as Stoa, provider);
```

Canonical program ID：

```text
7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM
```

保持 `Anchor.toml`、`declare_id!` 和 IDL 中的 ID 一致。修改任意一处后运行
`./scripts/check-program-id.sh`。

## 2. PDA 推导

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
  true, // allowOwnerOffCurve: Pool 是 PDA
  TOKEN_2022_PROGRAM_ID,
);
```

`curveConfiguration` 是全局的：每个 program 部署只有一个。`pool`、两个金库和 Pool ATA
都是每个 mint 一套。

## 3. 账户名与约定

- IDL/Rust 使用 snake_case；生成的 TypeScript 类型使用 camelCase。
- 所有 `mut` 账户必须是 writable。
- `user` 是买入、卖出和领取的签名者。
- `payer` 是 `create_pool` 的签名者。
- `protocol_account` 必须等于 `CurveConfiguration.admin`。
- `pool_creator` 必须等于 `pool.creator`；`distribution_pool_creator` 必须等于
  `distribution_pool.creator`。
- 代币数量是 **最小单位**（按 mint decimals），不是整枚代币。9 位小数时，1 枚 =
  `1_000_000_000`。

## 4. 指令账户表

### `initialize`

| 账户 | Mut | Signer | 说明 |
|---|---|---|---|
| `curve_configuration_account` | ✅ | | PDA `["CurveConfiguration"]` |
| `admin` | ✅ | ✅ | 成为 `admin` 和协议费收款地址 |
| `rent` | | | `SYSVAR_RENT_PUBKEY` |
| `system_program` | | | `SystemProgram.programId` |

参数：`protocol_fees: u64`、`creator_premint: u64`、`creator_fees: u64`。每个费率必须
<= 100，且两者之和 <= 100。`initialize` 只能成功一次。

### `create_pool`

| 账户 | Mut | Signer | 说明 |
|---|---|---|---|
| `curve_configuration` | ✅ | | 全局配置 |
| `pool` | ✅ | | PDA `["liquidity_pool", mint]` |
| `token_mint` | ✅ | | Token-2022 mint |
| `pool_token_account` | ✅ | | ATA(mint, authority = pool) |
| `pool_sol_vault` | ✅ | | PDA `["liquidity_sol_vault", mint]` |
| `pool_sol_fee_vault` | ✅ | | PDA `["liquidity_sol_fee_vault", mint]` |
| `user_token_account` | ✅ | | 创建者 ATA，接收预铸 |
| `payer` | ✅ | ✅ | 创建账户并成为 `pool.creator` |
| `token_program` | | | Token-2022 程序 |
| `associated_token_program` | | | Associated Token 程序 |
| `rent` | | | Rent sysvar |
| `system_program` | | | System 程序 |

`payer` 必须当前是 mint authority。`create_pool` 会把 mint authority 转给 Pool PDA，
并把 `creator_premint` 铸到 `user_token_account`。

### `buy`

| 账户 | Mut | Signer | 说明 |
|---|---|---|---|
| `curve_configuration_account` | ✅ | | 全局配置 |
| `pool` | ✅ | | 必须匹配 mint 和存储的 bump |
| `token_mint` | ✅ | | Token-2022 mint |
| `pool_token_account` | ✅ | | ATA(mint, authority = pool) |
| `pool_sol_vault` | ✅ | | 曲线储备 |
| `pool_sol_fee_vault` | ✅ | | 费用金库 |
| `user_token_account` | ✅ | | 买方 ATA，接收 Share |
| `protocol_account` | ✅ | | 必须等于 `config.admin` |
| `pool_creator` | ✅ | | 必须等于 `pool.creator` |
| `user` | ✅ | ✅ | 支付 SOL |
| `rent` | | | Rent sysvar |
| `system_program` | | | System 程序 |
| `token_program` | | | Token-2022 程序 |
| `associated_token_program` | | | Associated Token 程序 |

参数：`amount_tokens: u64`（最小单位）。费用在曲线价格之外支付；链下应复现
`get_buy_price_after_fees` 的逻辑来报价。

### `sell`

账户与 `buy` 相同，只是 `user` 是卖方，`user_token_account` 必须持有 Share。参数：
`amount_tokens: u64`。费用从曲线收入中扣除。

### `distribute_user_fee`

| 账户 | Mut | Signer | 说明 |
|---|---|---|---|
| `pool` | ✅ | | 必须由 `user`（创建者）拥有 |
| `token_mint` | ✅ | | 用于 Pool PDA |
| `recipient` | ✅ | | 之后可以领取的用户钱包 |
| `user` | ✅ | ✅ | 必须等于 `pool.creator` |
| `rent` | | | Rent sysvar |
| `system_program` | | | System 程序 |

参数：`fee_share_percentage: u8`。

### `distribute_pool_fee`

| 账户 | Mut | Signer | 说明 |
|---|---|---|---|
| `pool` | ✅ | | 源池 |
| `token_mint` | ✅ | | 源 mint |
| `distribute_fee_pool` | ✅ | | 目标 Pool PDA |
| `distribute_fee_pool_token_mint` | ✅ | | 目标 mint |
| `user` | ✅ | ✅ | 必须等于 `pool.creator` |
| `rent` | | | Rent sysvar |
| `system_program` | | | System 程序 |

参数：`fee_share_percentage: u8`。目标池不能是源池自己。

### `claim_user_fees`

| 账户 | Mut | Signer | 说明 |
|---|---|---|---|
| `pool` | ✅ | | 持有费用账本的池子 |
| `token_mint` | ✅ | | 池子 mint |
| `pool_sol_fee_vault` | ✅ | | SOL 来源 |
| `fee_recipient` | ✅ | | 必须等于 `user` |
| `user` | ✅ | ✅ | 领取到该钱包 |
| `rent` | | | Rent sysvar |
| `system_program` | | | System 程序 |
| `token_program` | | | Token-2022 程序 |
| `associated_token_program` | | | Associated Token 程序 |

### `claim_pool_fees`

| 账户 | Mut | Signer | 说明 |
|---|---|---|---|
| `pool` | ✅ | | 源池（支付费用） |
| `pool_sol_fee_vault` | ✅ | | 源费用金库 |
| `token_mint` | ✅ | | 源 mint |
| `curve_configuration_account` | ✅ | | 全局配置 |
| `distribution_pool` | ✅ | | 目标池（接收买入） |
| `distribution_pool_token_mint` | ✅ | | 目标 mint |
| `distribution_pool_token_account` | ✅ | | 目标池 ATA |
| `distribution_pool_sol_vault` | ✅ | | 目标 SOL 金库 |
| `distribution_pool_sol_fee_vault` | ✅ | | 目标费用金库 |
| `distribution_pool_user_token_account` | ✅ | | 领取人的目标 Share ATA |
| `protocol_account` | ✅ | | 必须等于 `config.admin` |
| `distribution_pool_creator` | ✅ | | 必须等于 `distribution_pool.creator` |
| `user` | ✅ | ✅ | 必须等于 `distribution_pool.creator` |
| `rent` | | | Rent sysvar |
| `system_program` | | | System 程序 |
| `token_program` | | | Token-2022 程序 |
| `associated_token_program` | | | Associated Token 程序 |

领取会把源池的未领取费用转换成目标池 Share，并铸到
`distribution_pool_user_token_account`。

## 5. 事件

Anchor 事件名在 TypeScript 中会转成 camelCase。使用
`program.addEventListener("buyTokensEvent", ...)` 订阅。

| 事件 | 字段 |
|---|---|
| `PoolCreated` | `creator`、`token_mint`、`pool`、`pool_sol_vault`、`pool_sol_fee_vault`、`pool_token_account` |
| `BuyTokensEvent` | `pool`、`user`、`user_token_account`、`amount_tokens`、`price` |
| `SellTokensEvent` | `pool`、`user`、`user_token_account`、`amount_tokens`、`price` |
| `DistributeFeeEvent` | `pool`、`fee_share_percentage`、`recipient_type`、`recipient` |
| `CleanZeroFeeRecipientEvent` | `pool`、`fee_recipient` |
| `ClaimFeesEvent` | `pool`、`fee_recipient_type`、`fee_recipient`、`sender`、`claim_amount` |

buy/sell 事件里的 `price` 是本笔成交的含费价格，在 supply 更新前计算。精确到 lamports
的记账仍建议结合转账指令核对。

## 6. 错误码

| Code | 错误 | 常见原因 |
|---|---|---|
| 6000 | `InvalidFee` | 费率 > 100，或 `protocol_fees + creator_fees > 100` |
| 6001 | `InvalidAmount` | 数量为 0，或曲线价格为 0 |
| 6002 | `InsufficientBalance` | 买方 lamports 不足以支付曲线价格 |
| 6003 | `InsufficientFunds` | 费用金库不足以支付领取 |
| 6004 | `InvalidOwner` | 调用者不是池创建者或费用接收者 |
| 6005 | `InvalidProtocolAccount` | `protocol_account != config.admin` |
| 6006 | `SupplyNotEnough` | 卖出会把供给降到预铸地板以下 |
| 6007 | `PoolInsufficientFunds` | 储备不足以支付卖出 |
| 6008 | `InvalidRecipient` | 自指的分润目标，或找不到接收者 |
| 6009 | `InvalidFeeSharePercentage` | 份额 > 100 或总份额 > 100 |
| 6010 | `MaxFeeRecipientReached` | 添加第 11 个接收者 |

## 7. Token-2022 要求

- 使用 Token-2022 程序创建 mint。
- 如果使用 Metadata Pointer 扩展，请在创建 mint 的同一笔交易中、或在 `create_pool` 之前
  初始化 metadata。
- 在 `create_pool` 之前创建创建者 ATA。
- Pool ATA 由 `create_pool` 创建；买入/卖出用的用户 ATA 必须存在（或由客户端创建）。
- 带 transfer hook 或 transfer fee 的 mint 可能需要额外账户，当前指令集没有覆盖。
  请测试你计划支持的扩展组合。

## 8. 常见坑

1. **`create_pool` 签名者错误。** payer 必须是当前 mint authority。
2. **`pool_creator` 错误。** 必须严格等于 `pool.creator`，否则 `address` 约束失败。
3. **`protocol_account` 错误。** 必须等于 `config.admin`。
4. **把整枚代币当成最小单位。** `amount_tokens` 使用 mint decimals。
5. **没有滑点检查。** 客户端应计算最大 SOL 输入；程序目前没有 `max_sol_in`。
6. **领取人错误。** `claim_user_fees` 要求接收者签名；`claim_pool_fees` 要求目标池创建者。
7. **IDL 过期。** 指令或账户变更后重新生成类型。

## 9. 版本管理

IDL 和 TypeScript 类型随程序版本管理。在发布 `@stoa/idl` 包之前，集成方应固定到某个
release tag，并从该 tag 重新生成产物。账户布局变更是 breaking change。
