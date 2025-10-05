# Changelog

[English](CHANGELOG.md) | [简体中文](CHANGELOG.zh.md)

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned

- Publish a versioned `@stoa/idl` package.
- Add on-chain slippage parameters (`max_sol_in` / `min_sol_out`).
- Add `update_config` and pause controls with a multisig-friendly authority model.
- Complete an independent security audit.

## [0.1.0] - 2025-10-05

### Added

- Initial commit and public release of Stoa Core.
- Quadratic bonding-curve pricing, Token-2022 mint/burn flows, protocol and
  creator fees, fee sharing, and cross-pool buybacks.
- TypeScript integration tests and Rust unit tests.
- Bilingual English / Simplified Chinese documentation.
- MIT license.

### Fixed

- Corrected `Pool` account size for the maximum number of fee recipients.
- Constrained `pool_creator` and `distribution_pool_creator` to the pool creator.
- Minted shares during cross-pool claims so supply and Token-2022 supply stay in sync.
- Used the target pool's supply when calculating cross-pool claim amounts.
- Enforced `protocol_fees + creator_fees <= 100` at initialization.
- Emitted buy/sell events with the price calculated before the supply update.
- Widened `total_fee_share_percentage` to `u16` to avoid overflow.

### Security

- Removed test keypairs from the repository. The test wallets should be
  rotated if they ever held value.
