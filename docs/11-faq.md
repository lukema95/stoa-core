# 11 FAQ

[English](11-faq.md) | [简体中文](11-faq.zh.md)

## General

### What is Stoa?

Stoa is an open-source Solana program that gives a Token-2022 mint a continuous bonding-curve market. Buys mint shares, sells burn them, and trading fees are split between the protocol, the pool creator, and configured recipients.

### Is Stoa audited?

No. Stoa is alpha software. See [09-security.md](09-security.md). Do not deploy to mainnet or use real funds without an independent audit.

### What is the canonical program ID?

```text
7kn2AcW5rJStbiAdKKrZ4cfZGR32eERDzPch7wJQk8xM
```

It is configured for localnet and devnet. Devnet deployment is pending.

### Why "Stoa"?

The name is a reference to the covered walkways of ancient Greek cities: a shared public structure where people meet and exchange ideas. The protocol aims to be a shared public market structure for token communities.

## Protocol

### How is the price calculated?

The curve is `f(x) = (x - c)^2` for `x > c`, where `c` is the creator premint. A buy pays the integral from current supply to supply plus amount, scaled by `/ 1e9 / 5000` into lamports. See [03-bonding-curve.md](03-bonding-curve.md).

### Why are buy fees charged on top of the price?

On buy, the buyer pays the full curve price into the reserve and pays protocol/creator fees separately. On sell, fees are deducted from the proceeds. This asymmetry keeps `reserve_sol` equal to the sum of curve prices, independent of fee parameters. See [05-fee-system.md](05-fee-system.md).

### Can the creator change fees after initialization?

No. `CurveConfiguration` is initialized once and there is no `update_config` in this version. Changing fees requires a program upgrade or a new deployment.

### What is the creator premint?

The creator receives `creator_premint` shares at pool creation. Those shares have zero curve value and form a floor: `total_supply` can never be sold below `creator_premint`.

### What happens if I try to sell below the premint floor?

The transaction fails with `SupplyNotEnough`. Only circulating shares above the premint can be sold back into the reserve.

### Can a pool allocate fees to itself?

No. `distribute_pool_fee` rejects the source pool with `InvalidRecipient`.

### How many fee recipients can a pool have?

Up to 10. The account is sized for 10 recipients; adding an 11th returns `MaxFeeRecipientReached`.

### What is a cross-pool claim?

A pool can route part of its creator fee to another pool. When the target pool's creator claims, the accrued SOL is used to buy target-pool shares, which are minted to the claimant. See [05-fee-system.md](05-fee-system.md).

## Integrators

### Which token program does Stoa use?

Token-2022 only. The program takes `Program<Token2022>` in pool creation and trading instructions.

### Can I use a mint with transfer hooks or transfer fees?

The current instruction set is tested with the standard Token-2022 extensions used by the test suite (Metadata Pointer and Token Metadata). Transfer hooks, transfer fees, and confidential transfers may require extra accounts or may not be supported. Test your exact mint configuration before deploying.

### Why does `buy` take `amount_tokens` instead of SOL?

It makes the share amount deterministic; the SOL cost is derived from the curve. The trade-off is that there is no on-chain maximum SOL input. Clients should quote with `get_buy_price_after_fees` and enforce a client-side limit.

### How do I get the IDL and TypeScript types?

Run `anchor build`; artifacts are written to `target/idl/stoa.json` and `target/types/stoa.ts`. A published `@stoa/idl` package is planned. See [06-integrators.md](06-integrators.md).

### Why does `anchor test` fail with `Failed to spawn surfpool`?

Anchor 0.32 defaults to the Surfpool validator. Pass `--validator legacy` to use `solana-test-validator`, or install Surfpool. See [07-testing.md](07-testing.md).

### Why does the program ID not match?

`declare_id!`, `Anchor.toml`, and the generated IDL must agree. Run `./scripts/check-program-id.sh` to find the mismatch. See [08-deployment.md](08-deployment.md).

## Contributing

### How can I contribute?

See [CONTRIBUTING.md](../CONTRIBUTING.md). Documentation, tests, bug reports, translations, and code are all welcome.

### Do I need a CLA?

No. Contributions are licensed under MIT.

### How do I report a security issue?

Email `lukema95@gmail.com` privately. Do not open a public issue. See [SECURITY.md](../SECURITY.md).

### Where can I ask questions?

See [SUPPORT.md](../SUPPORT.md). Use GitHub issues or discussions for non-security questions.
