# 为 Stoa 做贡献

[English](CONTRIBUTING.md) | [简体中文](CONTRIBUTING.zh.md)

感谢你愿意改进 Stoa。本文说明开发流程、编码规范和评审流程。

参与本项目即表示你同意遵守[行为准则](CODE_OF_CONDUCT.zh.md)。

## 贡献方式

- 使用 issue 模板报告 bug 和可用性问题。
- 改进文档、示例或翻译。
- 补充测试或复现已报告的问题。
- 提出并实现协议改进。
- 评审 pull request，参与设计讨论。
- 按照 [SECURITY.zh.md](SECURITY.zh.md) 私下报告安全问题。

## 开发环境

### 环境要求

- Rust stable（1.88+；由 `rust-toolchain.toml` 固定）
- Solana CLI 2.1.x
- Anchor CLI 0.32.1
- Node.js 20.18+ 和 Yarn 1.22+

### 安装与构建

```bash
git clone https://github.com/lukema95/stoa-core.git
cd stoa-core
yarn install
anchor build
```

### 运行测试

```bash
# Rust 单元测试（账户布局、曲线数学）
cargo test -p stoa

# 本地验证器上的 TypeScript 集成测试
anchor test --validator legacy

# 静态检查
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
yarn lint
npx tsc --noEmit -p tsconfig.json
./scripts/check-program-id.sh
./scripts/check-docs.sh
```

Anchor 0.32 默认使用 Surfpool 验证器。使用 `--validator legacy` 可以跑标准的 `solana-test-validator`。

### 本地 program keypair

程序 keypair 不会提交到仓库。本地开发时可以让 Anchor 自动生成，也可以显式创建：

```bash
mkdir -p target/deploy
solana-keygen new --no-bip39-passphrase -o target/deploy/stoa-keypair.json
```

永远不要提交 `target/deploy/*-keypair.json` 或任何私钥。

## 项目结构

```text
programs/stoa/src/
├── lib.rs          指令分发
├── state.rs        账户结构、曲线数学、费用、事件
├── errors.rs       CustomError
└── instructions/   账户约束和薄处理函数
tests/stoa.ts       集成测试
docs/               双语文档
```

## 编码规范

### Rust

- 提交前运行 `cargo fmt`。
- 金融计算优先使用显式 checked 运算（`checked_add`、`checked_mul`、`u128`）。
- 账户校验放在 `#[derive(Accounts)]` 约束里；处理函数保持轻薄。
- 不要在没有文档说明的情况下新增 `/// CHECK` 账户；尽量使用 `address`/`seeds` 约束。
- 预期失败使用 `Result<()>` 和 `CustomError`；程序代码里避免 `unwrap`/`expect`。
- 账户布局和纯数学逻辑的改动要补单元测试。
- 热路径中不要保留调试用的 `msg!`，除非它在 feature flag 后面。

### TypeScript

- 遵循仓库的 Prettier 配置（`yarn lint:fix`）。
- 测试不能依赖已提交的 keypair；运行时生成 keypair。
- 优先显式写出指令账户名，而不是依赖隐式解析。

### 文档

- 每个面向用户的文档都要有英文版和简体中文版，中文文件名是 `<name>.zh.md`。
- 两个文件顶部都要有语言切换：

  ```markdown
  [English](FILE.md) | [简体中文](FILE.zh.md)
  ```

- 在同一个 pull request 中同时更新两种语言。
- 运行 `./scripts/check-docs.sh` 检查配对和语言链接。
- 账户布局或 IDL 变更要写入 `docs/`，并标注为集成方的 breaking change。

## 测试

- 每个行为变更都要有集成测试覆盖。
- 每个 bug 修复都要有回归测试。
- 账户布局变更要同步更新 `Pool::ACCOUNT_SIZE` 和 `state.rs` 中的单元测试。
- 需要负向测试：未授权调用者、非法费用份额、余额不足和边界值。
- CI 会运行 `cargo test`、`anchor build`、`anchor test --validator legacy`、格式检查和文档检查。

## 提交与 pull request 规范

- 使用清晰的祈使句提交信息，例如：
  - `fix: validate pool_creator in buy and sell`
  - `docs: add integrator account tables`
  - `test: cover cross-pool claim minting`
- pull request 保持聚焦。重构和行为变更分开。
- 描述问题、解决方案和测试证据。
- 关联相关 issue。
- 明确说明账户、指令、IDL 或 program ID 的 breaking change。
- 用户可见变更要更新 `CHANGELOG.md`。
- 请求评审前确保所有 CI 检查通过。

## 评审与合并

- 至少需要一位维护者批准。
- 涉及安全的改动需要由负责安全的维护者评审。
- 修改 program ID、upgrade authority、费用模型或账户布局需要维护者共识；参见 [GOVERNANCE.zh.md](GOVERNANCE.zh.md)。
- 除非明确要求 merge commit，否则使用 squash 或 rebase 保持 `main` 线性历史。

## 安全

不要在公开 issue 中报告安全漏洞。请遵循 [SECURITY.zh.md](SECURITY.zh.md)。

## 许可证

Stoa 使用 MIT 许可证。提交贡献即表示你同意以相同条款授权你的贡献。不需要单独的 CLA。

## 翻译

欢迎改进翻译。英文版是事实来源；保留技术术语和程序标识符，并在同一个 pull request 中更新两个文件。

## 问题咨询

讨论渠道和 issue 指南见 [SUPPORT.zh.md](SUPPORT.zh.md)。
