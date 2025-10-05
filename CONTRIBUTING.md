# Contributing to Stoa

[English](CONTRIBUTING.md) | [简体中文](CONTRIBUTING.zh.md)

Thanks for your interest in improving Stoa. This guide covers the development workflow, coding standards, and review process.

By participating, you agree to follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Ways to contribute

- Report bugs and usability issues using the issue templates.
- Improve documentation, examples, or translations.
- Add tests or reproduce reported issues.
- Propose and implement protocol improvements.
- Review pull requests and participate in design discussions.
- Report security issues privately according to [SECURITY.md](SECURITY.md).

## Development setup

### Prerequisites

- Rust stable (1.88+; pinned in `rust-toolchain.toml`)
- Solana CLI 2.1.x
- Anchor CLI 0.32.1
- Node.js 20+ and Yarn 1.22+

### Install and build

```bash
git clone https://github.com/lukema95/stoa-core.git
cd stoa-core
yarn install
anchor build
```

### Run tests

```bash
# Rust unit tests (account layout, curve math)
cargo test -p stoa

# TypeScript integration tests on a local validator
anchor test --validator legacy

# Static checks
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
yarn lint
npx tsc --noEmit -p tsconfig.json
./scripts/check-program-id.sh
./scripts/check-docs.sh
```

Anchor 0.32 defaults to the Surfpool validator. Use `--validator legacy` to run against the standard `solana-test-validator`.

### Local program keypair

The program keypair is not committed. For local development, Anchor can generate one, or you can create it explicitly:

```bash
mkdir -p target/deploy
solana-keygen new --no-bip39-passphrase -o target/deploy/stoa-keypair.json
```

Never commit `target/deploy/*-keypair.json` or any private key.

## Project layout

```text
programs/stoa/src/
├── lib.rs          instruction dispatch
├── state.rs        account structs, curve math, fees, events
├── errors.rs       CustomError
└── instructions/   account constraints and thin handlers
tests/stoa.ts       integration tests
docs/               bilingual documentation
```

## Coding standards

### Rust

- Run `cargo fmt` before committing.
- Prefer explicit checked arithmetic (`checked_add`, `checked_mul`, `u128`) for financial math.
- Put account validation in `#[derive(Accounts)]` constraints; keep handlers thin.
- Do not add `/// CHECK` accounts without a documented reason and, where possible, an `address`/`seeds` constraint.
- Use `Result<()>` and `CustomError` for expected failures; avoid `unwrap`/`expect` in program code.
- Add or update unit tests for account layout and pure math changes.
- Keep debug `msg!` logging out of hot paths unless it is behind a feature flag.

### TypeScript

- Follow the repository Prettier configuration (`yarn lint:fix`).
- Tests must not depend on committed keypairs; generate keypairs at runtime.
- Prefer explicit instruction account names over implicit resolution.

### Documentation

- Every user-facing document has an English file and a Simplified Chinese counterpart named `<name>.zh.md`.
- Add the language switcher to the top of both files:

  ```markdown
  [English](FILE.md) | [简体中文](FILE.zh.md)
  ```

- Update both languages in the same pull request.
- Run `./scripts/check-docs.sh` to verify pairing and language links.
- Document account layout or IDL changes in `docs/` and note them as breaking changes for integrators.

## Tests

- Add integration coverage for every behavior change.
- Add a regression test for every bug fix.
- For account layout changes, update `Pool::ACCOUNT_SIZE` and the unit tests in `state.rs`.
- Negative tests are expected: unauthorized callers, invalid fee shares, insufficient balances, and boundary values.
- CI runs `cargo test`, `anchor build`, `anchor test --validator legacy`, formatting, and documentation checks.

## Commit and pull request conventions

- Use clear, imperative commit messages, for example:
  - `fix: validate pool_creator in buy and sell`
  - `docs: add integrator account tables`
  - `test: cover cross-pool claim minting`
- Keep pull requests focused. Separate refactors from behavior changes.
- Describe the problem, the solution, and the test evidence.
- Reference related issues.
- Call out breaking changes to accounts, instructions, IDL, or program ID.
- Update `CHANGELOG.md` for user-visible changes.
- Ensure all CI checks pass before requesting review.

## Review and merge

- At least one maintainer approval is required.
- Security-sensitive changes require review from a maintainer with security ownership.
- Changes to the program ID, upgrade authority, fee model, or account layout require maintainer consensus; see [GOVERNANCE.md](GOVERNANCE.md).
- Squash or rebase to keep the `main` history linear unless a merge commit is explicitly requested.

## Security

Do not report security vulnerabilities in public issues. Follow [SECURITY.md](SECURITY.md).

## License

Stoa is licensed under the MIT License. By contributing, you agree that your contributions are licensed under the same terms. No separate CLA is required.

## Translations

Translation improvements are welcome. Keep the English version as the source of truth, preserve technical terms and program identifiers, and update both files in the same pull request.

## Questions

See [SUPPORT.md](SUPPORT.md) for discussion channels and issue guidance.
