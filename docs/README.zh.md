# Stoa 文档

[English](README.md) | [简体中文](README.zh.md)

本目录包含 Stoa Core 的技术文档。每份文档都维护英文和简体中文两个版本。英文版是事实
来源；中文文件使用 `.zh.md` 后缀。

## 从这里开始

- 第一次接触 Stoa？先读[架构](01-architecture.zh.md)，再读[联合曲线](03-bonding-curve.zh.md)。
- 集成客户端？读[集成指南](06-integrators.zh.md)和[账户与 PDA](02-accounts-pdas.zh.md)。
- 审计或评审？读[安全模型](09-security.zh.md)、[核心流程](04-core-flows.zh.md)和[费用系统](05-fee-system.zh.md)。
- 部署？读[部署](08-deployment.zh.md)和[发布流程](10-release.zh.md)。
- 贡献代码？读[测试](07-testing.zh.md)和仓库的 [CONTRIBUTING.zh.md](../CONTRIBUTING.zh.md)。

## 文档地图

| # | 文档 | English | 简体中文 | 内容 |
|---|---|---|---|---|
| — | 索引 | [README.md](README.md) | [README.zh.md](README.zh.md) | 导航和约定 |
| 01 | 架构 | [01-architecture.md](01-architecture.md) | [01-architecture.zh.md](01-architecture.zh.md) | 系统边界、模块、指令、账户图、资金流 |
| 02 | 账户与 PDA | [02-accounts-pdas.md](02-accounts-pdas.md) | [02-accounts-pdas.zh.md](02-accounts-pdas.zh.md) | 账户布局、PDA 种子、权限、校验层 |
| 03 | 联合曲线 | [03-bonding-curve.md](03-bonding-curve.md) | [03-bonding-curve.zh.md](03-bonding-curve.zh.md) | 曲线数学、缩放、预铸地板、反函数 |
| 04 | 核心流程 | [04-core-flows.md](04-core-flows.md) | [04-core-flows.zh.md](04-core-flows.zh.md) | `initialize`、`create_pool`、`buy`、`sell` 时序 |
| 05 | 费用系统 | [05-fee-system.md](05-fee-system.md) | [05-fee-system.zh.md](05-fee-system.zh.md) | 协议费/创建者费、分润、跨池领取 |
| 06 | 集成指南 | [06-integrators.md](06-integrators.md) | [06-integrators.zh.md](06-integrators.zh.md) | PDA、指令账户、事件、错误码、TypeScript 示例 |
| 07 | 测试 | [07-testing.md](07-testing.md) | [07-testing.zh.md](07-testing.zh.md) | 单元/集成测试、本地验证器、CI、排错 |
| 08 | 部署 | [08-deployment.md](08-deployment.md) | [08-deployment.zh.md](08-deployment.zh.md) | Keypair、构建、本地/devnet 部署、初始化、验证 |
| 09 | 安全模型 | [09-security.md](09-security.md) | [09-security.zh.md](09-security.zh.md) | 信任假设、不变量、风险、已知限制 |
| 10 | 发布流程 | [10-release.md](10-release.md) | [10-release.zh.md](10-release.zh.md) | 版本、检查清单、产物、IDL 包、发布后 |
| 11 | FAQ | [11-faq.md](11-faq.md) | [11-faq.zh.md](11-faq.zh.md) | 用户和集成方的常见问题 |

## 文档约定

- **英文是事实来源。** 如果翻译与英文不一致，以英文为准。
- **同一个 pull request 内双语更新。** 修改 `<file>.md` 时，同时更新 `<file>.zh.md`。
- **语言切换。** 每份文档开头都有 `[English](FILE.md) | [简体中文](FILE.zh.md)`。
- **不要放密钥。** 文档中绝不能出现私钥、program keypair 或 `.env` 值。
- **标明版本。** 如果文档描述尚未发布的行为，请明确标注（例如 “planned” 或 “unreleased”）。
- **程序标识符保持原样。** 指令名、账户名、种子和 program ID 在中英文版本中不要翻译。

在仓库根目录运行 `./scripts/check-docs.sh` 可以检查配对和语言链接。

## 一句话总结

Stoa 把 Token-2022 mint 变成持续定价的联合曲线市场，SOL 储备和费用账本都在链上。
