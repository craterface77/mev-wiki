#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::sync::Arc;
use std::time::Instant;

use alloy::primitives::{address, U256};
use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::network::Ethereum;
use dotenv::dotenv;
use tracing::info;

use simulate_engine::{
    backend::SimulateBackend,
    engine::SimulateEngine,
    types::{BlockCtx, DexVersion, PoolState, SandwichCandidate, SimulateTx, V2PoolState},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter("simulate_engine=debug,simulate_bench=info")
        .init();

    let rpc_url = std::env::var("ETH_RPC_HTTP")
        .unwrap_or_else(|_| "https://eth.llamarpc.com".to_string());

    info!("Connecting to: {rpc_url}");

    let provider: Arc<RootProvider<Ethereum>> = Arc::new(
        ProviderBuilder::new()
            .disable_recommended_fillers()
            .connect_http(rpc_url.parse()?)
    );

    let block_number = provider.get_block_number().await?;
    info!("Forking at block: {block_number}");

    let t0 = Instant::now();
    let backend = SimulateBackend::new(provider.clone(), block_number);
    info!("Backend ready in {:?}", t0.elapsed());

    let engine = SimulateEngine::new(backend);

    let target_pair   = address!("B4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"); // WETH/USDC UniV2
    let bot_address   = address!("DeadDeadDeadDeadDeadDeadDeadDeadDeadDead");
    let victim_caller = address!("1234567890123456789012345678901234567890");

    let block_ctx = BlockCtx {
        number:    block_number,
        timestamp: 0,
        base_fee:  U256::from(30_000_000_000u64),
        max_fee:   U256::from(60_000_000_000u64),
        coinbase:  address!("0000000000000000000000000000000000000000"),
    };

    let candidate = SandwichCandidate {
        block_ctx,
        victim: SimulateTx {
            caller:       victim_caller,
            to:           target_pair,
            calldata:     alloy::primitives::Bytes::default(),
            value:        U256::ZERO,
            gas_limit:    200_000,
            gas_price:    U256::from(31_000_000_000u64),
            priority_fee: None,
        },
        target_pair,
        pool_state: PoolState::V2(V2PoolState {
            reserve0: U256::from(1_000u64) * U256::from(10u64).pow(U256::from(18u32)),
            reserve1: U256::from(2_000_000u64) * U256::from(10u64).pow(U256::from(6u32)),
        }),
        dex_version:       DexVersion::UniswapV2,
        zero_for_one:      true,
        token_in:          address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        token_out:         address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), // USDC
        amount_in_ceiling: U256::from(10u64) * U256::from(10u64).pow(U256::from(18u32)),
        victim_raw:        None,
        pool_key_hash:     alloy::primitives::B256::ZERO,
    };

    info!("Running optimize_sandwich (quick_profit_check only, no real EVM calls)...");
    let t1 = Instant::now();
    let result = engine.optimize_sandwich(&candidate, bot_address)?;
    info!("Done in {:?}", t1.elapsed());

    match result {
        Some(sim) => info!(
            "net_profit={} gross={} gas_cost={} front_gas={} back_gas={}",
            sim.net_profit, sim.gross_profit, sim.gas_cost,
            sim.front_gas_used, sim.back_gas_used
        ),
        None => info!("No profitable sandwich found (filtered by quick_profit_check)"),
    }

    Ok(())
}
