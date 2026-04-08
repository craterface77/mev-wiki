# mev-bot

Production-architecture MEV bot in Rust. Dry-run by default — full pipeline runs locally, Flashbots bundles are constructed and logged but not submitted.

---

## Architecture

```
MempoolWatcher ──▶ SandwichCandidate ──▶ SimulateEngine ──▶ submit_bundle()
     │                                         │                    │
  WS pending txs                         revm fork sim        Flashbots bundle
  swap decoding                          profit estimate       (logged, not sent)
  pool matching                          gas optimization
```

Two crates:

- **`simulate-engine`** — core simulation layer: revm fork backend, mempool watcher, pool state, sandwich optimizer
- **`flashbots-executor`** — bundle construction, Flashbots relay client, EIP-1559 signing, mempool watch binary

---

## Crate overview

### `simulate-engine`

| File | Role |
|------|------|
| `backend.rs` | `SimulateBackend` — wraps provider + block number for revm fork state |
| `db.rs` | `ForkDb` — alloy-backed revm Database implementation |
| `executor.rs` | `SimulateExecutor` — EVM call/commit wrapper, trap token detection |
| `engine.rs` | `SimulateEngine` — binary search optimizer, sandwich profit estimation |
| `mempool.rs` | `MempoolWatcher` — WS pending tx subscription, swap ABI decoding |
| `pool_loader.rs` | `PoolLoader` — initial V2/V3 state load + live Sync/Swap event updates |
| `pool_state.rs` | `PoolStateStore` — thread-safe pool state map, price impact helpers |
| `block_stream.rs` | `BlockStream` — WS newHeads subscription, base fee tracking |
| `token_index.rs` | `TokenIndex` — token→pairs reverse index |
| `profit_calculator.rs` | ERC-20 balance slot detection, gas cost estimation |
| `calldata_builder.rs` | V2/V3 swap calldata encoding for simulation |
| `huff_calldata.rs` | Huff contract calldata encoding for front/back run txs |
| `types.rs` | Core types: `SandwichCandidate`, `BundleSimulateResult`, pool states |

### `flashbots-executor`

| File | Role |
|------|------|
| `bundle.rs` | `build_sandwich_bundle` — EIP-1559 tx signing, bundle assembly |
| `relay.rs` | `FlashbotsRelay` — eth_sendBundle HTTP client |
| `signer.rs` | `FlashbotsSigner` — X-Flashbots-Signature header (keccak + secp256k1) |
| `bin/mempool_watch.rs` | Main binary — wires everything together, runs dry-run pipeline |

### `contracts/`

Huff sandwich contract (`SandwichBot.huff`) + Foundry test suite. Not deployed.

---

## Quick start

```bash
cp mev-bot/.env.example mev-bot/.env
# fill ETH_RPC_HTTP, ETH_RPC_WS, SEARCHER_PRIVATE_KEY, BOT_CONTRACT_ADDRESS

cd mev-bot
cargo run --release --bin mempool_watch
```

Runs in dry-run mode by default. Set `LIVE_MODE=1` in `.env` to enable live submission (requires deployed contract and funded wallet).

### Benchmark

```bash
cargo run --release --bin simulate_bench
```

Runs sandwich optimization against a forked mainnet state without WS connection.

---

## Configuration

| Env var | Description |
|---------|-------------|
| `ETH_RPC_HTTP` | HTTP RPC endpoint (Alchemy/Infura recommended) |
| `ETH_RPC_WS` | WebSocket RPC endpoint |
| `SEARCHER_PRIVATE_KEY` | Bot wallet private key (hex, no 0x prefix) |
| `BOT_CONTRACT_ADDRESS` | Deployed sandwich contract address |
| `LIVE_MODE` | `1` = submit bundles, `0` = dry-run (default) |

---

## Strategies

### Sandwich (implemented)

1. Decode pending swap from mempool
2. Match against tracked pools (V2/V3 top pairs)
3. Binary-search optimal front-run size via revm simulation
4. Construct front + back txs with Huff calldata
5. Set priority fee bribe (80% of gross profit, 2/3 front / 1/3 back)
6. Build EIP-1559 bundle, log dry-run output

### Arbitrage (planned)

- Directed pool graph + Bellman-Ford negative cycle detection
- BundleExecutor contract + flashloan for capital efficiency
- Target: low-liquidity tokens, fresh pools, L2s

---

## Stack

- **EVM simulation:** `revm` 36 with forked chain state
- **Chain interaction:** `alloy` 1.7.3
- **Async runtime:** `tokio` with `kanal` channels
- **Concurrency:** `rayon` for parallel simulation workers
- **Allocator:** `mimalloc`
- **Contracts:** Huff (sandwich bot) + Solidity stubs
- **Bundles:** Flashbots eth_sendBundle + MEV-Share (planned)
