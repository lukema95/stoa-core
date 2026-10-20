# 09 Security Model

[English](09-security.md) | [简体中文](09-security.zh.md)

> **Stoa is alpha software and has not been audited.** Do not deploy to mainnet or use it with real funds without an independent security review.

## 1. Audit status

| Item | Status |
|---|---|
| Independent third-party audit | ❌ Not completed |
| Internal review | ✅ Completed before the initial release |
| Formal verification | ❌ Not performed |
| Bug bounty | ❌ Not available |
| Mainnet deployment | ❌ Not recommended |

This document describes the intended security properties and the known gaps. It is not a substitute for an audit.

## 2. Trust assumptions

- **Protocol admin.** `CurveConfiguration.admin` receives the protocol fee. `buy`, `sell`, and `claim_pool_fees` require `protocol_account` to equal this address. Protect the admin key with a multisig in production.
- **Upgrade authority.** The program is deployed as an upgradeable BPF program. Whoever controls the upgrade authority can change program logic. Put it behind a multisig and publish an upgrade policy.
- **Token-2022.** The program assumes the Token-2022 program behaves according to its specification. Mints with unsupported extensions may fail or behave differently; see section 6.
- **SOL / rent.** Vaults are rent-exempt PDAs. The program does not top up rent; integrators must fund accounts appropriately.
- **Off-chain identity.** The program does not prove that a creator is entitled to tokenize an asset. That is an integrator/product responsibility.

## 3. Invariants

The program is designed to preserve:

1. A mint has at most one `Pool`, because the pool PDA is derived from the mint.
2. After `create_pool`, the pool PDA is the mint authority; only the program can mint.
3. `pool.total_supply` never falls below `curve_configuration.creator_premint`.
4. `pool.reserve_sol` equals the sum of curve prices paid on buys minus curve prices released on sells.
5. The sum of all fee-recipient shares is at most 100.
6. `pool_sol_vault` and `pool_sol_fee_vault` are owned by the program and derived from the mint; callers cannot substitute arbitrary accounts.
7. `total_supply` and the Token-2022 mint supply move together on every mint or burn path, including cross-pool claims.
8. A pool cannot allocate fees to itself.
9. Only the pool creator can configure fee shares; only the recipient can claim user fees; only the target pool creator can claim pool fees.

## 4. Account validation

- PDAs use `seeds` + `bump` constraints.
- `pool_creator` is constrained with `address = pool.creator` in `buy` and `sell`.
- `distribution_pool_creator` is constrained with `address = distribution_pool.creator` in `claim_pool_fees`.
- `protocol_account` is checked in `buy`, `sell`, and `claim_pool_fees` against `curve_configuration.admin`.
- Token accounts are constrained as associated token accounts with the expected mint and authority.
- All remaining `/// CHECK` accounts must be reviewed when adding instructions.

When adding an instruction:

1. Prefer `has_one`, `address`, and `seeds` over manual checks.
2. Document every `/// CHECK` account and its validation.
3. Add a negative test for each authorization boundary.
4. Re-run the account-size and serialization tests if layout changes.

## 5. Economic risks

- **No on-chain slippage limit.** `buy` takes `amount_tokens`, not a maximum SOL input; `sell` takes `amount_tokens`, not a minimum SOL output. A transaction can be sandwiched. Integrators should compute limits client-side or wait for a future version that adds `max_sol_in` / `min_sol_out`.
- **Fee parameters are global and immutable.** There is no `update_config` in this version. Changing fees requires a program upgrade or a new deployment.
- **Rounding.** Percentages are integers out of 100 and integer division floors; dust can remain in vaults or payer accounts.
- **Inverse pricing uses `f64`.** `get_tokens_for_sol` converts a SOL amount to a token amount with a floating-point square root. It is deterministic on Solana but loses precision for very large supplies. A future version should use an integer square root.
- **`initialize` is first-caller.** The first caller becomes the admin. Deploy and initialize as close together as possible, ideally in one controlled deployment job. If an attacker initializes first, the configuration is compromised.
- **Premint and supply bounds.** There is no protocol-level cap on `creator_premint`; extremely large values can stress curve arithmetic. Integrators should choose sane token decimals and premint values.

## 6. Token-2022 extension caveats

The program creates associated token accounts and uses `mint_to` / `burn`. Mints with extensions that add transfer hooks, transfer fees, confidential transfers, or other transfer-time behavior may require additional accounts or may not be supported by the current instruction set.

Before deploying:

- Test the exact extension combination you intend to support.
- Check whether the extension changes account sizes or requires extra accounts.
- Document any unsupported extensions for integrators.

## 7. Known limitations

- No formal audit.
- No pause / emergency stop.
- No `update_config`; admin and fee parameters are fixed after initialization.
- No on-chain slippage parameters.
- No protocol-level cap on `creator_premint`.
- Debug `msg!` logging is present in several instruction paths.
- `total_fee_share_percentage` is a `u16`; the code checks the running total before accepting a distribution, but the account layout should be re-reviewed if recipients or fields are added.
- Upgrade authority is a single key unless the deployer configures a multisig.
- The IDL package is not yet published; artifact drift is possible until it is.

## 8. Incident response

If a vulnerability is exploited:

1. Pause integrations and warn users through the repository and official channels.
2. Preserve logs and transaction signatures.
3. If the program is upgradeable, coordinate a fix with the upgrade-authority multisig.
4. Publish a post-mortem after the fix is deployed.

## 9. Reporting

See [SECURITY.md](../SECURITY.md). Do not open public issues for vulnerabilities. The disclosure policy, scope, and safe harbor are defined there.

## 10. References

- [Architecture](01-architecture.md)
- [Accounts and PDAs](02-accounts-pdas.md)
- [Core flows](04-core-flows.md)
- [Fee system](05-fee-system.md)
- [Deployment](08-deployment.md)
