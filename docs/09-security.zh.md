# 09 安全模型

[English](09-security.md) | [简体中文](09-security.zh.md)

> **Stoa 是 alpha 软件，尚未审计。** 在没有独立安全审查的情况下，不要部署到主网，也不要
> 使用真实资金。

## 1. 审计状态

| 项目 | 状态 |
|---|---|
| 独立第三方审计 | ❌ 未完成 |
| 内部评审 | ✅ 初始发布前已完成 |
| 形式化验证 | ❌ 未进行 |
| 漏洞赏金 | ❌ 暂无 |
| 主网部署 | ❌ 不建议 |

本文描述预期的安全属性和已知缺口，不能替代审计。

## 2. 信任假设

- **协议管理员。** `CurveConfiguration.admin` 接收协议费。`buy`、`sell` 和
  `claim_pool_fees` 要求 `protocol_account` 等于该地址。生产环境应使用多签保护。
- **Upgrade authority。** 程序以可升级 BPF 程序部署。控制 upgrade authority 的人可以
  修改程序逻辑。应放在多签后面，并公开升级策略。
- **Token-2022。** 程序假设 Token-2022 按规范运行。不支持的扩展可能导致失败或行为不同；
  见第 6 节。
- **SOL / rent。** 金库是 rent-exempt PDA。程序不会补 rent；集成方需要正确注资。
- **链下身份。** 程序不证明创建者有权代币化某个资产。这是集成方/产品的责任。

## 3. 不变量

程序设计上保持：

1. 一个 mint 最多一个 `Pool`，因为 Pool PDA 由 mint 推导。
2. `create_pool` 之后，Pool PDA 是 mint authority；只有程序可以 mint。
3. `pool.total_supply` 永远不低于 `curve_configuration.creator_premint`。
4. `pool.reserve_sol` 等于所有买入曲线价之和减去所有卖出曲线价之和。
5. 所有费用接收者份额之和不超过 100。
6. `pool_sol_vault` 和 `pool_sol_fee_vault` 由程序拥有并由 mint 推导；调用者不能替换成
   任意账户。
7. `total_supply` 与 Token-2022 mint supply 在每条 mint/burn 路径上同步变化，包括跨池
   领取。
8. 池子不能把费用分配给自己。
9. 只有池创建者可以配置费用份额；只有接收者可以领取用户费用；只有目标池创建者可以领取
   池费用。

## 4. 账户校验

- PDA 使用 `seeds` + `bump` 约束。
- `buy` / `sell` 中 `pool_creator` 使用 `address = pool.creator` 约束。
- `claim_pool_fees` 中 `distribution_pool_creator` 使用
  `address = distribution_pool.creator` 约束。
- `protocol_account` 在 `buy`、`sell` 和 `claim_pool_fees` 中与
  `curve_configuration.admin` 校验。
- 代币账户使用关联代币账户约束，校验 mint 和 authority。
- 新增指令时，所有剩余的 `/// CHECK` 账户都必须评审。

新增指令时：

1. 优先使用 `has_one`、`address`、`seeds`，而不是手写校验。
2. 为每个 `/// CHECK` 账户写明校验理由。
3. 为每个授权边界补负向测试。
4. 账户布局变化时重跑账户大小和序列化测试。

## 5. 经济风险

- **没有链上滑点限制。** `buy` 接收 `amount_tokens`，不是最大 SOL 输入；`sell` 接收
  `amount_tokens`，不是最小 SOL 输出。交易可能被夹。集成方应在客户端计算限制，或等待
  后续版本加入 `max_sol_in` / `min_sol_out`。
- **费用参数全局且不可变。** 本版本没有 `update_config`。改费率需要升级程序或重新部署。
- **取整。** 百分比是 0–100 的整数，整数除法向下取整；灰尘可能留在金库或付款账户。
- **反函数使用 `f64`。** `get_tokens_for_sol` 用浮点平方根把 SOL 数量转成代币数量。
  在 Solana 上确定，但超大供给会损失精度。后续版本应使用整数平方根。
- **`initialize` 是首调用者模式。** 首个调用者成为 admin。部署和初始化应尽量放在同一个
  受控任务中。如果攻击者先初始化，配置就被劫持。
- **预铸和供给边界。** 协议没有限制 `creator_premint` 的上限；极端值会给曲线算术带来
  压力。集成方应选择合理的 decimals 和预铸量。

## 6. Token-2022 扩展注意事项

程序创建关联代币账户并使用 `mint_to` / `burn`。带 transfer hook、transfer fee、
confidential transfer 或其他转账时行为的扩展，可能需要额外账户，当前指令集也未必支持。

部署前：

- 测试你计划支持的精确扩展组合。
- 检查扩展是否改变账户大小或需要额外账户。
- 为集成方记录不支持的扩展。

## 7. 已知限制

- 没有正式审计。
- 没有 pause / emergency stop。
- 没有 `update_config`；初始化后 admin 和费用参数固定。
- 没有链上滑点参数。
- 没有 `creator_premint` 的协议级上限。
- 多条指令路径仍有调试 `msg!`。
- `total_fee_share_percentage` 是 `u16`；代码在接受分润前检查累计值，但如果增加接收者
  或字段，应重新评审账户布局。
- 除非部署者配置多签，upgrade authority 是单把密钥。
- IDL 包尚未发布；在发布前可能存在产物漂移。

## 8. 事件响应

如果漏洞被利用：

1. 暂停集成，并通过仓库和官方渠道警告用户。
2. 保留日志和交易签名。
3. 如果程序可升级，与 upgrade-authority 多签协调修复。
4. 修复部署后发布复盘。

## 9. 报告

见 [SECURITY.zh.md](../SECURITY.zh.md)。不要在公开 issue 中报告漏洞。披露政策、范围和安
全港都在那里定义。

## 10. 参考

- [架构](01-architecture.zh.md)
- [账户与 PDA](02-accounts-pdas.zh.md)
- [核心流程](04-core-flows.zh.md)
- [费用系统](05-fee-system.zh.md)
- [部署](08-deployment.zh.md)
