# 11 FAQ

[English](11-faq.md) | [简体中文](11-faq.zh.md)

## 通用

### Stoa 是什么？

Stoa 是一个开源 Solana 程序，为 Token-2022 mint 提供持续的联合曲线市场。买入时铸币，
卖出时销毁，交易费用在协议、池创建者和配置的接收者之间分配。

### Stoa 审计过吗？

没有。Stoa 是 alpha 软件。见 [09-security.zh.md](09-security.zh.md)。在没有独立审计的
情况下，不要部署到主网或使用真实资金。

### Canonical program ID 是什么？

```text
7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM
```

已配置到 localnet 和 devnet。Devnet 部署待完成。

### 为什么叫 Stoa？

名字源自古希腊城市的柱廊：一个人们相遇、交流思想的公共结构。协议希望成为代币社区的
公共市场结构。

## 协议

### 价格怎么计算？

曲线是 `f(x) = (x - c)^2`（`x > c`），其中 `c` 是创建者预铸量。买入支付从当前 supply
到 supply + amount 的积分，再按 `/ 1e9 / 5000` 缩放成 lamports。见
[03-bonding-curve.zh.md](03-bonding-curve.zh.md)。

### 为什么买入费用在价格之外？

买入时，买方把完整曲线价格打入储备，并另外支付协议费/创建者费。卖出时费用从收入中
扣除。这种不对称让 `reserve_sol` 始终等于曲线价格之和，与费用参数无关。见
[05-fee-system.zh.md](05-fee-system.zh.md)。

### 创建者初始化后能改费率吗？

不能。`CurveConfiguration` 只初始化一次，本版本没有 `update_config`。改费率需要升级
程序或重新部署。

### 什么是创建者预铸？

创建池时创建者获得 `creator_premint` 枚 Share。这些 Share 的曲线价值为 0，并形成地板：
`total_supply` 永远不能卖到 `creator_premint` 以下。

### 卖到预铸地板以下会怎样？

交易失败，返回 `SupplyNotEnough`。只有预铸以上的流通 Share 才能卖回储备。

### 池子能把费用分给自己吗？

不能。`distribute_pool_fee` 会用 `InvalidRecipient` 拒绝源池自己。

### 一个池子最多多少个费用接收者？

10 个。账户按 10 个接收者分配空间；添加第 11 个会返回 `MaxFeeRecipientReached`。

### 什么是跨池领取？

池子可以把部分创建者费路由到另一个池。目标池创建者领取时，累积的 SOL 会用来买入目标
池 Share，并铸给领取人。见 [05-fee-system.zh.md](05-fee-system.zh.md)。

## 集成方

### Stoa 使用哪个代币程序？

只使用 Token-2022。创建池和交易指令都接收 `Program<Token2022>`。

### 可以用带 transfer hook 或 transfer fee 的 mint 吗？

当前指令集只测试了测试套件使用的标准 Token-2022 扩展（Metadata Pointer 和 Token
Metadata）。Transfer hook、transfer fee 和 confidential transfer 可能需要额外账户，
或者不受支持。部署前请测试你的精确 mint 配置。

### 为什么 `buy` 接收 `amount_tokens` 而不是 SOL？

这样 Share 数量是确定的，SOL 成本由曲线推导。代价是链上没有最大 SOL 输入。客户端应
用 `get_buy_price_after_fees` 报价，并执行客户端限制。

### 怎么获取 IDL 和 TypeScript 类型？

运行 `anchor build`；产物在 `target/idl/stoa.json` 和 `target/types/stoa.ts`。计划发布
`@stoa/idl` 包。见 [06-integrators.zh.md](06-integrators.zh.md)。

### 为什么 `anchor test` 报 `Failed to spawn surfpool`？

Anchor 0.32 默认使用 Surfpool 验证器。传 `--validator legacy` 使用
`solana-test-validator`，或安装 Surfpool。见 [07-testing.zh.md](07-testing.zh.md)。

### 为什么 program ID 不一致？

`declare_id!`、`Anchor.toml` 和生成的 IDL 必须一致。运行
`./scripts/check-program-id.sh` 定位差异。见 [08-deployment.zh.md](08-deployment.zh.md)。

## 贡献

### 如何贡献？

见 [CONTRIBUTING.zh.md](../CONTRIBUTING.zh.md)。文档、测试、bug 报告、翻译和代码都欢迎。

### 需要签 CLA 吗？

不需要。贡献按 MIT 许可证授权。

### 如何报告安全问题？

私下发邮件到 `lukema95@gmail.com`。不要开公开 issue。见 [SECURITY.zh.md](../SECURITY.zh.md)。

### 在哪里提问？

见 [SUPPORT.zh.md](../SUPPORT.zh.md)。非安全问题使用 GitHub issues 或 discussions。
