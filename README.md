# mev-wiki

A personal research repository for MEV (Maximal Extractable Value) on EVM chains.

This is a showcase and paper-trading implementation built from a deep investigation into MEV strategies, tooling, and infrastructure. The goal was not just to understand MEV theory but to build something production-grade enough to validate the concepts end-to-end — without live capital.

---

## What's inside

### `mev-bot/`
A production-architecture MEV bot written in Rust, targeting:
- **Strategies:** sandwich attacks (V2 + V3) and DEX arbitrage (cross-DEX)
- **Networks:** Ethereum mainnet, Arbitrum, Base
- **Submission:** Flashbots bundles + MEV-Share (dry-run mode — no live funds required)

Full pipeline: mempool monitoring → swap decoding → revm simulation → profit estimation → Flashbots bundle construction. See [`mev-bot/README.md`](mev-bot/README.md) for details.

### `codebases/`
Reference codebases studied during research:

| Name | Focus |
|------|-------|
| `rusty-sando` | V2/V3 multi-meat sandwiches, Artemis, Huff contracts |
| `sandooo` | Stablecoin sandwiches, multi-bundle grouping, salmonella detection |
| `subway-rs` | Simple V2 sandwich, Flashbots relayer pattern |
| `simple-arbitrage` | Clean BundleExecutor + Flashbots pattern (TypeScript) |
| `mev-design` | LST arb, Alloy + revm, multi-chain patterns |
| `artemis` | Paradigm's event-driven MEV framework |
| `cfmms-rs` | CFMM math and pool syncing |
| `mev-rs` | MEV-Boost, relay, validator/builder integration |
| `revm-is-all-you-need` | revm simulation patterns |
| `ethers-flashbots` | Flashbots middleware for ethers-rs |

### `docs/`
Knowledge base assembled during research:

- MEV mechanics, sandwich and arbitrage strategy deep-dives
- Flashbots internals and bundle submission
- MEV-Share programmable orderflow
- Simulation engine architecture (revm fork patterns)
- Trap token / salmonella detection
- CEX-DEX arbitrage analysis
- Quantitative strategy research (`docs/Solid Quant/`)
- Academic papers: Flash Boys 2.0, HFT on DEX, AMM price impact

---

## Context

This repository represents an investigation into the MEV landscape as it exists on Ethereum mainnet and L2s. The implementation is intentionally built to production standards — real revm simulation, real Flashbots bundle construction, real mempool decoding — but operates in dry-run mode (no live submissions, no capital at risk).

The primary value is educational and architectural: understanding what it actually takes to compete in the MEV space, what the bottlenecks are, and where the real alpha is.

---

## Status

- Simulation engine: complete
- Mempool watcher + swap decoder: complete
- Sandwich bundle builder (V2): complete
- Flashbots bundle construction + dry-run logging: complete
- Live submission: disabled (set `LIVE_MODE=1` to enable once contract is deployed)
- Arbitrage (Bellman-Ford cycle detection): planned
