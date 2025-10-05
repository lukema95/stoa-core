# 10 发布流程

[English](10-release.md) | [简体中文](10-release.zh.md)

本文说明 Stoa Core 的版本、构建、验证和发布方式，面向维护者。

## 1. 版本管理

Stoa 遵循 [Semantic Versioning](https://semver.org/)：

- **MAJOR** —— 指令、账户布局、PDA seeds 或 program ID 的 breaking change。
- **MINOR** —— 向后兼容的指令、账户或功能。
- **PATCH** —— 向后兼容的修复、文档或构建变更。

项目处于 `0.x` 时，minor release 也可能包含 breaking change。每个 breaking change 都必须
在 `CHANGELOG.md` 和 release notes 中说明。

协议版本、IDL 版本和客户端包版本应同步推进。

## 2. 发布历史

| 版本 | 日期 | 说明 |
|---|---|---|
| 0.1.0 | 2025-10-05 | 首次提交并公开发布 |

## 3. 发布检查清单

- [ ] `main` 为绿色且是最新。
- [ ] `CHANGELOG.md` 和 `CHANGELOG.zh.md` 已更新。
- [ ] 文档中英文都已更新；`./scripts/check-docs.sh` 通过。
- [ ] `Cargo.toml` 和 `programs/stoa/Cargo.toml` 版本已提升。
- [ ] 如果客户端产物变化，`package.json` 版本已提升。
- [ ] `cargo fmt --all -- --check` 通过。
- [ ] `cargo clippy --all-targets -- -D warnings` 通过。
- [ ] `cargo test -p stoa` 通过。
- [ ] 在干净 checkout 上 `anchor build` 成功。
- [ ] `anchor test --validator legacy` 通过。
- [ ] `./scripts/check-program-id.sh` 通过。
- [ ] 部署者持有 program keypair 或 upgrade authority。
- [ ] 主网：独立审计已完成并链接报告。

## 4. 构建产物

```bash
anchor build
```

发布产物：

| 产物 | 路径 | 是否提交 |
|---|---|---|
| SBF 程序 | `target/deploy/stoa.so` | 否 |
| IDL | `target/idl/stoa.json` | 可选 |
| TypeScript 类型 | `target/types/stoa.ts` | 可选 |
| Program keypair | `target/deploy/stoa-keypair.json` | **绝不** |

生成校验和：

```bash
shasum -a 256 target/deploy/stoa.so target/idl/stoa.json target/types/stoa.ts
```

把 `.so`、IDL、类型和校验和附到 GitHub release。绝不附上 keypair。

## 5. 打 tag

```bash
git tag -s v0.1.0 -m "Stoa Core v0.1.0"
git push origin v0.1.0
```

使用签名 tag。Tag 必须指向产生这些产物的 commit。

## 6. GitHub release

Release notes 应包含：

- 变更摘要。
- Breaking change 和升级说明。
- Program ID 和 cluster 状态。
- 精确工具链版本（Rust、Solana、Anchor、Node）。
- 产物校验和。
- 审计报告链接（如有）。
- 完整 changelog 链接。

## 7. IDL 包

计划发布带版本的 `@stoa/idl` 包。发布后：

1. 从 tag commit 构建 IDL 和类型。
2. 包版本与程序版本一致。
3. 带 provenance 发布。
4. 在包 README 中记录 program ID 和 cluster。
5. 保持包版本与程序版本同步。

在此之前，集成方应固定 release tag，并从该 tag 重新生成产物。

## 8. 部署

遵循 [08-deployment.zh.md](08-deployment.zh.md)。发布时：

1. 用记录的 upgrade authority 部署 `.so`。
2. 验证 program ID 和 data length。
3. 只有全新部署才执行 initialize。
4. 跑冒烟测试：创建池、买入、卖出、领取。
5. 公告部署，附上 program ID 和 commit。

## 9. 安全发布

安全修复：

1. 与报告者私下协调。
2. 准备修复和回归测试。
3. 构建并验证产物。
4. 发布 patch 版本和公告。
5. 如报告者愿意，进行署名。
6. 在用户有时间升级后公开细节。

披露前不要在 commit message 中包含利用细节。

## 10. 回滚

保留上一个 `.so`、IDL 和 commit hash。回滚时重新构建上一个 tag，并用同一个 upgrade
authority 部署。公告回滚及其数据影响。

## 11. 发布后

- 监控程序和事件索引器是否异常。
- 如果行为与 release notes 不一致，更新文档。
- 为延后事项开 follow-up issue。
- 在 `CHANGELOG.md` 中记录发布。
