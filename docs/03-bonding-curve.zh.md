# 03 联合曲线原理

[English](03-bonding-curve.md) | [简体中文](03-bonding-curve.zh.md)

曲线是 Stoa 的核心。本文从「积分定价」推导到代码里的 `/ 1e9 / 5000`，并解释缩放常数、预铸地板和反函数定价。

## 1. 为什么用 Bonding Curve

开源仓库的 Share **没有初始对手盘**。AMM 需要双边流动性；订单簿需要做市商。Bonding Curve 用一条公开函数：

> 当前供给为 `S` 时，再买 `Δ` 枚的价格 = 曲线从 `S` 积到 `S+Δ` 的面积。

性质：

- 买让供给右移，下一笔更贵（滑点内生）。
- 卖让供给左移，价格回落。
- 任意时刻都可以和合约成交，不依赖另一个人挂单。
- 储备 SOL 理论上等于历史上净买入的曲线积分。

Stoa 选二次曲线而不是线性：早期更便宜、随着关注度上升边际价格加速，符合「明星项目溢价」。

## 2. 公式

记 `c = creator_premint`（预铸量）。

```
f(x) = 0                 若 x ≤ c
f(x) = (x - c)²          若 x > c
```

买入 `amount` 枚、当前供给 `supply`：

```
raw = f(supply + amount) - f(supply)
price_lamports = raw / 1_000_000_000 / 5_000
```

卖出时用**卖完之后**的供给当起点，等价于「从 `supply - amount` 积到 `supply`」：

```
sell_price = f(supply) - f(supply - amount)
           = get_price(supply - amount, amount)
```

对应代码：

```rust
fn curve(&self, cfg, amount: u64) -> u128 {
    if amount <= cfg.creator_premint { return 0; }
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

### 缩放常数从哪来

`f(x)` 的值域是「供给差的平方」，数量级是 token 最小单位的平方（9 位小数时一次买 1 枚 = `1e9` 个最小单位，平方是 `1e18`）。直接当 lamports 会大到离谱。

`/ 1e9 / 5000` 把曲线单位压到 SOL。可以把它记成：

```
1 lamport 对应的曲线面积 = 5e12
```

反函数 `get_tokens_for_sol` 里把 SOL 乘回去：

```rust
let target_price_delta = (sol_amount as u128) * 1_000_000_000 * 5_000;
```

**注意**：这不是经济学常数，是把整数曲线映射到 lamports 的量纲转换。改 `5000` 会整体平移价格，所有池一起变贵或变便宜。

## 3. 预铸 `c` 的经济学含义

创建池时立刻给 creator mint `c` 枚，且：

```rust
if amount <= creator_premint { return 0; }
```

所以 **前 `c` 枚在曲线上价值为 0**。它们是「治理/所有权份额」，不是从市场上买来的。

卖出保护：

```rust
if self.total_supply - creator_premint < amount {
    return Err(SupplyNotEnough);
}
```

含义：链上 `total_supply` **永远不能低于 `c`**。流通盘（`total_supply - c`）才是可赎回的。创建者钱包里虽有预铸仓位，但不能把曲线卖穿到预铸以下——否则 `f(x)` 会掉进恒 0 区间，储备会计会坏掉。

白板图：

```
price
  ^
  |                 /
  |               /
  |             /
  |           /
  |_________/____________> supply
            c
         预铸地板   流通盘沿 (x-c)² 上升
```

## 4. 展开式（手算/验证用）

当 `supply ≥ c` 且 `supply + amount ≥ c`，令 `s = supply - c`：

```
f(s+a) - f(s) = (s+a)² - s² = 2s·a + a²
price = (2s·a + a²) / 5e12
```

特例：

- 刚建池 `s = 0`，买 `a` 枚只要 `a² / 5e12`。第一笔极便宜。
- 供给已经很大时，主导项是 `2s·a`，近似线性于当前供给。

这就是「离散联合曲线」：不是按瞬时边际价 `2(x-c)` 一枚一枚买，而是对整段 `Δ` 积分，**大单自己吃自己的滑点**。

## 5. 反函数：已知 SOL，求能买多少枚

`claim_pool_fees` 需要「手里有一笔待领 SOL，能在目标池买多少 token」。

要求：

```
f(supply + amount) - f(supply) = sol * 5e12
```

令 `b = max(0, supply - c)`：

```
amount² + 2b·amount - target = 0
amount = -b + sqrt(b² + target)
```

代码用 `f64`：

```rust
let discriminant = (b * b + target_price_delta) as f64;
let amount_f64 = discriminant.sqrt() - b as f64;
let result = amount_f64.floor().max(1) as u64;
```

### 为什么这是一个可以主动讲的风险点

Solana BPF 上 `f64` 对同一输入是确定的，所以不会像跨客户端共识那样「算不出同一个块」。但：

1. `u128 → f64` 超过 2^53 会丢整数精度，大供给时 `sqrt` 可能偏。
2. `floor` 导致用户用同样 SOL 算出的枚数略少，多出来的 lamports 仍按 `get_buy_price(token_amount)` 扣。
3. 工业级实现会用整数平方根（牛顿迭代 / `isqrt`）。

`get_tokens_for_sol_after_fees` 先扣协议费+创建者费，再用净额反推枚数——因为跨池 claim 走的是「用 fee vault 付款」的买路径，目标池仍会抽一次费。

## 6. 含费报价

```rust
buy_after_fees  = price + price * creator_fees/100 + price * protocol_fees/100
sell_after_fees = price - price * creator_fees/100 - price * protocol_fees/100
```

默认 5% + 2%：

- 买 1 单位曲线价，用户实付 `1.07`
- 卖 1 单位曲线价，用户实收 `0.93`

买卖价差来自手续费，不是来自 AMM 的 k。曲线本身买卖是同一条积分（无 round-trip 损耗），**损耗全部是协议设计的税**。

注意：`buy()` 真正转账用的是不含费的 `get_buy_price`，费用在 `distribute_fees` 里**另外**从用户钱包扣。前端必须按 `get_buy_price_after_fees` 展示「你要准备多少 SOL」。

## 7. `total_supply` vs mint supply

程序自己维护 `pool.total_supply`，同时 Token-2022 mint 也有 `supply`。

正常路径二者一起变：

| 动作 | mint | pool.total_supply |
|------|------|-------------------|
| create_pool 预铸 | +c | = c |
| buy | +amount | +amount |
| sell | -amount | -amount |

曲线计算用的是 **pool.total_supply**，不是 mint 账户。因此任何让两者分叉的路径都会让定价失真。`buy_with_vault`（跨池领取）在增加 `total_supply` 与 `reserve_sol` 的同时，会把等量 Share mint 到领取人的 ATA，保持 `total_supply` 与 Token-2022 mint supply 一致。

## 8. 和瞬时价格的关系

边际价（再买 1 个最小单位）：

```
p(x) ≈ [f(x+1) - f(x)] / 5e12 ≈ 2(x-c) / 5e12
```

积分价（一次买 `Δ`）：

```
∫ p = [f(x+Δ)-f(x)] / 5e12
```

用户看到的「当前价」如果用边际价，大单实际付款会更高。产品文案里的「slippage-aware」指的就是这段积分。
