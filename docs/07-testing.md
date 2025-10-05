# 07 Testing

[English](07-testing.md) | [简体中文](07-testing.zh.md)

Stoa uses two test layers: Rust unit tests for pure logic and account layout, and TypeScript integration tests against a local validator.

## 1. Test strategy

| Layer | Location | What it covers |
|---|---|---|
| Rust unit tests | `programs/stoa/src/**/*.rs` (`#[cfg(test)]`) | Account sizes, serialization, pure math helpers |
| TypeScript integration | `tests/stoa.ts` | Full instruction flows against a deployed program |
| Static checks | CI | `cargo fmt`, `cargo clippy`, Prettier, TypeScript, program ID, docs pairing |

The integration suite is stateful and runs in order: initialize, airdrop, create mints, create pools, distribute fees, buy, sell, claim.

## 2. Prerequisites

- Rust stable (pinned by `rust-toolchain.toml`)
- Solana CLI 2.1.x
- Anchor CLI 0.32.1
- Node.js 20+ and Yarn 1.22+

```bash
yarn install
anchor build
```

## 3. Run the tests

```bash
# Rust unit tests
cargo test -p stoa

# TypeScript integration tests on a local validator
anchor test --validator legacy

# Static checks used by CI
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
yarn lint
npx tsc --noEmit -p tsconfig.json
./scripts/check-program-id.sh
./scripts/check-docs.sh
```

Anchor 0.32 defaults to the Surfpool validator. Use `--validator legacy` for the standard `solana-test-validator`.

`anchor test` starts a fresh local validator, deploys `target/deploy/stoa.so`, and runs the suite. The program keypair at `target/deploy/stoa-keypair.json` is used for the local deployment and is not committed.

## 4. What the integration tests cover

| Test | Coverage |
|---|---|
| `Initialize Contract` | Creates the global `CurveConfiguration` with fees and premint |
| `Airdrop SOL` | Funds generated test users on localnet |
| `Create TokenMint, Metadata and PDA` | Token-2022 mint with Metadata Pointer, metadata fields, PDAs |
| `Create Pool` | Two pools with different creators and mints |
| `Distribute Fee` | User recipient and pool recipient, total share accounting |
| `Buy Token` | Curve price, fee transfers, supply/reserve updates, fee accrual |
| `Sell Token` | Burn, reserve decrease, fee deduction, floor behavior |
| `Claim Fee` | User fee claim and cross-pool claim with minted target shares |

The cross-pool claim test asserts that the claimant's target-token balance increases, which guards the `buy_with_vault` mint path.

## 5. Unit tests

The Rust tests currently cover:

- `FeeRecipient` Borsh length is exactly 50 bytes.
- `Pool::ACCOUNT_SIZE` fits 10 recipients plus the discriminator.
- The Anchor-generated program ID test.

When changing account fields:

1. Update `Pool::ACCOUNT_SIZE` or the `InitSpace` derivation.
2. Update the serialization test.
3. Run `cargo test -p stoa`.
4. Rebuild and rerun the integration tests.

## 6. Writing a new integration test

- Generate keypairs at runtime; never commit private keys.
- Derive PDAs from the program ID returned by `program.programId`.
- Use explicit account names in `.accounts({...})` to catch IDL drift.
- Airdrop enough SOL for rent and trade fees.
- Assert both token balances and lamport balances where value moves.
- Add a negative test for the failure mode you are fixing.

Example skeleton:

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
    // Anchor surfaces CustomError::InvalidOwner as 0x1774 (6004).
    expect(err.toString()).to.contain("InvalidOwner");
  }
});
```

## 7. CI

The GitHub Actions workflow:

1. Installs Solana and Anchor.
2. Creates a local wallet and prepares `~/.cache/solana`.
3. Installs JS dependencies from `yarn.lock`.
4. Runs formatting, builds the program, runs Rust unit tests.
5. Checks program ID consistency and documentation pairing.
6. Runs `anchor test --skip-build --validator legacy`.

## 8. Troubleshooting

### `Failed to spawn surfpool`

Anchor 0.32 defaults to Surfpool. Pass `--validator legacy` or install Surfpool.

### `cargo-build-sbf` panics with `No such file or directory`

Create the cache directory:

```bash
mkdir -p ~/.cache/solana
```

### `cargo-build-sbf` cannot find `solana-install`

Newer Solana releases ship `agave-install`. Create a compatibility symlink:

```bash
SOLANA_BIN="$HOME/.local/share/solana/install/active_release/bin"
ln -sf "$SOLANA_BIN/agave-install" "$SOLANA_BIN/solana-install"
```

### Airdrop fails

The local validator faucet may rate-limit large airdrops. Lower the amount or split it into multiple requests.

### Tests pass locally but fail in CI

Check the Anchor, Solana, and Node versions first. CI uses the versions in the README; floating versions are a common source of failures.
