# Stoa Documentation

[English](README.md) | [简体中文](README.zh.md)

This directory contains the technical documentation for Stoa Core. Every document is maintained in English and Simplified Chinese. The English version is the source of truth; Chinese files use the `.zh.md` suffix.

## Start here

- New to Stoa? Read [Architecture](01-architecture.md), then [Bonding Curve](03-bonding-curve.md).
- Integrating a client? Read [Integrators](06-integrators.md) and [Accounts & PDAs](02-accounts-pdas.md).
- Auditing or reviewing? Read [Security](09-security.md), [Core Flows](04-core-flows.md), and [Fee System](05-fee-system.md).
- Deploying? Read [Deployment](08-deployment.md) and [Release Process](10-release.md).
- Contributing? Read [Testing](07-testing.md) and the repository [CONTRIBUTING.md](../CONTRIBUTING.md).

## Documentation map

| # | Document | English | 简体中文 | What it covers |
|---|---|---|---|---|
| — | Index | [README.md](README.md) | [README.zh.md](README.zh.md) | Navigation and conventions |
| 01 | Architecture | [01-architecture.md](01-architecture.md) | [01-architecture.zh.md](01-architecture.zh.md) | System boundaries, modules, instructions, account graph, fund flows |
| 02 | Accounts & PDAs | [02-accounts-pdas.md](02-accounts-pdas.md) | [02-accounts-pdas.zh.md](02-accounts-pdas.zh.md) | Account layouts, PDA seeds, authorities, validation layers |
| 03 | Bonding curve | [03-bonding-curve.md](03-bonding-curve.md) | [03-bonding-curve.zh.md](03-bonding-curve.zh.md) | Curve math, scaling, premint floor, inverse pricing |
| 04 | Core flows | [04-core-flows.md](04-core-flows.md) | [04-core-flows.zh.md](04-core-flows.zh.md) | `initialize`, `create_pool`, `buy`, `sell` sequences |
| 05 | Fee system | [05-fee-system.md](05-fee-system.md) | [05-fee-system.zh.md](05-fee-system.zh.md) | Protocol/creator fees, sharing, cross-pool claims |
| 06 | Integrators | [06-integrators.md](06-integrators.md) | [06-integrators.zh.md](06-integrators.zh.md) | PDAs, instruction accounts, events, errors, TypeScript examples |
| 07 | Testing | [07-testing.md](07-testing.md) | [07-testing.zh.md](07-testing.zh.md) | Unit/integration tests, local validator, CI, troubleshooting |
| 08 | Deployment | [08-deployment.md](08-deployment.md) | [08-deployment.zh.md](08-deployment.zh.md) | Keypairs, build, local/devnet deploy, initialize, verify |
| 09 | Security model | [09-security.md](09-security.md) | [09-security.zh.md](09-security.zh.md) | Trust assumptions, invariants, risks, known limitations |
| 10 | Release process | [10-release.md](10-release.md) | [10-release.zh.md](10-release.zh.md) | Versioning, checklists, artifacts, IDL package, post-release |
| 11 | FAQ | [11-faq.md](11-faq.md) | [11-faq.zh.md](11-faq.zh.md) | Common questions from users and integrators |

## Documentation conventions

- **English is canonical.** If a translation and the English text disagree, the English text wins.
- **Bilingual in the same pull request.** Add or update `<file>.zh.md` whenever you change `<file>.md`.
- **Language switcher.** Every document starts with `[English](FILE.md) | [简体中文](FILE.zh.md)`.
- **No secrets.** Never put private keys, program keypairs, or `.env` values in docs.
- **Versioned behavior.** If a document describes behavior that is not yet released, mark it clearly (for example, "planned" or "unreleased").
- **Program identifiers are literal.** Keep instruction names, account names, seeds, and program IDs unchanged across languages.

Run `./scripts/check-docs.sh` from the repository root to verify pairing and language links.

## One-sentence summary

Stoa turns a Token-2022 mint into a continuously priced bonding-curve market with program-owned SOL vaults and an on-chain fee-sharing ledger.
