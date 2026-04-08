# MEV Bot — Production Build Context

## Goal
Build a production-grade MEV bot in Rust targeting:
- **Strategies:** DEX arbitrage (cross-DEX) + sandwich attacks (V2/V3)
- **Networks:** Ethereum mainnet, Arbitrum, Base
- **Submission:** Flashbots bundles + MEV-Share

## Engineer Profile
Senior Rust/Solidity/DeFi engineer, 5+ years. Deep EVM knowledge.
- Skip basic explanations
- Go straight to architecture, edge cases, production concerns
- Idiomatic Rust, performance-first

---

## Reference Codebases (`codebases/`)

### Sandwich Bots
| Codebase | Key Value |
|----------|-----------|
| `rusty-sando` | Best reference — V2/V3 multi-meat sandwiches, Artemis framework, Huff contracts, revm simulation |
| `sandooo` | Advanced — stablecoin sandwiches, multi-bundle grouping, revm, Telegram alerts, salmonella detection |
| `subway-rs` | Simpler — V2 only, good for Flashbots relayer pattern and benchmarks |

### Arbitrage
| Codebase | Key Value |
|----------|-----------|
| `simple-arbitrage` | TypeScript but shows clean Flashbots + BundleExecutor pattern |
| `mev-design` | LST arb + mempool monitor, uses Alloy + revm 18+, good for multi-chain patterns |
| `cex-dex-arb-research` | Research-grade CEX/DEX arbitrage analysis |

### Frameworks & Infrastructure
| Codebase | Key Value |
|----------|-----------|
| `artemis` | Paradigm's event-driven MEV framework — Collectors / Strategies / Executors pattern |
| `mev-rs` | MEV-Boost, relay, validator/builder integration |
| `cfmms-rs` | CFMM math and pool syncing utilities |
| `revm-is-all-you-need` | revm simulation patterns |
| `evm-simulation` | EVM simulation examples |
| `ethers-flashbots` | Flashbots middleware for ethers-rs |
| `evm-tracing-samples` | EVM tracing for simulation debugging |
| `mev-bundle-generator` | Bundle construction patterns |

---

## AMM Math (Internalize This)

### Uniswap V2 — Constant Product
```
x * y = k
amountOut = (amountIn * 997 * reserveOut) / (reserveIn * 1000 + amountIn * 997)
```
- 0.30% fee (0.3% on in, so 99.7% of amountIn used)
- Price impact ≈ 2 × (order_size / pool_size) as a rough estimate
- Minimum cost on any arb: 0.60% spread (two hops)

### Uniswap V3
- Use `QuoterV2.quoteExactInputSingle()` for off-chain simulation
- Tick-based liquidity — price impact is non-linear per tick range
- Always simulate via revm, not off-chain math (too complex for production)

### Curve
- `get_dy(i, j, amount)` for stablecoin pool quotes
- StableSwap invariant — very low slippage near peg

---

## Simulation Engine (Critical Layer)

**Rule: Never submit without local simulation. eth_call is free. Reverted Flashbots bundles cost nothing.**

### Simulation methods (in order of preference):
1. **revm fork simulation** — fork chain state at target block, replay full bundle
   - See: `codebases/revm-is-all-you-need`, `codebases/mev-design/crates/simulator/`
2. **Protocol-specific math** — fast pre-filter (V2 getAmountOut), run first, simulate second
3. **eth_call virtual contracts** — deploy fake contract on-the-fly for ERC-20 tax detection

### Salmonella / Trap Token Detection
```
1. Fork state, set fake balance via state override
2. Call transferFrom(victim, bot, amount)
3. Assert received == sent
4. If received < sent → fee-on-transfer token → skip
```
Common traps: burn-on-transfer, whitelist/blacklist, pausable, owner-controlled params.
See: `docs/Avoiding Trap Tokens With eth_call.md`

---

## Sandwich Strategy

### Mechanics
```
Block N:
  [frontrun]  buy X tokens (inflates price)
  [victim tx] victim buys at inflated price
  [backrun]   sell X tokens for profit
```

### Profit formula
```
profit = backrun_out - frontrun_in - gas_cost - bribe
```

### Production thresholds (from docs)
- Typical sandwich profit: 1–5% of victim trade size
- Bribe to builder: 50–80% of gross profit
- At least 480,000 sandwiches/year on mainnet → high competition
- Target: victims with high slippage tolerance (>0.5%) on mid-size pools

### Contract
- Use Huff for gas efficiency (see `rusty-sando/contract/src/sando.huff`)
- Multi-meat sandwich: multiple victims grouped in one bundle (see `sandooo`)
- Deploy separately per chain — gas models differ

---

## Arbitrage Strategy

### Atomic DEX Arb
- **Mainnet reality**: <$1 profit per trade, 91–99% bid to builder. Extremely competitive.
- **Target instead**: Low-liquidity tokens, fresh pools, L2s (ARB, Base)
- **Execution**: BundleExecutor contract + flashloan for capital efficiency
  - See: `simple-arbitrage/contracts/BundleExecutor.sol`

### Cycle Detection
- Build directed weighted graph: pools as edges, tokens as nodes
- Weight = -log(exchange_rate) → Bellman-Ford finds negative cycles = profitable arb
- Update graph on every new block (pool state changes)

### CeFi-DeFi Arb (higher alpha)
- Buy on-chain at discount, hedge off-chain on CEX simultaneously
- Revenue split: only 35–77% bid to builder (vs 91–99% for atomic)
- Requires: $100k+ capital, inventory management, CEX API latency optimization
- From docs: $37.8M in Q1 2023 vs $25M for atomic arb

---

## Execution Venues

### Ethereum Mainnet
- **Submit via**: Flashbots MEV-Boost (private bundles)
- **Competition**: Extreme — 75% of miners use Flashbots, 50% of blocks have bundles
- **Winning bid**: 91–99% of profit for atomic arb
- **Best for**: Sandwich attacks (higher margins than arb)

### Arbitrum
- Centralized sequencer → coordinate for N+1 block placement
- Private mempool → less frontrunning competition
- Different gas model: L1 calldata cost + L2 execution cost

### Base
- OP Stack, EIP-1559
- Sequencer-based ordering → similar to Arbitrum
- Growing DEX volume, less saturated than mainnet

### MEV-Share
- Users share orderflow hints, receive 40–60% of MEV extracted
- Matchmaker pairs user txs with searcher bundles
- Use for backrunning user transactions with partial info
- See: `docs/MEV-Share - programmably private orderflow.md`

---

## Architecture

### Artemis Framework Pattern
```
Collectors → Engine (broadcast channel) → Strategy → Executors
```
- Collector types: `MempoolCollector`, `BlockCollector`, `LogCollector`
- Strategy: stateful, receives events, emits actions
- Executor: `FlashbotsExecutor`, custom per-chain
- See: `codebases/artemis/crates/artemis-core/`

### Stack
- **Framework:** Artemis (event-driven, composable)
- **EVM Simulation:** `revm` with forked state
- **Chain interaction:** `alloy` (preferred, modern) or `ethers-rs`
- **Contracts:** Huff (sandwich) + Solidity (arb executor)
- **Bundle submission:** `ethers-flashbots` + MEV-Share API
- **Async:** tokio with broadcast channels
- **Logging:** `tracing` crate + CSV for PnL + Telegram alerts

### Multi-Chain
- Separate RPC connections per chain
- Shared strategy logic, chain-specific collectors/executors
- Self-hosted nodes required (public RPCs rate-limit under MEV load)

---

## Production Numbers

### Infrastructure costs
- Mainnet full node: ~$150/month (24h sync)
- Multi-chain (ETH + ARB + Base): ~$750/month (192GB RAM VPS)
- RPC services (fallback): $50–$1000+/month

### Capital requirements
- Atomic arb with flashloans: ~$10k minimum (flashloan covers rest)
- Sandwich attacks: $10k–$50k (inventory in bot contract)
- CeFi-DeFi: $100k+ (inventory + hedging capital)

### Gas budget per trade
- Simple V2 swap: ~21,000–80,000 gas
- Complex V3 + multi-hop: up to 500,000 gas
- Sandwich (front + back): ~150,000–300,000 gas total

---

## Risk Management (Non-Negotiable)

### Killswitch
Monitor bot contract balance every block:
```rust
if balance < MIN_BALANCE_THRESHOLD {
    // disable submissions, alert operator
}
```

### Failed tx protection
- Flashbots: reverted bundles cost $0 → always simulate
- Public mempool sidechains: failed txs cost gas → tight filtering required

### Inclusion risk
- Bundle may not be included (outbid, reorg)
- Never assume inclusion; always verify on next block

### PGA (Priority Gas Auction) Arms Race
- On non-Flashbots chains: winning = outbidding 10+ bots simultaneously
- Second place pays 50%+ of profit in gas
- Strategy: accurate profit estimation + aggressive bidding or don't play

---

## Development Order

1. **Simulation engine** — revm fork + local bundle replay (foundation for everything)
2. **Mempool streaming** — pending tx decoding, V2/V3 swap detection
3. **Sandwich bot V2** — front/back run construction, Flashbots submission
4. **Sandwich bot V3** — tick-aware simulation, revm required
5. **Flashbots executor** — bundle building + MEV-Share integration
6. **Arbitrage** — pool graph + Bellman-Ford + BundleExecutor + flashloan
7. **Multi-chain** — ARB then Base, parameterize chain-specific logic
8. **Monitoring** — Telegram, CSV PnL, killswitch, dashboards

---

## Key Docs Reference

| Doc | Why Read It |
|-----|-------------|
| `First key to building MEV bots - The simulation engine.md` | revm simulation architecture |
| `Avoiding Trap Tokens With eth_call.md` | salmonella/honeypot detection |
| `Wrecking sandwich traders for fun and profit.md` | sandwich mechanics + edge cases |
| `The fastest draw on the Blockchain - Ethereum Backrunning.md` | backrun patterns |
| `MEV-Share - programmably private orderflow.md` | MEV-Share integration |
| `Flash Boys 2.0 - Frontrunning, Transaction Reordering...` | foundational paper |
| `How I've built an unprofitable Crypto MEV Bot in Rust.md` | Rust-specific pitfalls |
| `Ethereum is a Dark Forest.md` | generalized frontrunner threat model |
| `A Tale of Two Arbitrages.md` | atomic vs CeFi-DeFi comparison |
| `High-Frequency Trading on Decentralized On-Chain Exchanges.md` | HFT on DEX mechanics |
| `Understanding Automated Market-Makers, Part 1 - Price Impact.md` | AMM math |
| `MEV - Maximal Extractable Value Pt. 1.md` + `Pt. 2.md` | comprehensive overview |
| `docs/Solid Quant/` | quantitative strategy research |
| `docs/flashbots-docs/` | Flashbots system internals |
| `docs/mev-research/` | research-grade MEV papers |
