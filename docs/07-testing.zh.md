# 07 测试

[English](07-testing.md) | [简体中文](07-testing.zh.md)

Stoa 使用两层测试：Rust 单元测试覆盖纯逻辑和账户布局，TypeScript 集成测试在本地验证器上
覆盖完整指令流程。

## 1. 测试策略

| 层级 | 位置 | 覆盖内容 |
|---|---|---|
| Rust 单元测试 | `programs/stoa/src/**/*.rs`（`#[cfg(test)]`） | 账户大小、序列化、纯数学辅助函数 |
| TypeScript 集成测试 | `tests/stoa.ts` | 对已部署程序的完整指令流程 |
| 静态检查 | CI | `cargo fmt`、`cargo clippy`、Prettier、TypeScript、program ID、文档配对 |

集成测试是有状态的，按顺序执行：initialize、airdrop、创建 mint、创建池、分润、买入、
卖出、领取。

## 2. 环境要求

- Rust stable（由 `rust-toolchain.toml` 固定）
- Solana CLI 2.1.x
- Anchor CLI 0.32.1
- Node.js 20+ 和 Yarn 1.22+

```bash
yarn install
anchor build
```

## 3. 运行测试

```bash
# Rust 单元测试
cargo test -p stoa

# 本地验证器上的 TypeScript 集成测试
anchor test --validator legacy

# CI 使用的静态检查
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
yarn lint
npx tsc --noEmit -p tsconfig.json
./scripts/check-program-id.sh
./scripts/check-docs.sh
```

Anchor 0.32 默认使用 Surfpool 验证器。使用 `--validator legacy` 可以跑标准的
`solana-test-validator`。

`anchor test` 会启动全新的本地验证器、部署 `target/deploy/stoa.so` 并运行测试套件。
本地部署使用 `target/deploy/stoa-keypair.json`，该文件不会提交。

## 4. 集成测试覆盖

| 测试 | 覆盖 |
|---|---|
| `Initialize Contract` | 用费用和预铸量创建全局 `CurveConfiguration` |
| `Airdrop SOL` | 给运行时生成的测试用户空投 SOL |
| `Create TokenMint, Metadata and PDA` | 带 Metadata Pointer 和 metadata 字段的 Token-2022 mint、PDA |
| `Create Pool` | 两个不同创建者和 mint 的池子 |
| `Distribute Fee` | 用户接收者和池接收者、总份额记账 |
| `Buy Token` | 曲线价格、费用转账、supply/reserve 更新、费用累积 |
| `Sell Token` | 销毁、储备减少、费用扣除、地板行为 |
| `Claim Fee` | 用户费用领取和跨池领取（实际 mint 目标 Share） |

跨池领取测试会断言领取人的目标代币余额增加，用于保护 `buy_with_vault` 的 mint 路径。

## 5. 单元测试

Rust 测试目前覆盖：

- `FeeRecipient` 的 Borsh 长度正好是 50 字节。
- `Pool::ACCOUNT_SIZE` 能容纳 10 个接收者和 discriminator。
- Anchor 生成的 program ID 测试。

修改账户字段时：

1. 更新 `Pool::ACCOUNT_SIZE` 或 `InitSpace` 推导。
2. 更新序列化测试。
3. 运行 `cargo test -p stoa`。
4. 重新构建并重跑集成测试。

## 6. 编写新的集成测试

- 运行时生成 keypair；永远不要提交私钥。
- 从 `program.programId` 推导 PDA。
- 在 `.accounts({...})` 中显式写出账户名，以便发现 IDL 漂移。
- 空投足够的 SOL 用于 rent 和交易费。
- 涉及价值转移时，同时断言代币余额和 lamport 余额。
- 为你修复的失败模式补一个负向测试。

示例骨架：

```ts
it("rejects a non-creator fee distribution", async () => {
  try {
    await program.methods
      .distributeUserFee(10)
      .accounts({
        pool: poolPda,
        tokenMint: mint,
        recipient: attacker.publicKey,
        user: attacker.publicKey,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([attacker])
      .rpc();
    assert.fail("expected InvalidOwner");
  } catch (err) {
    // Anchor 把 CustomError::InvalidOwner 暴露为 0x1774（6004）。
    expect(err.toString()).to.contain("InvalidOwner");
  }
});
```

## 7. CI

GitHub Actions workflow：

1. 安装 Solana 和 Anchor。
2. 创建本地钱包并准备 `~/.cache/solana`。
3. 从 `yarn.lock` 安装 JS 依赖。
4. 运行格式检查、构建程序、运行 Rust 单元测试。
5. 检查 program ID 一致性和文档配对。
6. 运行 `anchor test --skip-build --validator legacy`。

## 8. 排错

### `Failed to spawn surfpool`

Anchor 0.32 默认使用 Surfpool。传 `--validator legacy`，或安装 Surfpool。

### `cargo-build-sbf` 报 `No such file or directory`

创建缓存目录：

```bash
mkdir -p ~/.cache/solana
```

### `cargo-build-sbf` 找不到 `solana-install`

较新的 Solana release 提供的是 `agave-install`。创建兼容符号链接：

```bash
SOLANA_BIN="$HOME/.local/share/solana/install/active_release/bin"
ln -sf "$SOLANA_BIN/agave-install" "$SOLANA_BIN/solana-install"
```

### 空投失败

本地验证器水龙头可能限制大额空投。降低金额或拆成多次请求。

### 本地通过但 CI 失败

先检查 Anchor、Solana 和 Node 版本。CI 使用 README 中的版本；浮动的版本是常见故障源。
