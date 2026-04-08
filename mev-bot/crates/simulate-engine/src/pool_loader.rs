use std::sync::Arc;

use alloy::primitives::{address, Address, Bytes, U256, B256};
use alloy::providers::{Provider, ProviderBuilder, RootProvider, WsConnect};
use alloy::pubsub::Subscription;
use alloy::rpc::types::{Filter, Log, TransactionRequest};
use futures::StreamExt;
use tracing::{debug, info, warn};

use crate::pool_state::PoolStateStore;
use crate::token_index::TokenIndex;
use crate::types::{DexVersion, PoolState, V2PoolState, V3PoolState};

pub const UNISWAP_V4_POOL_MANAGER: Address =
    address!("000000000004444c5dc75cB358380D2e3dE08A90");

pub const TOP_V2_PAIRS: &[(Address, Address, Address, u32, DexVersion, &str)] = &[
    (
        address!("B4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc"),
        address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), // USDC
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        30, DexVersion::UniswapV2, "USDC/WETH UniV2",
    ),
    (
        address!("0d4a11d5EEaaC28EC3F61d100daF4d40471f1852"),
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        address!("dAC17F958D2ee523a2206206994597C13D831ec7"), // USDT
        30, DexVersion::UniswapV2, "WETH/USDT UniV2",
    ),
    (
        address!("BB2B8038a1640196FbE3e38816F3e67Cba72D940"),
        address!("2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"), // WBTC
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        30, DexVersion::UniswapV2, "WBTC/WETH UniV2",
    ),
    (
        address!("A478c2975Ab1Ea89e8196811F51A7B7Ade33eB11"),
        address!("6B175474E89094C44Da98b954EedeAC495271d0F"), // DAI
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        30, DexVersion::UniswapV2, "DAI/WETH UniV2",
    ),
    (
        address!("d3d2E2692501A5c9Ca623199D38826e513033a17"),
        address!("1f9840a85d5aF5bf1D1762F925BDADdC4201F984"), // UNI
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        30, DexVersion::UniswapV2, "UNI/WETH UniV2",
    ),
    // Sushiswap V2
    (
        address!("397FF1542f962076d0BFE58eA045FfA2d347ACa0"),
        address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), // USDC
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        30, DexVersion::SushiswapV2, "USDC/WETH SushiV2",
    ),
    (
        address!("06da0fd433C1A5d7a4faa01111c044910A184553"),
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        address!("dAC17F958D2ee523a2206206994597C13D831ec7"), // USDT
        30, DexVersion::SushiswapV2, "WETH/USDT SushiV2",
    ),
];

pub const TOP_V3_POOLS: &[(Address, Address, Address, u32, DexVersion, &str)] = &[
    (
        address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640"),
        address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), // USDC
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        500, DexVersion::UniswapV3, "USDC/WETH 0.05% UniV3",
    ),
    (
        address!("8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8"),
        address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), // USDC
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        3000, DexVersion::UniswapV3, "USDC/WETH 0.3% UniV3",
    ),
    (
        address!("4585FE77225b41b697C938B018E2ac67Ac5a20c0"),
        address!("2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"), // WBTC
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        500, DexVersion::UniswapV3, "WBTC/WETH 0.05% UniV3",
    ),
    (
        address!("4e68Ccd3E89f51C3074ca5072bbAC773960dFa36"),
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        address!("dAC17F958D2ee523a2206206994597C13D831ec7"), // USDT
        3000, DexVersion::UniswapV3, "WETH/USDT 0.3% UniV3",
    ),
    (
        address!("5777d92f208679DB4b9778590Fa3CAB3aC9e2168"),
        address!("6B175474E89094C44Da98b954EedeAC495271d0F"), // DAI
        address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"), // USDC
        100, DexVersion::UniswapV3, "DAI/USDC 0.01% UniV3",
    ),
    (
        address!("1d42064Fc4Beb5F8aAF85F4617AE8b3b5B8Bd801"),
        address!("1f9840a85d5aF5bf1D1762F925BDADdC4201F984"), // UNI
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
        3000, DexVersion::UniswapV3, "UNI/WETH 0.3% UniV3",
    ),
];

// keccak256("Sync(uint112,uint112)")
const SYNC_TOPIC: B256 = alloy::primitives::b256!(
    "1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1"
);

// keccak256("Swap(address,address,int256,int256,uint160,uint128,int24)")
const V3_SWAP_TOPIC: B256 = alloy::primitives::b256!(
    "c42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67"
);

// Function selectors
const GET_RESERVES_SEL: [u8; 4] = [0x09, 0x02, 0xf1, 0xac];
const SLOT0_SEL:         [u8; 4] = [0x38, 0x50, 0xc7, 0xbd]; // slot0()
const LIQUIDITY_SEL:     [u8; 4] = [0x1a, 0x68, 0x65, 0x02]; // liquidity()

pub struct PoolLoader {
    ws_url:      String,
    http_url:    String,
    pool_store:  Arc<PoolStateStore>,
    token_index: Arc<TokenIndex>,
}

impl PoolLoader {
    pub fn new(
        ws_url:      String,
        http_url:    String,
        pool_store:  Arc<PoolStateStore>,
        token_index: Arc<TokenIndex>,
    ) -> Self {
        Self { ws_url, http_url, pool_store, token_index }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        self.load_initial_state().await?;

        loop {
            match self.run_event_listener().await {
                Ok(())  => info!("PoolLoader: stream ended, reconnecting..."),
                Err(e)  => warn!("PoolLoader: {e:#}, reconnecting..."),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    async fn load_initial_state(&self) -> anyhow::Result<()> {
        let provider: RootProvider =
            ProviderBuilder::new()
                .disable_recommended_fillers()
                .connect_http(self.http_url.parse()?);

        info!(
            "PoolLoader: loading {} V2 + {} V3 pools...",
            TOP_V2_PAIRS.len(),
            TOP_V3_POOLS.len(),
        );

        self.load_v2_reserves(&provider).await;
        self.load_v3_state(&provider).await;

        let (v2, v3, v4) = self.pool_store.pool_count_by_version();
        info!(
            "PoolLoader: loaded V2={v2} V3={v3} V4={v4}, {} tokens indexed",
            self.token_index.pair_count(),
        );

        Ok(())
    }

    async fn load_v2_reserves(&self, provider: &RootProvider) {
        let calldata = Bytes::from(GET_RESERVES_SEL.to_vec());

        for (pair, token0, token1, _fee, dex, label) in TOP_V2_PAIRS {
            let req = TransactionRequest::default()
                .to(*pair)
                .input(calldata.clone().into());

            match provider.call(req).await {
                Ok(result) => {
                    if let Some((r0, r1)) = decode_get_reserves(&result) {
                        self.pool_store.update(*pair, PoolState::V2(V2PoolState {
                            reserve0: r0, reserve1: r1,
                        }));
                        self.token_index.insert_pair(*pair, *token0, *token1, *dex);
                        info!("  {label} r0={r0} r1={r1}");
                    } else {
                        warn!("  {label} — bad reserves response");
                    }
                }
                Err(e) => warn!("  {label} — call failed: {e:#}"),
            }
        }
    }

    async fn load_v3_state(&self, provider: &RootProvider) {
        let slot0_data     = Bytes::from(SLOT0_SEL.to_vec());
        let liquidity_data = Bytes::from(LIQUIDITY_SEL.to_vec());

        for (pool, token0, token1, fee, dex, label) in TOP_V3_POOLS {
            let slot0_req = TransactionRequest::default()
                .to(*pool)
                .gas_limit(100_000)
                .input(slot0_data.clone().into());
            let liq_req = TransactionRequest::default()
                .to(*pool)
                .gas_limit(100_000)
                .input(liquidity_data.clone().into());

            let (slot0_res, liq_res) = tokio::join!(
                provider.call(slot0_req),
                provider.call(liq_req),
            );

            match (slot0_res, liq_res) {
                (Ok(s0), Ok(liq)) => {
                    if let Some((sqrt_price, tick)) = decode_slot0(&s0) {
                        let liquidity = decode_liquidity(&liq).unwrap_or(0);
                        let state = V3PoolState {
                            sqrt_price_x96: sqrt_price,
                            tick,
                            liquidity,
                            fee: *fee,
                            token0: *token0,
                            token1: *token1,
                        };
                        self.pool_store.update(*pool, PoolState::V3(state));
                        self.token_index.insert_pair(*pool, *token0, *token1, *dex);
                        info!("  {label} tick={tick} liq={liquidity}");
                    } else {
                        warn!("  {label} — bad slot0 response");
                    }
                }
                (Err(e), _) | (_, Err(e)) => warn!("  {label} — call failed: {e:#}"),
            }
        }
    }

    async fn run_event_listener(&self) -> anyhow::Result<()> {
        let ws = WsConnect::new(self.ws_url.clone());
        let provider: RootProvider =
            ProviderBuilder::new()
                .disable_recommended_fillers()
                .connect_ws(ws)
                .await?;

        let v2_addrs: Vec<Address> = TOP_V2_PAIRS.iter().map(|(p, ..)| *p).collect();
        let v3_addrs: Vec<Address> = TOP_V3_POOLS.iter().map(|(p, ..)| *p).collect();

        let v2_filter = Filter::new()
            .address(v2_addrs)
            .event_signature(SYNC_TOPIC);
        let sub_v2: Subscription<Log> = provider.subscribe_logs(&v2_filter).await?;
        let mut stream_v2 = sub_v2.into_stream();

        let v3_filter = Filter::new()
            .address(v3_addrs)
            .event_signature(V3_SWAP_TOPIC);
        let sub_v3: Subscription<Log> = provider.subscribe_logs(&v3_filter).await?;
        let mut stream_v3 = sub_v3.into_stream();

        info!("PoolLoader: subscribed to V2 Sync + V3 Swap events");

        loop {
            tokio::select! {
                Some(log) = stream_v2.next() => self.process_v2_sync(log),
                Some(log) = stream_v3.next() => self.process_v3_swap(log),
                else => break,
            }
        }

        Ok(())
    }

    fn process_v2_sync(&self, log: Log) {
        let pair = log.address();
        let data = log.data().data.as_ref();
        if data.len() < 64 { return; }
        let reserve0 = U256::from_be_slice(&data[0..32]);
        let reserve1 = U256::from_be_slice(&data[32..64]);
        self.pool_store.update_v2_reserves(pair, reserve0, reserve1);
        debug!("V2 Sync {} r0={reserve0} r1={reserve1}", pair);
    }

    fn process_v3_swap(&self, log: Log) {
        let pool = log.address();
        let data = log.data().data.as_ref();
        if data.len() < 5 * 32 { return; }
        let sqrt_price_x96 = U256::from_be_slice(&data[64..96]);
        let liquidity      = U256::from_be_slice(&data[96..128]).saturating_to::<u128>();
        let tick_raw       = U256::from_be_slice(&data[128..160]);
        let tick           = sign_extend_24(tick_raw);

        if let Some(mut state) = self.pool_store.get(&pool) {
            if let PoolState::V3(ref mut v3) = state {
                v3.sqrt_price_x96 = sqrt_price_x96;
                v3.liquidity      = liquidity;
                v3.tick           = tick;
            }
            self.pool_store.update(pool, state);
        }
        debug!("V3 Swap {} sqrtPrice={sqrt_price_x96} tick={tick}", pool);
    }
}

fn decode_get_reserves(data: &Bytes) -> Option<(U256, U256)> {
    if data.len() < 64 { return None; }
    let r0 = U256::from_be_slice(&data[0..32]);
    let r1 = U256::from_be_slice(&data[32..64]);
    if r0.is_zero() && r1.is_zero() { return None; }
    Some((r0, r1))
}

fn decode_slot0(data: &Bytes) -> Option<(U256, i32)> {
    if data.len() < 64 { return None; }
    let sqrt_price = U256::from_be_slice(&data[0..32]);
    let tick_raw   = U256::from_be_slice(&data[32..64]);
    if sqrt_price.is_zero() { return None; }
    Some((sqrt_price, sign_extend_24(tick_raw)))
}

fn decode_liquidity(data: &Bytes) -> Option<u128> {
    if data.len() < 32 { return None; }
    Some(U256::from_be_slice(&data[0..32]).saturating_to::<u128>())
}

#[inline]
fn sign_extend_24(raw: U256) -> i32 {
    let bytes = raw.to_be_bytes::<32>();
    let v = ((bytes[29] as u32) << 16) | ((bytes[30] as u32) << 8) | (bytes[31] as u32);
    if v & 0x0080_0000 != 0 {
        (v | 0xFF00_0000) as i32
    } else {
        v as i32
    }
}
