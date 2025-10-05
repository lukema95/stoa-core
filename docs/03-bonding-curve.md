# 03 Bonding Curve

[English](03-bonding-curve.md) | [简体中文](03-bonding-curve.zh.md)

The curve is the core of Stoa. This document derives the pricing code from the integral definition and explains the scaling constants, the premint floor, and the inverse pricing path.

## 1. Why a bonding curve

A new token has no initial counterparty. An AMM needs two-sided liquidity; an order book needs market makers. A bonding curve replaces both with a public function:

> Given current supply `S`, the price of buying `Δ` more shares is the area under the curve from `S` to `S + Δ`.

Properties:

- Buying moves supply right and makes the next trade more expensive (endogenous slippage).
- Selling moves supply left and lowers the price.
- Anyone can trade against the contract at any time; no counterparty is required.
- The SOL reserve is, in theory, the integral of all net buys.

Stoa uses a quadratic curve rather than a linear one: early shares are cheap, and the marginal price accelerates as attention grows.

## 2. Formula

Let `c = creator_premint` (the premint amount).

```text
f(x) = 0                  if x <= c
f(x) = (x - c)^2          if x > c
```

For a buy of `amount` shares at current supply `supply`:

```text
raw = f(supply + amount) - f(supply)
price_lamports = raw / 1_000_000_000 / 5_000
```

For a sell, use the post-sale supply as the starting point, which is equivalent to integrating from `supply - amount` to `supply`:

```text
sell_price = f(supply) - f(supply - amount)
           = get_price(supply - amount, amount)
```

Corresponding code:

```rust
fn curve(&self, cfg, amount: u64) -> u128 {
    if amount <= cfg.creator_premint {
        return 0;
    }
    let x = amount as u128 - cfg.creator_premint as u128;
    x * x
}

fn get_price(&self, cfg, supply, amount) -> u64 {
    (self.curve(cfg, supply + amount) - self.curve(cfg, supply))
        / 1_000_000_000
        / 5_000
}

fn get_buy_price(...)  { self.get_price(cfg, supply, amount) }
fn get_sell_price(...) { self.get_price(cfg, supply - amount, amount) }
```

### Where the scaling constants come from

`f(x)` is a squared supply difference. With 9 decimals, buying one whole token means `1e9` base units, whose square is `1e18`; using that directly as lamports would be enormous.

`/ 1e9 / 5000` maps curve units to SOL. Equivalently:

```text
1 lamport of curve area = 5e12
```

The inverse function multiplies SOL back:

```rust
let target_price_delta = (sol_amount as u128) * 1_000_000_000 * 5_000;
```

**Note:** this is a unit conversion, not an economic constant. Changing `5000` shifts every pool's price up or down.

## 3. The economics of the premint `c`

At pool creation, the creator receives `c` shares immediately, and:

```rust
if amount <= creator_premint {
    return 0;
}
```

So the first `c` shares have zero curve value. They are ownership/governance shares, not shares purchased from the market.

Sell protection:

```rust
if self.total_supply - creator_premint < amount {
    return Err(SupplyNotEnough);
}
```

This means `total_supply` can never fall below `c`. Only the circulating supply (`total_supply - c`) is redeemable. The creator cannot sell through the premint floor; doing so would push `f(x)` into the flat zero region and break reserve accounting.

```text
price
  ^
  |                 /
  |               /
  |             /
  |           /
  |_________/____________> supply
            c
       premint floor   circulating supply follows (x-c)^2
```

## 4. Expanded form (for hand calculations and tests)

When `supply >= c` and `supply + amount >= c`, let `s = supply - c`:

```text
f(s + a) - f(s) = (s + a)^2 - s^2 = 2*s*a + a^2
price = (2*s*a + a^2) / 5e12
```

Special cases:

- At pool creation `s = 0`, buying `a` shares costs only `a^2 / 5e12`; the first buy is extremely cheap.
- When supply is large, the leading term is `2*s*a`, approximately linear in current supply.

This is a discrete bonding curve: a trade pays the integral over the whole `Δ`, not the instantaneous marginal price `2(x-c)` one unit at a time. Large orders move their own price.

## 5. Inverse function: given SOL, how many shares?

`claim_pool_fees` needs to know how many target-pool shares a given amount of unclaimed SOL can buy.

Requirement:

```text
f(supply + amount) - f(supply) = sol * 5e12
```

Let `b = max(0, supply - c)`:

```text
amount^2 + 2*b*amount - target = 0
amount = -b + sqrt(b^2 + target)
```

The implementation uses `f64`:

```rust
let discriminant = (b * b + target_price_delta) as f64;
let amount_f64 = discriminant.sqrt() - b as f64;
let result = amount_f64.floor().max(1) as u64;
```

### Risk note

On Solana BPF, `f64` is deterministic for the same input, so validators do not disagree. However:

1. Converting `u128` to `f64` loses integer precision above `2^53`; large supplies can make `sqrt` slightly inaccurate.
2. `floor` gives the user a slightly smaller share amount; the remaining lamports are still charged by `get_buy_price(token_amount)`.
3. A production-grade implementation should use an integer square root (Newton iteration or `isqrt`).

`get_tokens_for_sol_after_fees` deducts the protocol and creator fees first, then runs the inverse function on the net amount, because a cross-pool claim buys through the normal fee path and the target pool charges fees again.

## 6. Quotes with fees

```rust
buy_after_fees  = price + price * creator_fees / 100 + price * protocol_fees / 100
sell_after_fees = price - price * creator_fees / 100 - price * protocol_fees / 100
```

With the defaults 5% + 2%:

- Buying one unit of curve price costs the user `1.07`.
- Selling one unit of curve price returns `0.93`.

The spread comes from the fee design, not from an AMM invariant. The curve itself uses the same integral for buys and sells, so there is no round-trip loss beyond fees.

Note: `buy` transfers the **pre-fee** `get_buy_price`; fees are charged separately from the user's wallet in `distribute_fees`. Clients must quote with `get_buy_price_after_fees` to show the total SOL required.

## 7. `total_supply` vs Token-2022 mint supply

The program tracks `pool.total_supply`; the Token-2022 mint also tracks its own supply.

Normal paths keep them in sync:

| Action | Mint supply | `pool.total_supply` |
|---|---|---|
| `create_pool` premint | `+c` | `= c` |
| `buy` | `+amount` | `+amount` |
| `sell` | `-amount` | `-amount` |
| `claim_pool_fees` | `+amount` | `+amount` |

Pricing uses **`pool.total_supply`**, not the mint account. Any path that makes them diverge distorts pricing. `buy_with_vault` (cross-pool claims) increases `total_supply` and `reserve_sol` while minting the same amount of shares to the claimant's ATA, so the two supplies stay aligned.

## 8. Relationship to instantaneous price

Marginal price (buying one more base unit):

```text
p(x) ~= [f(x+1) - f(x)] / 5e12 ~= 2(x-c) / 5e12
```

Integral price (buying `Δ` at once):

```text
integral = [f(x+Δ) - f(x)] / 5e12
```

If a UI displays the marginal price, a large order will pay more than that. The curve is "slippage-aware" by construction.
