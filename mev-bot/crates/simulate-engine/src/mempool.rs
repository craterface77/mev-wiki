use std::sync::Arc;

use alloy::consensus::Transaction as TxTrait;
use alloy::eips::eip2718::Encodable2718;
use alloy::network::TransactionResponse;
use alloy::primitives::{address, Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder, RootProvider, WsConnect};
use alloy::rpc::types::Transaction;
use futures::StreamExt;
use kanal::AsyncSender;
use tracing::{debug, info, warn};

use crate::pool_state::PoolStateStore;
use crate::token_index::TokenIndex;
use crate::types::{BlockCtx, DexVersion, SandwichCandidate, SimulateTx};

const UNISWAP_V2_ROUTER:       Address = address!("7a250d5630B4cF539739dF2C5dAcb4c659F2488D");
const SUSHISWAP_ROUTER:        Address = address!("d9e1cE17f2641f24aE83637ab66a2cca9C378B9F");
const UNISWAP_V3_ROUTER:       Address = address!("E592427A0AEce92De3Edee1F18E0157C05861564");
const UNISWAP_V3_ROUTER02:     Address = address!("68b3465833fb72A70ecDF485E0e4C7bD8665Fc45");
const UNISWAP_UNIVERSAL_OLD:   Address = address!("3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD");
const UNISWAP_UNIVERSAL_V4:    Address = address!("66a9893cC07D91D95644AEDD05d03f95e1dBA8Af");
const PANCAKE_V2_ROUTER:       Address = address!("10ED43C718714eb63d5aA57B78B54704E256024E");
const PANCAKE_V3_ROUTER:       Address = address!("13f4EA83D0bd40E75C8222255bc855a974568Dd4");

const SEL_EXACT_TOKENS_FOR_TOKENS:  [u8; 4] = [0x38, 0xed, 0x17, 0x39];
const SEL_TOKENS_FOR_EXACT_TOKENS:  [u8; 4] = [0x88, 0x03, 0xdb, 0xee];
const SEL_EXACT_ETH_FOR_TOKENS:     [u8; 4] = [0x7f, 0xf3, 0x6a, 0xb5];
const SEL_ETH_FOR_EXACT_TOKENS:     [u8; 4] = [0xfb, 0x3b, 0xdb, 0x41];
const SEL_EXACT_TOKENS_FOR_ETH:     [u8; 4] = [0x18, 0xcb, 0xaf, 0xe5];
const SEL_TOKENS_FOR_EXACT_ETH:     [u8; 4] = [0x4a, 0x25, 0xd9, 0x4a];
const SEL_V3_EXACT_INPUT_SINGLE:    [u8; 4] = [0x41, 0x4b, 0xf3, 0x89];
const SEL_V3_EXACT_INPUT:           [u8; 4] = [0xc0, 0x4b, 0x8d, 0x59];
#[allow(dead_code)]
const SEL_V3_EXACT_OUTPUT_SINGLE:   [u8; 4] = [0xdb, 0x3e, 0x21, 0x98];
#[allow(dead_code)]
const SEL_V3_EXACT_OUTPUT:          [u8; 4] = [0xf2, 0x8c, 0x04, 0x87];
const SEL_V3_EXACT_INPUT_SINGLE_R2: [u8; 4] = [0x04, 0xe4, 0x5a, 0xaf];
const SEL_MULTICALL:                [u8; 4] = [0x5a, 0xe4, 0x01, 0xdc];
const SEL_MULTICALL2:               [u8; 4] = [0x1f, 0x0f, 0xac, 0x7f];
const SEL_EXECUTE:                  [u8; 4] = [0x24, 0x85, 0x6b, 0xc3];
const SEL_EXECUTE_DEADLINE:         [u8; 4] = [0x3c, 0xc8, 0x31, 0xe5];

#[derive(Debug)]
pub struct DecodedSwap {
    pub token_in:    Address,
    pub token_out:   Address,
    pub amount_in:   U256,
    pub zero_for_one: bool,
    pub dex_hint:    DexVersion,
}

pub fn decode_swap(to: Address, data: &Bytes, value: U256) -> Option<DecodedSwap> {
    if data.len() < 4 { return None; }
    let sel: [u8; 4] = data[..4].try_into().ok()?;

    let is_v2_router = to == UNISWAP_V2_ROUTER || to == SUSHISWAP_ROUTER || to == PANCAKE_V2_ROUTER;
    let is_v3_router = to == UNISWAP_V3_ROUTER || to == PANCAKE_V3_ROUTER;
    let is_router02  = to == UNISWAP_V3_ROUTER02;
    let is_universal = to == UNISWAP_UNIVERSAL_OLD || to == UNISWAP_UNIVERSAL_V4;

    match sel {
        SEL_EXACT_ETH_FOR_TOKENS | SEL_ETH_FOR_EXACT_TOKENS if is_v2_router => {
            let dex = if to == SUSHISWAP_ROUTER { DexVersion::SushiswapV2 }
                      else if to == PANCAKE_V2_ROUTER { DexVersion::PancakeV2 }
                      else { DexVersion::UniswapV2 };
            decode_v2_eth_in(data, value, dex)
        }

        SEL_EXACT_TOKENS_FOR_TOKENS | SEL_TOKENS_FOR_EXACT_TOKENS
        | SEL_EXACT_TOKENS_FOR_ETH  | SEL_TOKENS_FOR_EXACT_ETH
            if is_v2_router =>
        {
            let dex = if to == SUSHISWAP_ROUTER { DexVersion::SushiswapV2 }
                      else if to == PANCAKE_V2_ROUTER { DexVersion::PancakeV2 }
                      else { DexVersion::UniswapV2 };
            decode_v2_tokens(data, dex)
        }

        SEL_V3_EXACT_INPUT_SINGLE if is_v3_router => {
            let dex = if to == PANCAKE_V3_ROUTER { DexVersion::PancakeV3 } else { DexVersion::UniswapV3 };
            decode_v3_exact_input_single_old(data, dex)
        }
        SEL_V3_EXACT_INPUT if is_v3_router => {
            decode_v3_exact_input(data, DexVersion::UniswapV3)
        }

        // SwapRouter02
        SEL_V3_EXACT_INPUT_SINGLE_R2 if is_router02 => {
            decode_v3_exact_input_single_r2(data)
        }
        SEL_MULTICALL | SEL_MULTICALL2 if is_router02 => {
            decode_multicall(data)
        }

        SEL_EXECUTE | SEL_EXECUTE_DEADLINE if is_universal => {
            let dex = if to == UNISWAP_UNIVERSAL_V4 { DexVersion::UniswapV4 } else { DexVersion::UniswapV3 };
            decode_universal_router(data, value, dex)
        }

        _ => None,
    }
}

fn decode_v2_tokens(data: &Bytes, dex: DexVersion) -> Option<DecodedSwap> {
    let body = &data[4..];
    if body.len() < 5 * 32 { return None; }
    let amount_in   = U256::from_be_slice(&body[0..32]);
    let path_offset = U256::from_be_slice(&body[64..96]).saturating_to::<usize>();
    if path_offset + 64 > body.len() { return None; }
    let path_len    = U256::from_be_slice(&body[path_offset..path_offset + 32]).saturating_to::<usize>();
    if path_len < 2 { return None; }
    let token_in  = read_address(body, path_offset + 32)?;
    let token_out = read_address(body, path_offset + 64)?;
    Some(DecodedSwap { token_in, token_out, amount_in, zero_for_one: true, dex_hint: dex })
}

fn decode_v2_eth_in(data: &Bytes, value: U256, dex: DexVersion) -> Option<DecodedSwap> {
    let body = &data[4..];
    if body.len() < 4 * 32 { return None; }
    let path_offset = U256::from_be_slice(&body[32..64]).saturating_to::<usize>();
    if path_offset + 64 > body.len() { return None; }
    let path_len    = U256::from_be_slice(&body[path_offset..path_offset + 32]).saturating_to::<usize>();
    if path_len < 2 { return None; }
    let token_in  = read_address(body, path_offset + 32)?;
    let token_out = read_address(body, path_offset + 64)?;
    Some(DecodedSwap { token_in, token_out, amount_in: value, zero_for_one: true, dex_hint: dex })
}

// SwapRouter (old): exactInputSingle(tokenIn,tokenOut,fee,recipient,deadline,amountIn,...)
fn decode_v3_exact_input_single_old(data: &Bytes, dex: DexVersion) -> Option<DecodedSwap> {
    let body = &data[4..];
    if body.len() < 6 * 32 { return None; }
    let token_in  = read_address(body, 0)?;
    let token_out = read_address(body, 32)?;
    let amount_in = U256::from_be_slice(&body[160..192]);
    Some(DecodedSwap { token_in, token_out, amount_in, zero_for_one: true, dex_hint: dex })
}

// SwapRouter02: exactInputSingle((tokenIn,tokenOut,fee,recipient,amountIn,...))
fn decode_v3_exact_input_single_r2(data: &Bytes) -> Option<DecodedSwap> {
    let body = &data[4..];
    if body.len() < 5 * 32 { return None; }
    let token_in  = read_address(body, 0)?;
    let token_out = read_address(body, 32)?;
    let amount_in = U256::from_be_slice(&body[128..160]); // slot[4]
    Some(DecodedSwap { token_in, token_out, amount_in, zero_for_one: true, dex_hint: DexVersion::UniswapV3 })
}

// exactInput(path, recipient, deadline, amountIn) — multi-hop, take first two tokens
fn decode_v3_exact_input(data: &Bytes, dex: DexVersion) -> Option<DecodedSwap> {
    // path encoding: tokenIn(20) fee(3) tokenOut(20) ...
    let body = &data[4..];
    if body.len() < 4 * 32 { return None; }
    let path_offset = U256::from_be_slice(&body[0..32]).saturating_to::<usize>();
    if path_offset + 64 > body.len() { return None; }
    let path_len = U256::from_be_slice(&body[path_offset..path_offset + 32]).saturating_to::<usize>();
    if path_len < 43 { return None; }
    let path_start = path_offset + 32;
    if path_start + 43 > body.len() { return None; }
    let token_in  = Address::from_slice(&body[path_start..path_start + 20]);
    let token_out = Address::from_slice(&body[path_start + 23..path_start + 43]);
    let amount_in = U256::from_be_slice(&body[96..128]);
    Some(DecodedSwap { token_in, token_out, amount_in, zero_for_one: true, dex_hint: dex })
}

// UniversalRouter execute(bytes commands, bytes[] inputs, [uint256 deadline])
// Command bytes: 0x00=V3_EXACT_IN, 0x08=V2_EXACT_IN, 0x10=V4_SWAP
fn decode_universal_router(data: &Bytes, value: U256, _dex: DexVersion) -> Option<DecodedSwap> {
    let body = &data[4..];
    if body.len() < 3 * 32 { return None; }

    let cmd_offset    = U256::from_be_slice(&body[0..32]).saturating_to::<usize>();
    let inputs_offset = U256::from_be_slice(&body[32..64]).saturating_to::<usize>();

    if cmd_offset + 32 > body.len() { return None; }
    let cmd_len = U256::from_be_slice(&body[cmd_offset..cmd_offset + 32]).saturating_to::<usize>();
    if cmd_len == 0 { return None; }
    let cmd_start = cmd_offset + 32;
    if cmd_start + cmd_len > body.len() { return None; }

    let first_cmd = body[cmd_start] & 0x3F;

    if inputs_offset + 32 > body.len() { return None; }
    let inputs_len = U256::from_be_slice(&body[inputs_offset..inputs_offset + 32]).saturating_to::<usize>();
    if inputs_len == 0 { return None; }
    let first_input_ptr = inputs_offset + 32;
    if first_input_ptr + 32 > body.len() { return None; }
    let first_input_off = U256::from_be_slice(&body[first_input_ptr..first_input_ptr + 32])
        .saturating_to::<usize>() + inputs_offset + 32;
    if first_input_off + 32 > body.len() { return None; }
    let first_input_len  = U256::from_be_slice(&body[first_input_off..first_input_off + 32])
        .saturating_to::<usize>();
    let input_data_start = first_input_off + 32;
    if input_data_start + first_input_len > body.len() { return None; }
    let input = &body[input_data_start..input_data_start + first_input_len];

    match first_cmd {
        // V3_SWAP_EXACT_IN
        0x00 => {
            if input.len() < 4 * 32 { return None; }
            let amount_in = U256::from_be_slice(&input[32..64]);
            let path_off  = U256::from_be_slice(&input[96..128]).saturating_to::<usize>();
            if path_off + 32 > input.len() { return None; }
            let path_len  = U256::from_be_slice(&input[path_off..path_off + 32]).saturating_to::<usize>();
            let ps = path_off + 32;
            if path_len < 43 || ps + 43 > input.len() { return None; }
            let token_in  = Address::from_slice(&input[ps..ps + 20]);
            let token_out = Address::from_slice(&input[ps + 23..ps + 43]);
            Some(DecodedSwap { token_in, token_out, amount_in, zero_for_one: true, dex_hint: DexVersion::UniswapV3 })
        }
        // V2_SWAP_EXACT_IN
        0x08 => {
            if input.len() < 4 * 32 { return None; }
            let amount_in  = U256::from_be_slice(&input[32..64]);
            let path_off   = U256::from_be_slice(&input[96..128]).saturating_to::<usize>();
            if path_off + 32 > input.len() { return None; }
            let path_count = U256::from_be_slice(&input[path_off..path_off + 32]).saturating_to::<usize>();
            if path_count < 2 { return None; }
            let ps = path_off + 32;
            if ps + 64 > input.len() { return None; }
            let token_in  = Address::from_slice(&input[ps + 12..ps + 32]);
            let token_out = Address::from_slice(&input[ps + 44..ps + 64]);
            Some(DecodedSwap { token_in, token_out, amount_in, zero_for_one: true, dex_hint: DexVersion::UniswapV2 })
        }
        // V4_SWAP — best-effort PoolKey extraction
        0x10 => {
            if input.len() < 5 * 32 { return None; }
            let token_in  = Address::from_slice(&input[12..32]);
            let token_out = Address::from_slice(&input[44..64]);
            Some(DecodedSwap { token_in, token_out, amount_in: value, zero_for_one: true, dex_hint: DexVersion::UniswapV4 })
        }
        _ => None,
    }
}

fn decode_multicall(data: &Bytes) -> Option<DecodedSwap> {
    let body = &data[4..];
    if body.len() < 2 * 32 { return None; }
    let arr_offset = U256::from_be_slice(&body[32..64]).saturating_to::<usize>();
    if arr_offset + 32 > body.len() { return None; }
    let arr_len = U256::from_be_slice(&body[arr_offset..arr_offset + 32]).saturating_to::<usize>();
    if arr_len == 0 { return None; }
    let elem_ptr = arr_offset + 32;
    if elem_ptr + 32 > body.len() { return None; }
    let elem_off = U256::from_be_slice(&body[elem_ptr..elem_ptr + 32]).saturating_to::<usize>()
        + arr_offset + 32;
    if elem_off + 32 > body.len() { return None; }
    let elem_len = U256::from_be_slice(&body[elem_off..elem_off + 32]).saturating_to::<usize>();
    let es = elem_off + 32;
    if es + elem_len > body.len() || elem_len < 4 { return None; }
    let inner = Bytes::copy_from_slice(&body[es..es + elem_len]);
    decode_swap(UNISWAP_V3_ROUTER02, &inner, U256::ZERO)
}

#[inline]
fn read_address(data: &[u8], offset: usize) -> Option<Address> {
    if offset + 32 > data.len() { return None; }
    Some(Address::from_slice(&data[offset + 12..offset + 32]))
}

const MIN_PRICE_IMPACT_BPS: u64 = 10; // 0.1%
const MIN_GAS_FACTOR_NUM:   u64 = 11;
const MIN_GAS_FACTOR_DEN:   u64 = 10;

pub struct MempoolWatcher {
    ws_url:      String,
    pool_store:  Arc<PoolStateStore>,
    token_index: Arc<TokenIndex>,
    sender:      AsyncSender<SandwichCandidate>,
}

impl MempoolWatcher {
    pub fn new(
        ws_url:      String,
        pool_store:  Arc<PoolStateStore>,
        token_index: Arc<TokenIndex>,
        sender:      AsyncSender<SandwichCandidate>,
    ) -> Self {
        Self { ws_url, pool_store, token_index, sender }
    }

    pub async fn run(self) {
        loop {
            info!("MempoolWatcher: connecting to {}", self.ws_url);
            match self.run_inner().await {
                Ok(())  => info!("MempoolWatcher: stream ended, reconnecting..."),
                Err(e)  => warn!("MempoolWatcher: {e:#}, reconnecting..."),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    async fn run_inner(&self) -> anyhow::Result<()> {
        let ws = WsConnect::new(self.ws_url.clone());
        let provider: RootProvider =
            ProviderBuilder::new()
                .disable_recommended_fillers()
                .connect_ws(ws)
                .await?;

        let router_list: Vec<String> = [
            UNISWAP_V2_ROUTER, SUSHISWAP_ROUTER,
            UNISWAP_V3_ROUTER, UNISWAP_V3_ROUTER02,
            UNISWAP_UNIVERSAL_OLD, UNISWAP_UNIVERSAL_V4,
            PANCAKE_V2_ROUTER, PANCAKE_V3_ROUTER,
        ]
        .iter()
        .map(|a| format!("{a:?}"))
        .collect();

        let sub = provider
            .subscribe::<_, Transaction>(
                (
                    "alchemy_pendingTransactions",
                    serde_json::json!({
                        "toAddress": router_list,
                        "hashesOnly": false,
                    }),
                )
            )
            .await;

        let sub = match sub {
            Ok(s) => {
                info!("MempoolWatcher: using alchemy_pendingTransactions");
                s
            }
            Err(e) => {
                warn!("alchemy_pendingTransactions unavailable ({e:#}), falling back");
                provider.subscribe_full_pending_transactions().await?
            }
        };

        info!("MempoolWatcher: subscribed");

        let mut stream = sub.into_stream();
        while let Some(tx) = stream.next().await {
            self.process_tx(tx);
        }

        Ok(())
    }

    fn process_tx(&self, tx: Transaction) {
        let victim_raw: Option<Bytes> = {
            let mut buf = Vec::new();
            tx.inner.encode_2718(&mut buf);
            if buf.is_empty() { None } else { Some(Bytes::from(buf)) }
        };

        let from      = tx.from();
        let to        = tx.to();
        let input     = tx.inner.input().clone();
        let value     = tx.inner.value();
        let gas       = tx.inner.gas_limit();
        let gas_price_raw = tx.inner.gas_price()
            .unwrap_or_else(|| tx.inner.max_fee_per_gas());
        let priority_fee = tx.inner.max_priority_fee_per_gas();

        let Some(to_addr) = to else { return };

        let Some(decoded) = decode_swap(to_addr, &input, value) else { return };

        let block_number = self.pool_store.block_number();
        if block_number == 0 { return; }

        let base_fee  = U256::from(self.pool_store.base_fee());
        let gas_price = U256::from(gas_price_raw);

        let min_gas = base_fee * U256::from(MIN_GAS_FACTOR_NUM) / U256::from(MIN_GAS_FACTOR_DEN);
        if gas_price < min_gas { return; }

        let pairs = self.token_index.pairs_for_token(&decoded.token_in);
        if pairs.is_empty() { return; }

        for pair in pairs {
            let Some(pool_state) = self.pool_store.get(&pair) else { continue };
            let Some(info) = self.token_index.pair_info(&pair) else { continue };

            let pair_has_token_out = info.token0 == decoded.token_out
                || info.token1 == decoded.token_out;
            if !pair_has_token_out { continue; }

            let zero_for_one = info.token0 == decoded.token_in;

            let impact = pool_state.price_impact_bps(decoded.amount_in, zero_for_one);
            if impact < MIN_PRICE_IMPACT_BPS { continue; }

            if let crate::types::PoolState::V4(ref v4) = pool_state {
                if v4.has_hooks() { continue; }
            }

            let block_ctx = BlockCtx {
                number:    block_number,
                timestamp: 0,
                base_fee,
                max_fee:   base_fee * U256::from(2u64),
                coinbase:  Address::ZERO,
            };

            let victim = SimulateTx {
                caller:       from,
                to:           to_addr,
                calldata:     input.clone(),
                value,
                gas_limit:    gas,
                gas_price,
                priority_fee: priority_fee.map(U256::from),
            };

            let (token_in, token_out) = if zero_for_one {
                (info.token0, info.token1)
            } else {
                (info.token1, info.token0)
            };

            let pool_key_hash = match &pool_state {
                crate::types::PoolState::V3(v3) => {
                    use alloy::primitives::{keccak256, B256};
                    let mut key_data = [0u8; 5 * 32];
                    key_data[12..32].copy_from_slice(v3.token0.as_slice());
                    key_data[44..64].copy_from_slice(v3.token1.as_slice());
                    key_data[60..64].copy_from_slice(&v3.fee.to_be_bytes());
                    B256::from(keccak256(&key_data))
                }
                _ => alloy::primitives::B256::ZERO,
            };

            let candidate = SandwichCandidate {
                block_ctx,
                victim,
                victim_raw: victim_raw.clone(),
                target_pair:       pair,
                pool_state,
                dex_version:       info.version,
                zero_for_one,
                token_in,
                token_out,
                amount_in_ceiling: decoded.amount_in,
                pool_key_hash,
            };

            if let Err(e) = self.sender.try_send(candidate) {
                debug!("channel full: {e}");
            }
        }
    }
}
