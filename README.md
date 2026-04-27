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

Reference codebases studied during research (84 repos total):

#### Sandwich

| Name               | Focus                                                              |
| ------------------ | ------------------------------------------------------------------ |
| `rusty-sando`      | V2/V3 multi-meat sandwiches, Artemis, Huff contracts               |
| `sandooo`          | Stablecoin sandwiches, multi-bundle grouping, salmonella detection |
| `subway`           | Original libevm V2 sandwich reference                              |
| `subway-rs`        | Rust V2 sandwich, Flashbots relayer pattern                        |
| `sando-rs`         | Another Rust sandwich implementation                               |
| `optimal-sandwich` | Optimal sandwich sizing math                                       |

#### Arbitrage

| Name                         | Focus                                                 |
| ---------------------------- | ----------------------------------------------------- |
| `simple-arbitrage`           | Clean BundleExecutor + Flashbots pattern (TypeScript) |
| `simple-arbitrage-rs`        | Rust port of flashbots simple-arb                     |
| `amm-arbitrageur`            | Multi-DEX arb, classic reference                      |
| `mev-design`                 | LST arb, Alloy + revm, multi-chain patterns           |
| `cex-dex-arb-research`       | CEX/DEX arbitrage analysis                            |
| `loom`                       | Production Rust MEV framework with arb, Alloy-based   |
| `hindsight`                  | Flashbots backtesting / hindsight arb tool            |
| `univ3-revm-arbitrage`       | V3 arb with revm simulation                           |
| `uniswap-arbitrage-analysis` | Uniswap V2 arb analysis                               |
| `arbitrage-graph-engine`     | Graph-based cycle detection                           |
| `defi-path-finder`           | Path finding for arb routes                           |
| `BaseBuster`                 | Base-chain arb bot                                    |
| `rusty-john`                 | Rust arb bot                                          |
| `unibot-rs`                  | Rust Uniswap bot                                      |
| `FrontrunBot`                | Frontrun bot implementation                           |
| `Arbitrage-Example`          | Yul-optimized arb example                             |
| `Atomic-Arbitrage`           | Atomic arb reference                                  |
| `merkle-generator`           | Merkle proof utilities                                |

#### Liquidation

| Name                                | Focus                                      |
| ----------------------------------- | ------------------------------------------ |
| `New-Bedford`                       | Compound/Aave liquidation bot              |
| `liquidator-v3`                     | Solana/cross-chain liquidator (blockworks) |
| `Liquidator-Morpho`                 | Morpho protocol liquidator                 |
| `yield-liquidator`                  | Yield Protocol liquidation bot             |
| `abracadabra-money-liquidation-bot` | Abracadabra liquidation                    |
| `liqbot`                            | Liquity liquidation bot                    |
| `grim-reaper`                       | Generic DeFi liquidator                    |
| `aave-liquidation`                  | Aave liquidation example                   |
| `liquidation-bot-fall-2020`         | Early liquidation bot reference            |

#### Longtail / Sniping

| Name               | Focus                                    |
| ------------------ | ---------------------------------------- |
| `uniswapx-artemis` | UniswapX intent filling via Artemis      |
| `flashside`        | Flash loan side-channel exploits         |
| `cake_sniper`      | Token sniper (PancakeSwap)               |
| `WolfGameMEV`      | NFT/game MEV extraction                  |
| `apebot`           | Token sniper bot                         |
| `degenbot`         | Well-maintained Python arb/MEV framework |

#### Symbolic Execution

| Name               | Focus                                      |
| ------------------ | ------------------------------------------ |
| `hevm`             | Ethereum's official symbolic EVM           |
| `manticore`        | Trail of Bits symbolic execution framework |
| `pakala`           | Ethereum smart contract analyzer           |
| `rhoevm`           | Rust EVM symbolic execution                |
| `dl_symb_exec_sol` | Symbolic execution for Solidity            |
| `evm`              | EVM bytecode analyzer                      |

#### Frameworks & Infrastructure

| Name                     | Focus                                                  |
| ------------------------ | ------------------------------------------------------ |
| `artemis`                | Paradigm's event-driven MEV framework                  |
| `revm`                   | revm source — EVM simulation engine                    |
| `revm-is-all-you-need`   | revm simulation patterns                               |
| `amms-rs`                | Modern Alloy-based AMM library (successor to cfmms-rs) |
| `cfmms-rs`               | CFMM math and pool syncing                             |
| `mev-rs`                 | MEV-Boost, relay, validator/builder integration        |
| `ethers-flashbots`       | Flashbots middleware for ethers-rs                     |
| `mev-bundle-generator`   | Bundle construction patterns                           |
| `mev-template-rs`        | Rust MEV bot template                                  |
| `mev-flood`              | Flashbots load testing / flood tool                    |
| `mev-inspect-rs`         | Ethereum MEV inspector in Rust                         |
| `PoolSync`               | Pool state syncing library                             |
| `NodeDB`                 | Node DB caching layer                                  |
| `storage-extractor`      | Storage slot analysis (Dedaub)                         |
| `multicaller`            | Gas-efficient multicall contract                       |
| `swap-optimizer`         | Swap route optimization                                |
| `uni-v4-core-flashloans` | Uniswap V4 flash loans                                 |
| `huff-examples`          | Huff language examples                                 |
| `Yul-Optimization-Tips`  | Yul gas optimization techniques                        |
| `eth-sim`                | Ethereum simulation tool                               |
| `evm-simulation`         | EVM simulation examples                                |
| `evm-tracing-samples`    | EVM tracing for simulation debugging                   |
| `evm-bench`              | EVM stress tests and benchmarks                        |

#### Misc / Historical

| Name                             | Focus                                    |
| -------------------------------- | ---------------------------------------- |
| `bancor`                         | Front-running Bancor (early MEV history) |
| `flashboys2`                     | Flash Boys 2.0 paper data                |
| `mev`                            | Flash Boys 2.0 associated code           |
| `Flashloan-MEV`                  | Flash loan MEV platform                  |
| `mev-templates`                  | MEV bot templates                        |
| `mev-design`                     | LST arb + mempool monitor                |
| `swap-simulator-v1`              | Early swap simulator                     |
| `trustless-token-transfer-trade` | Trustless token trade reference          |
| `whack-a-mole`                   | Whack-a-mole MEV game                    |

#### Resource Dumps & Aggregators

| Name                    | Focus                                  |
| ----------------------- | -------------------------------------- |
| `mev-toolkit`           | Comprehensive MEV resources collection |
| `Mev_Book`              | MEV Book reference                     |
| `Awesome-MEV`           | Curated MEV resources list             |
| `awesome-MEV-resources` | Another MEV resources aggregator       |
| `awesome-mev-searching` | MEV searching resources                |
| `Dogetoshi-MEV`         | Dogetoshi's MEV research               |
| `mev-research`          | Flashbots research papers and notes    |
| `0xmebius-mev`          | 0xmebius MEV research                  |

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
