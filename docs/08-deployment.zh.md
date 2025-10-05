# 08 部署

[English](08-deployment.md) | [简体中文](08-deployment.zh.md)

本文覆盖本地和 devnet 部署、program ID 管理、初始化和验证。主网部署必须先完成独立审计。

## 1. 环境要求

```bash
rustc --version        # stable，1.88+（或使用 rust-toolchain.toml）
solana --version       # 2.1.x
anchor --version       # 0.32.1
node --version         # 20+
yarn --version         # 1.22+
```

```bash
yarn install
anchor build
```

## 2. Program keypair

Canonical program ID 已提交在 `programs/stoa/src/lib.rs` 和 `Anchor.toml`。Keypair **不会**
提交。

创建或恢复：

```bash
mkdir -p target/deploy
solana-keygen new --no-bip39-passphrase -o target/deploy/stoa-keypair.json
solana-keygen pubkey target/deploy/stoa-keypair.json
```

公钥必须匹配：

```text
7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM
```

请安全备份 `target/deploy/stoa-keypair.json`。丢失它不影响升级（upgrade authority 可以
升级），但无法从零重建同一个 program ID。

如果确实要使用不同的 ID：

1. 更新 `programs/stoa/src/lib.rs` 中的 `declare_id!`。
2. 更新 `Anchor.toml` 中的 `[programs.localnet]` 和 `[programs.devnet]`。
3. 运行 `./scripts/check-program-id.sh`。
4. 更新下游客户端和文档。

永远不要提交 keypair，也不要把 keypair 放进公开的 CI 产物。

## 3. 构建

```bash
anchor build
```

产物：

- `target/deploy/stoa.so` —— SBF 程序
- `target/deploy/stoa-keypair.json` —— 程序 keypair（机密）
- `target/idl/stoa.json` —— Anchor IDL
- `target/types/stoa.ts` —— TypeScript 类型

构建后运行 `./scripts/check-program-id.sh`，确认 `declare_id!`、`Anchor.toml` 和 IDL 一致。

## 4. 本地测试

```bash
anchor test --validator legacy
```

这会启动本地 `solana-test-validator`、部署程序并运行 `tests/stoa.ts` 中的集成测试。

Anchor 0.32 默认使用 Surfpool。使用 `--validator legacy` 走标准验证器，或按喜好安装
Surfpool。

## 5. 部署前检查清单

- [ ] 在干净 checkout 上 `anchor build` 成功。
- [ ] `cargo test -p stoa` 通过。
- [ ] `anchor test --validator legacy` 通过。
- [ ] `cargo fmt --all -- --check` 和 `cargo clippy --all-targets -- -D warnings` 通过。
- [ ] `./scripts/check-program-id.sh` 通过。
- [ ] Program keypair 已备份，部署者可用。
- [ ] Upgrade authority 是项目控制的多签。
- [ ] 记录了 release commit 和工具链版本。
- [ ] 主网部署：独立安全审计已完成。

## 6. Devnet 部署

```bash
solana config set --url devnet
solana balance
anchor deploy --provider.cluster devnet
```

如果钱包需要资金：

```bash
solana airdrop 2
```

### 只初始化一次

`initialize` 只能成功一次，首个调用者会成为 admin 和协议费收款地址。请把部署和初始化
放在同一个受控任务中，尽量缩小被抢跑窗口。

```ts
const [config] = PublicKey.findProgramAddressSync(
  [Buffer.from("CurveConfiguration")],
  program.programId,
);

await program.methods
  .initialize(new anchor.BN(2), new anchor.BN(1_000_000_000), new anchor.BN(5))
  .accounts({
    curveConfigurationAccount: config,
    admin: provider.wallet.publicKey,
    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

如果程序已经初始化，不要再运行。本版本没有 `update_config`。

## 7. 验证

```bash
solana program show 7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM --url devnet
```

检查：

- Program owner 是 BPF upgradeable loader。
- Upgrade authority 是预期的多签。
- 部署的 data length 与 release 一致。
- IDL address 与 program ID 一致。

可复现构建请使用 `solana-verify` 或 `anchor verify`，并在 release notes 中记录精确的
commit 和工具链。

## 8. 集群矩阵

| Cluster | Program ID | 状态 |
|---|---|---|
| Localnet | `7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM` | 已由 `anchor test --validator legacy` 验证 |
| Devnet | `7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM` | 待部署 |
| Mainnet | — | 未部署；需要审计 |

## 9. Upgrade authority

- 使用多签，不要用单个热钱包。
- 在 release notes 中记录 authority 地址和升级策略。
- 如果部署不可升级程序，只有在代码最终确定并审计通过后才把 authority 设为 `none`。
- 改变账户布局的升级需要升级计划。

## 10. 回滚

Anchor 部署是升级，不是原子回滚。回滚步骤：

1. 构建上一个已审计的 commit。
2. 确认 program ID 和 IDL。
3. 用同一个 upgrade authority 部署之前的 `.so`。
4. 公告回滚及其数据影响。

为每个 release 保留 `.so`、IDL 和 commit hash。

## 11. 排错

### `cargo-build-sbf` 报 `No such file or directory`

```bash
mkdir -p ~/.cache/solana
```

Solana 2.1.x 要求缓存目录存在。

### `cargo-build-sbf` 找不到 `solana-install`

较新的 Solana release 提供 `agave-install`。如果 Anchor 版本仍调用旧名字，创建兼容符号链接：

```bash
SOLANA_BIN="$HOME/.local/share/solana/install/active_release/bin"
ln -sf "$SOLANA_BIN/agave-install" "$SOLANA_BIN/solana-install"
```

### `Failed to spawn surfpool`

传 `--validator legacy`，或安装 Surfpool。

### Program ID 不一致

```bash
./scripts/check-program-id.sh
```

它会比较 `declare_id!`、`Anchor.toml` 和生成的 IDL address。
