# 更新日志

[English](CHANGELOG.md) | [简体中文](CHANGELOG.zh.md)

本文件记录项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)，
版本号遵循 [Semantic Versioning](https://semver.org/spec/v2.0.0.html)。

## [Unreleased]

### 计划

- 发布带版本的 `@stoa/idl` 包。
- 增加链上滑点参数（`max_sol_in` / `min_sol_out`）。
- 增加 `update_config` 和 pause 控制，并采用适合多签的权限模型。
- 完成独立安全审计。

## [0.1.0] - 2025-10-05

### 新增

- Stoa Core 首次提交并公开发布。
- 二次联合曲线定价、Token-2022 铸销流程、协议费和创建者费、费用分润、
  跨池回购。
- TypeScript 集成测试和 Rust 单元测试。
- 英文 / 简体中文双语文档。
- MIT 许可证。

### 修复

- 修正 `Pool` 账户在最大费用接收者数量下的空间。
- 将 `pool_creator` 和 `distribution_pool_creator` 约束到池创建者。
- 跨池领取时实际 mint Share，保持 supply 与 Token-2022 supply 一致。
- 计算跨池领取数量时使用目标池的 supply。
- 初始化时强制 `protocol_fees + creator_fees <= 100`。
- 在 supply 更新前计算价格并发出 buy/sell 事件。
- 将 `total_fee_share_percentage` 扩宽为 `u16`，避免溢出。
- 在 `claim_pool_fees` 中校验 `protocol_account`。
- 允许费用接收者表满时更新已有接收者。

### 安全

- 从仓库移除测试 keypair。测试钱包如果曾持有价值，应立即轮换。
