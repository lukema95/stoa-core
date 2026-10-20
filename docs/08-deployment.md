# 08 Deployment

[English](08-deployment.md) | [简体中文](08-deployment.zh.md)

This guide covers local and devnet deployment, program ID management, initialization, and verification. Mainnet deployment requires an independent audit first.

## 1. Prerequisites

```bash
rustc --version        # stable, 1.88+ (or use rust-toolchain.toml)
solana --version       # 2.1.x
anchor --version       # 0.32.1
node --version         # 20.18+
yarn --version         # 1.22+
```

```bash
yarn install
anchor build
```

## 2. Program keypair

The canonical program ID is committed in `programs/stoa/src/lib.rs` and `Anchor.toml`. The keypair is **not** committed.

Create or restore it:

```bash
mkdir -p target/deploy
solana-keygen new --no-bip39-passphrase -o target/deploy/stoa-keypair.json
solana-keygen pubkey target/deploy/stoa-keypair.json
```

The public key must match:

```text
7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM
```

Back up `target/deploy/stoa-keypair.json` securely. Losing it does not prevent an upgrade (the upgrade authority can upgrade), but you cannot recreate the same program ID from scratch.

If you intentionally use a different ID:

1. Update `declare_id!` in `programs/stoa/src/lib.rs`.
2. Update `[programs.localnet]` and `[programs.devnet]` in `Anchor.toml`.
3. Run `./scripts/check-program-id.sh`.
4. Update downstream clients and documentation.

Never commit the keypair or place it in a public CI artifact.

## 3. Build

```bash
anchor build
```

Artifacts:

- `target/deploy/stoa.so` — SBF program
- `target/deploy/stoa-keypair.json` — program keypair (secret)
- `target/idl/stoa.json` — Anchor IDL
- `target/types/stoa.ts` — TypeScript types

Run `./scripts/check-program-id.sh` after the build. It verifies that `declare_id!`, `Anchor.toml`, and the IDL agree.

## 4. Local test

```bash
anchor test --validator legacy
```

This starts a local `solana-test-validator`, deploys the program, and runs the integration tests in `tests/stoa.ts`.

Anchor 0.32 defaults to Surfpool. Use `--validator legacy` for the standard validator, or install Surfpool if you prefer it.

## 5. Pre-deployment checklist

- [ ] `anchor build` succeeds on a clean checkout.
- [ ] `cargo test -p stoa` passes.
- [ ] `anchor test --validator legacy` passes.
- [ ] `cargo fmt --all -- --check` and `cargo clippy --all-targets -- -D warnings` pass.
- [ ] `./scripts/check-program-id.sh` passes.
- [ ] The program keypair is backed up and available to the deployer.
- [ ] The upgrade authority is a multisig controlled by the project.
- [ ] The release commit and toolchain versions are recorded.
- [ ] For mainnet: an independent security audit is complete.

## 6. Devnet deployment

```bash
solana config set --url devnet
solana balance
anchor deploy --provider.cluster devnet
```

If the wallet needs funds:

```bash
solana airdrop 2
```

### Initialize once

`initialize` can succeed only once, and the first caller becomes the admin and protocol fee recipient. Deploy and initialize as part of the same controlled job to minimize the front-running window.

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

If the program is already initialized, do not run this again. There is no `update_config` in this version.

## 7. Verification

```bash
solana program show 7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM --url devnet
```

Check:

- Program owner is the BPF upgradeable loader.
- Upgrade authority is the expected multisig.
- The deployed data length matches the release.
- The IDL address matches the program ID.

For reproducible builds, use `solana-verify` or `anchor verify` with the exact commit and toolchain recorded in the release notes.

## 8. Cluster matrix

| Cluster | Program ID | Status |
|---|---|---|
| Localnet | `7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM` | Tested by `anchor test --validator legacy` |
| Devnet | `7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM` | Deployment pending |
| Mainnet | — | Not deployed; requires audit |

## 9. Upgrade authority

- Use a multisig, not a single hot wallet.
- Record the authority address and upgrade policy in release notes.
- If you deploy an immutable program, set the authority to `none` only after the code is final and audited.
- Upgrades that change account layout require an upgrade plan.

## 10. Rollback

Anchor deployments are upgrades, not atomic rollbacks. To roll back:

1. Build the previous audited commit.
2. Verify the program ID and IDL.
3. Deploy the previous `.so` with the same upgrade authority.
4. Announce the rollback and any data implications.

Keep the `.so`, IDL, and commit hash for every release.

## 11. Troubleshooting

### `cargo-build-sbf` panics with `No such file or directory`

```bash
mkdir -p ~/.cache/solana
```

Solana 2.1.x expects the cache directory to exist.

### `cargo-build-sbf` cannot find `solana-install`

Newer Solana releases ship `agave-install`. Create a compatibility symlink if your Anchor version still calls the old name:

```bash
SOLANA_BIN="$HOME/.local/share/solana/install/active_release/bin"
ln -sf "$SOLANA_BIN/agave-install" "$SOLANA_BIN/solana-install"
```

### `Failed to spawn surfpool`

Pass `--validator legacy` or install Surfpool.

### Program ID mismatch

```bash
./scripts/check-program-id.sh
```

It compares `declare_id!`, `Anchor.toml`, and the generated IDL address.
