#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::Instant;

use alloy::signers::local::PrivateKeySigner;

use alloy::network::Ethereum;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use dotenv::dotenv;
use tracing::{debug, info, warn};

use simulate_engine::{
    backend::SimulateBackend,
    block_stream::BlockStream,
    engine::SimulateEngine,
    huff_calldata,
    mempool::MempoolWatcher,
    pool_loader::PoolLoader,
    pool_state::PoolStateStore,
    token_index::TokenIndex,
    types::{BundleSimulateResult, SandwichCandidate},
};

use flashbots_executor::{
    bundle::{build_sandwich_bundle, SandwichBundleParams},
    relay::FlashbotsRelay,
    signer::FlashbotsSigner,
};

const PAPER_BOT: Address = alloy::primitives::address!("B0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0");

struct PnlTracker {
    candidates_seen:  AtomicU64,
    simulations_run:  AtomicU64,
    profitable_count: AtomicU64,
    total_gross_wei:  AtomicI64,
    total_net_wei:    AtomicI64,
    total_gas_wei:    AtomicI64,
}

impl PnlTracker {
    fn new() -> Self {
        Self {
            candidates_seen:  AtomicU64::new(0),
            simulations_run:  AtomicU64::new(0),
            profitable_count: AtomicU64::new(0),
            total_gross_wei:  AtomicI64::new(0),
            total_net_wei:    AtomicI64::new(0),
            total_gas_wei:    AtomicI64::new(0),
        }
    }

    fn record_candidate(&self) {
        self.candidates_seen.fetch_add(1, Ordering::Relaxed);
    }

    fn record_result(&self, gross: i128, net: i128, gas: i128) {
        self.simulations_run.fetch_add(1, Ordering::Relaxed);
        if net > 0 {
            self.profitable_count.fetch_add(1, Ordering::Relaxed);
            self.total_gross_wei.fetch_add(gross.min(i64::MAX as i128) as i64, Ordering::Relaxed);
            self.total_net_wei.fetch_add(net.min(i64::MAX as i128) as i64, Ordering::Relaxed);
            self.total_gas_wei.fetch_add(gas.min(i64::MAX as i128) as i64, Ordering::Relaxed);
        }
    }

    fn print_summary(&self) {
        let seen   = self.candidates_seen.load(Ordering::Relaxed);
        let sims   = self.simulations_run.load(Ordering::Relaxed);
        let profit = self.profitable_count.load(Ordering::Relaxed);
        let gross  = self.total_gross_wei.load(Ordering::Relaxed) as f64 / 1e18;
        let net    = self.total_net_wei.load(Ordering::Relaxed) as f64 / 1e18;
        let gas    = self.total_gas_wei.load(Ordering::Relaxed) as f64 / 1e18;
        info!("┌─ SESSION PnL ────────────────────────────────────────");
        info!("│  candidates seen : {seen}");
        info!("│  simulations     : {sims}  profitable: {profit}");
        info!("│  gross profit    : {gross:+.6} ETH");
        info!("│  gas + bribe     : {gas:.6} ETH");
        info!("│  NET PROFIT      : {net:+.6} ETH  ← would have earned");
        info!("└──────────────────────────────────────────────────────");
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "simulate_engine=info,mempool_watch=info".to_string()),
        )
        .init();

    let http_url = std::env::var("ETH_RPC_HTTP")
        .unwrap_or_else(|_| "https://eth.llamarpc.com".to_string());
    let ws_url = std::env::var("ETH_RPC_WS")
        .unwrap_or_else(|_| "wss://eth.llamarpc.com".to_string());

    // LIVE_MODE=1 enables real Flashbots submission; default is paper trading (simulate only)
    let live_mode = std::env::var("LIVE_MODE").as_deref() == Ok("1");

    let bot_contract: Address = std::env::var("BOT_CONTRACT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(PAPER_BOT);

    let relay: Option<Arc<FlashbotsRelay>> = if live_mode {
        let pk = std::env::var("SEARCHER_PRIVATE_KEY")
            .expect("LIVE_MODE=1 requires SEARCHER_PRIVATE_KEY env var");
        let flashbots_signer = FlashbotsSigner::from_hex(&pk)?;
        info!("Live mode: searcher address = {}", flashbots_signer.address());
        Some(Arc::new(FlashbotsRelay::new(flashbots_signer)))
    } else {
        None
    };

    info!("┌─ MEV Bot ────────────────────────────────────────────────");
    info!("│  HTTP : {http_url}");
    info!("│  WS   : {ws_url}");
    info!("│  Bot  : {bot_contract}");
    info!("│  Mode : {}", if live_mode { "LIVE  ← REAL MONEY" } else { "PAPER (simulation only)" });
    info!("└──────────────────────────────────────────────────────────");

    let provider: Arc<RootProvider<Ethereum>> = Arc::new(
        ProviderBuilder::new()
            .disable_recommended_fillers()
            .connect_http(http_url.parse()?)
    );

    let block_number = provider.get_block_number().await?;
    info!("Forking from block #{block_number}\n");

    let tx_signer: Option<Arc<PrivateKeySigner>> = if live_mode {
        let pk = std::env::var("SEARCHER_PRIVATE_KEY")
            .expect("LIVE_MODE=1 requires SEARCHER_PRIVATE_KEY env var");
        let s: PrivateKeySigner = pk.trim_start_matches("0x").parse()
            .expect("invalid SEARCHER_PRIVATE_KEY");
        Some(Arc::new(s))
    } else {
        None
    };

    let nonce_counter: Arc<AtomicU64> = if live_mode {
        let addr = tx_signer.as_ref().unwrap().address();
        let n    = provider.get_transaction_count(addr).await?;
        info!("Live mode: initial nonce = {n}  addr = {addr}");
        Arc::new(AtomicU64::new(n))
    } else {
        Arc::new(AtomicU64::new(0))
    };

    let pool_store  = Arc::new(PoolStateStore::new());
    let token_index = Arc::new(TokenIndex::new());
    let pnl         = Arc::new(PnlTracker::new());

    let backend = SimulateBackend::new(provider.clone(), block_number);
    let engine  = Arc::new(SimulateEngine::new(backend));

    tokio::spawn({
        let loader = PoolLoader::new(
            ws_url.clone(), http_url.clone(),
            pool_store.clone(), token_index.clone(),
        );
        async move {
            if let Err(e) = loader.run().await {
                tracing::error!("PoolLoader: {e:#}");
            }
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    tokio::spawn({
        let engine_ref   = engine.clone();
        let pool_store_r = pool_store.clone();
        let ws           = ws_url.clone();
        async move {
            BlockStream::new(ws, pool_store_r)
                .run_with_callback(move |n| {
                    if let Err(e) = engine_ref.on_new_block(n) {
                        warn!("on_new_block: {e}");
                    }
                })
                .await;
        }
    });

    let (tx, rx) = kanal::unbounded_async::<SandwichCandidate>();
    tokio::spawn({
        let watcher = MempoolWatcher::new(
            ws_url, pool_store.clone(), token_index.clone(), tx,
        );
        async move { watcher.run().await }
    });

    info!("Watching mempool...\n");

    // ── Main loop ─────────────────────────────────────────────────
    loop {
        let candidate = match rx.recv().await {
            Ok(c)  => c,
            Err(e) => { tracing::error!("channel closed: {e}"); break; }
        };

        pnl.record_candidate();

        let engine2    = engine.clone();
        let pnl2       = pnl.clone();
        let relay2     = relay.clone();
        let signer2    = tx_signer.clone();
        let nonce2     = nonce_counter.clone();
        let bot_addr   = bot_contract;

        tokio::spawn(async move {
            let r = tokio::task::spawn_blocking(move || {
                simulate_candidate(engine2, pnl2, relay2, signer2, nonce2, bot_addr, live_mode, candidate)
            })
            .await;
            if let Err(e) = r {
                tracing::error!("spawn_blocking panic: {e:?}");
            }
        });
    }

    Ok(())
}

fn simulate_candidate(
    engine:    Arc<SimulateEngine<RootProvider<Ethereum>>>,
    pnl:       Arc<PnlTracker>,
    relay:     Option<Arc<FlashbotsRelay>>,
    signer:    Option<Arc<PrivateKeySigner>>,
    nonce:     Arc<AtomicU64>,
    bot_addr:  Address,
    live_mode: bool,
    candidate: SandwichCandidate,
) {
    let t0 = Instant::now();

    let impact_bps = candidate.pool_state.price_impact_bps(
        candidate.amount_in_ceiling,
        candidate.zero_for_one,
    );

    match engine.optimize_sandwich(&candidate, bot_addr) {
        Ok(Some(result)) => {
            let elapsed   = t0.elapsed();
            let gross_eth = result.gross_profit as f64 / 1e18;
            let net_eth   = result.net_profit   as f64 / 1e18;
            let gas_eth   = result.gas_cost      as f64 / 1e18;
            let bribe_eth = (result.gross_profit - result.gas_cost - result.net_profit)
                as f64 / 1e18;

            info!("╔═ SANDWICH [{elapsed:?}] ═══════════════════════════════════");
            info!("║  DEX      : {}  ({})", candidate.dex_version, candidate.pool_state.version());
            info!("║  pair     : {}", candidate.target_pair);
            info!("║  token_in : {}", candidate.token_in);
            info!("║  token_out: {}", candidate.token_out);
            info!("║  direction: {}", if candidate.zero_for_one { "token0 → token1" } else { "token1 → token0" });
            info!("║");
            info!("║  victim   : {}", candidate.victim.caller);
            info!("║  amount   : {:.6} ETH  impact: {impact_bps}bps",
                candidate.amount_in_ceiling.saturating_to::<u128>() as f64 / 1e18);
            info!("║  gas_price: {:.2} gwei",
                candidate.victim.gas_price.saturating_to::<u128>() as f64 / 1e9);
            info!("║  block    : #{}", candidate.block_ctx.number);
            info!("║");
            info!("║  PROFIT BREAKDOWN:");
            info!("║  gross    : {:+.8} ETH", gross_eth);
            info!("║  gas cost : -{gas_eth:.8} ETH  ({} + {} gas)",
                result.front_gas_used, result.back_gas_used);
            info!("║  bribe    : -{bribe_eth:.8} ETH  (80% → builder)");
            info!("║  ┌───────────────────────────────────");
            info!("║  │  NET    : {:+.8} ETH  ← {}", net_eth,
                if live_mode { "SUBMITTING" } else { "paper only" });
            info!("║  └───────────────────────────────────");
            info!("╚═══════════════════════════════════════════════════════");

            pnl.record_result(result.gross_profit, result.net_profit, result.gas_cost);
            pnl.print_summary();

            if live_mode {
                if let (Some(relay), Some(signer)) = (relay, signer) {
                    submit_bundle(relay, signer, nonce, &candidate, &result, bot_addr);
                }
            }
        }

        Ok(None) => {
            debug!(
                "✗ unprofitable  dex={}  pair={}  impact={impact_bps}bps  amount={:.4}ETH",
                candidate.dex_version,
                candidate.target_pair,
                candidate.amount_in_ceiling.saturating_to::<u128>() as f64 / 1e18,
            );
        }

        Err(e) => {
            debug!(
                "✗ sim error: {e}  pair={}  dex={}",
                candidate.target_pair,
                candidate.dex_version,
            );
        }
    }
}

const FRONT_GAS_LIMIT: u64 = 250_000;
const BACK_GAS_LIMIT:  u64 = 250_000;
const CHAIN_ID:        u64 = 1;

fn submit_bundle(
    relay:     Arc<FlashbotsRelay>,
    signer:    Arc<PrivateKeySigner>,
    nonce:     Arc<AtomicU64>,
    candidate: &SandwichCandidate,
    result:    &BundleSimulateResult,
    bot_addr:  Address,
) {
    let huff = match huff_calldata::build_huff_bundle(
        candidate.dex_version,
        &candidate.pool_state,
        candidate.target_pair,
        candidate.token_in,
        candidate.token_out,
        candidate.zero_for_one,
        result.front_optimal_in,
        result.front_out,
        candidate.pool_key_hash,
    ) {
        Some(h) => h,
        None => {
            warn!("submit_bundle: unsupported dex {} — skipping", candidate.dex_version);
            return;
        }
    };

    let victim_raw = match &candidate.victim_raw {
        Some(r) => r.clone(),
        None => {
            warn!("submit_bundle: no victim_raw bytes — cannot build bundle");
            return;
        }
    };

    let base_fee       = candidate.block_ctx.base_fee.saturating_to::<u128>();
    let gross          = result.gross_profit.max(0) as u128;
    let total_bribe    = gross * 80 / 100;
    // front carries 2/3 of bribe; floor at 1 gwei
    let front_priority = (total_bribe * 2 / 3 / FRONT_GAS_LIMIT as u128).max(1_000_000_000);
    let back_priority  = (total_bribe / 3     / BACK_GAS_LIMIT  as u128).max(1_000_000_000);
    let _base_fee      = base_fee;

    // atomic nonce: front and back use sequential nonces
    let front_nonce = nonce.fetch_add(2, Ordering::SeqCst);
    let back_nonce  = front_nonce + 1;

    let params = SandwichBundleParams {
        bot_contract:       bot_addr,
        searcher_signer:    (*signer).clone(),
        chain_id:           CHAIN_ID,
        front_nonce,
        back_nonce,
        front_gas_limit:    FRONT_GAS_LIMIT,
        back_gas_limit:     BACK_GAS_LIMIT,
        front_priority_fee: front_priority,
        back_priority_fee:  back_priority,
    };

    let bundle = match build_sandwich_bundle(
        &params, &huff, victim_raw, candidate, gross,
    ) {
        Ok(b)  => b,
        Err(e) => {
            warn!("build_sandwich_bundle: {e:#}");
            nonce.fetch_sub(2, Ordering::SeqCst);
            return;
        }
    };

    info!("┌─ BUNDLE READY ──────────────────────────────────────────");
    info!("│  block_target : #{}", bundle.block_number);
    info!("│  txs          : {}", bundle.txs.len());
    info!("│  front nonce  : {front_nonce}  back nonce: {back_nonce}");
    info!("│  front priority: {:.4} gwei", front_priority as f64 / 1e9);
    info!("│  back priority : {:.4} gwei", back_priority  as f64 / 1e9);
    info!("│  bribe total   : {:.8} ETH  ({:.0}% of gross)",
        total_bribe as f64 / 1e18, 80.0);
    info!("│  front calldata: 0x{}", hex::encode(&huff.front_calldata));
    info!("│  back calldata : 0x{}", hex::encode(&huff.back_calldata));
    info!("│  front raw len : {} bytes", bundle.txs.get(0).map(|t| t.raw.len()).unwrap_or(0));
    info!("│  victim raw len: {} bytes", bundle.txs.get(1).map(|t| t.raw.len()).unwrap_or(0));
    info!("│  back raw len  : {} bytes", bundle.txs.get(2).map(|t| t.raw.len()).unwrap_or(0));
    info!("└── [DRY RUN] ───────────────────────────────────────────");

    // Uncomment to enable live submission after deploying the contract:
    // let relay_clone = relay.clone();
    // tokio::runtime::Handle::current().block_on(async move {
    //     match relay_clone.send_bundle(&bundle).await {
    //         Ok(hash) => info!("bundle submitted: {hash}"),
    //         Err(e)   => warn!("bundle submission failed: {e}"),
    //     }
    // });
    let _ = relay;
}
