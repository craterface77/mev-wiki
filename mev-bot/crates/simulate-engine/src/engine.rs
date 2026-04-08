use std::sync::Arc;

use alloy::primitives::{Address, U256};
use alloy_provider::Provider;
use rayon::prelude::*;

use crate::backend::SimulateBackend;
use crate::calldata_builder::{encode_v2_swap, encode_v3_swap};
use crate::error::{SimulateError, SimulateResult};
use crate::executor::SimulateExecutor;
use crate::profit_calculator::{balance_slot, gas_estimates, known_balance_slot, GasCostModel};
use crate::types::{BlockCtx, BundleSimulateResult, DexVersion, PoolState, SandwichCandidate, SimulateTx};

const SEARCH_INTERVALS: u64 = 10;
const SEARCH_ITERATIONS: u32 = 4;

const BRIBE_RATIO_BPS: u64 = 8_000;

pub struct SimulateEngine<P: Provider + Clone + Send + Sync + 'static> {
    backend: Arc<SimulateBackend<P>>,
}

impl<P: Provider + Clone + Send + Sync + 'static> SimulateEngine<P> {
    pub fn new(backend: SimulateBackend<P>) -> Self {
        Self { backend: Arc::new(backend) }
    }

    pub fn on_new_block(&self, block_number: u64) -> SimulateResult<()> {
        self.backend.update_block(block_number)
    }

    pub fn optimize_sandwich(
        &self,
        candidate: &SandwichCandidate,
        bot_address: Address,
    ) -> SimulateResult<Option<BundleSimulateResult>> {
        if !self.quick_profit_check(candidate) {
            return Ok(None);
        }

        let mut min_in = U256::ZERO;
        let mut max_in = candidate.amount_in_ceiling;
        let mut best: Option<BundleSimulateResult> = None;

        for _ in 0..SEARCH_ITERATIONS {
            let diff = max_in.saturating_sub(min_in);
            if diff.is_zero() { break; }
            let step = diff / U256::from(SEARCH_INTERVALS);
            if step.is_zero() { break; }

            let inputs: Vec<U256> = (0..=SEARCH_INTERVALS)
                .map(|i| min_in + U256::from(i) * step)
                .collect();

            let template_db = self.backend.fork_db();

            let results: Vec<(usize, SimulateResult<BundleSimulateResult>)> = inputs
                .par_iter()
                .enumerate()
                .map(|(idx, &amount_in)| {
                    let db = template_db.clone();
                    let mut executor = SimulateExecutor::new(db);
                    let r = simulate_sandwich_once(
                        &mut executor,
                        &candidate.block_ctx,
                        &candidate.victim,
                        &candidate.pool_state,
                        candidate.dex_version,
                        candidate.zero_for_one,
                        candidate.token_in,
                        candidate.token_out,
                        candidate.target_pair,
                        amount_in,
                        bot_address,
                    );
                    (idx, r)
                })
                .collect();

            let mut best_idx = 0;
            let mut best_profit = i128::MIN;

            for (idx, result) in &results {
                if let Ok(sim) = result {
                    if sim.net_profit > best_profit {
                        best_profit = sim.net_profit;
                        best_idx = *idx;
                        best = Some(sim.clone());
                    }
                }
            }

            if best_profit <= 0 {
                return Ok(None);
            }

            min_in = if best_idx == 0 { U256::ZERO } else { inputs[best_idx - 1] };
            max_in = if best_idx == inputs.len() - 1 {
                inputs[best_idx]
            } else {
                inputs[best_idx + 1]
            };
        }

        Ok(best)
    }

    fn quick_profit_check(&self, candidate: &SandwichCandidate) -> bool {
        let victim_in = candidate.amount_in_ceiling;
        if victim_in.is_zero() { return false; }

        match &candidate.pool_state {
            PoolState::V2(pool) => {
                let front_in = victim_in / U256::from(2u64);
                if front_in.is_zero() { return false; }

                let front_out = pool.amount_out(front_in, candidate.zero_for_one);
                if front_out.is_zero() { return false; }

                let (r0_after_front, r1_after_front) = if candidate.zero_for_one {
                    (pool.reserve0 + front_in, pool.reserve1 - front_out)
                } else {
                    (pool.reserve0 - front_out, pool.reserve1 + front_in)
                };

                let pool_after_front = crate::types::V2PoolState {
                    reserve0: r0_after_front,
                    reserve1: r1_after_front,
                };
                let victim_out = pool_after_front.amount_out(victim_in, candidate.zero_for_one);

                let (r0_after_victim, r1_after_victim) = if candidate.zero_for_one {
                    (r0_after_front + victim_in, r1_after_front - victim_out)
                } else {
                    (r0_after_front - victim_out, r1_after_front + victim_in)
                };

                let pool_after_victim = crate::types::V2PoolState {
                    reserve0: r0_after_victim,
                    reserve1: r1_after_victim,
                };
                let back_out = pool_after_victim.amount_out(front_out, !candidate.zero_for_one);

                if back_out <= front_in { return false; }
                let gross = back_out - front_in;
                const MIN_GROSS: u128 = 1_000_000_000_000_000; // 0.001 ETH
                gross.saturating_to::<u128>() >= MIN_GROSS
            }
            PoolState::V3(_) | PoolState::V4(_) => {
                let impact = candidate.pool_state.price_impact_bps(victim_in, candidate.zero_for_one);
                impact >= 30
            }
        }
    }
}

fn simulate_sandwich_once<P: Provider + Clone + 'static>(
    executor: &mut SimulateExecutor<P>,
    block_ctx: &BlockCtx,
    victim: &SimulateTx,
    pool_state: &PoolState,
    dex_version: DexVersion,
    zero_for_one: bool,
    token_in: Address,
    token_out: Address,
    candidate_pair: Address,
    amount_in: U256,
    bot_address: Address,
) -> SimulateResult<BundleSimulateResult> {
    if amount_in.is_zero() {
        return Err(SimulateError::Unprofitable);
    }

    let expected_front_out = match pool_state {
        PoolState::V2(v2) => v2.amount_out(amount_in, zero_for_one),
        PoolState::V3(v3) => {
            let liq = U256::from(v3.liquidity);
            if liq.is_zero() { return Err(SimulateError::Unprofitable); }
            amount_in * liq / (liq + amount_in)
        }
        PoolState::V4(v4) => {
            let liq = U256::from(v4.liquidity);
            if liq.is_zero() { return Err(SimulateError::Unprofitable); }
            amount_in * liq / (liq + amount_in)
        }
    };

    let swap_gas_limit = match dex_version {
        DexVersion::UniswapV2 | DexVersion::SushiswapV2 | DexVersion::PancakeV2 =>
            gas_estimates::V2_SWAP + gas_estimates::ERC20_TRANSFER_COLD,
        DexVersion::UniswapV3 | DexVersion::SushiswapV3 | DexVersion::PancakeV3 =>
            gas_estimates::V3_SWAP_1_TICK + gas_estimates::ERC20_TRANSFER_WARM,
        DexVersion::UniswapV4 =>
            gas_estimates::V4_SWAP + gas_estimates::ERC20_TRANSFER_WARM,
        DexVersion::CurveStable =>
            gas_estimates::V3_SWAP_1_TICK,
    };

    let front_calldata = encode_front_calldata(pool_state, dex_version, zero_for_one, amount_in, bot_address)?;
    let back_calldata  = encode_back_calldata(pool_state, dex_version, zero_for_one, expected_front_out, bot_address)?;

    executor.set_eth_balance(bot_address, amount_in * U256::from(10u64));

    let pair_addr = candidate_pair;

    inflate_pair_balance(&mut executor.db, token_in, pair_addr, amount_in)?;

    let front_tx = SimulateTx {
        caller:       bot_address,
        to:           pair_addr,
        calldata:     front_calldata.clone(),
        value:        U256::ZERO,
        gas_limit:    swap_gas_limit,
        gas_price:    block_ctx.base_fee,
        priority_fee: None,
    };
    let front = executor.call_commit(block_ctx, &front_tx)?;

    let _ = executor.call_commit(block_ctx, victim);

    inflate_pair_balance(&mut executor.db, token_out, pair_addr, expected_front_out)?;

    let back_tx = SimulateTx {
        caller:       bot_address,
        to:           pair_addr,
        calldata:     back_calldata.clone(),
        value:        U256::ZERO,
        gas_limit:    swap_gas_limit,
        gas_price:    block_ctx.max_fee,
        priority_fee: None,
    };
    let back = executor.call_commit(block_ctx, &back_tx)?;

    let back_out = {
        let slot_idx = known_balance_slot(token_in).unwrap_or(0);
        let slot_key = balance_slot(bot_address, slot_idx);
        executor.db.read_storage(token_in, U256::from_be_bytes(*slot_key))
            .unwrap_or(U256::ZERO)
            .saturating_to::<i128>()
    };
    let amount_in_i128 = amount_in.saturating_to::<i128>();
    let gross = back_out - amount_in_i128;
    if gross <= 0 {
        return Err(SimulateError::Unprofitable);
    }

    let total_gas  = front.gas_used + back.gas_used;
    let cost_model = GasCostModel::new(block_ctx.base_fee, BRIBE_RATIO_BPS);
    let gas_cost   = (block_ctx.base_fee * U256::from(total_gas)).saturating_to::<i128>();
    let net_profit = cost_model.net_profit(gross, total_gas);

    if net_profit <= 0 {
        return Err(SimulateError::Unprofitable);
    }

    Ok(BundleSimulateResult {
        net_profit,
        gross_profit:      gross,
        gas_cost,
        front_gas_used:    front.gas_used,
        back_gas_used:     back.gas_used,
        front_calldata,
        back_calldata,
        front_access_list: vec![],
        back_access_list:  vec![],
        front_optimal_in:  amount_in,
        front_out:         expected_front_out,
    })
}

fn inflate_pair_balance<P: alloy_provider::Provider + Clone + 'static>(
    db: &mut crate::db::ForkDb<P>,
    token: Address,
    pair: Address,
    delta: U256,
) -> SimulateResult<()> {
    if delta.is_zero() { return Ok(()); }
    let slot_idx = known_balance_slot(token).unwrap_or(0);
    let slot_key = balance_slot(pair, slot_idx);
    let slot = U256::from_be_bytes(*slot_key);
    let current = db.read_storage(token, slot).unwrap_or(U256::ZERO);
    db.set_storage(token, slot, current + delta)
}

fn encode_front_calldata(
    pool_state: &PoolState,
    _dex_version: DexVersion,
    zero_for_one: bool,
    amount_in: U256,
    recipient: Address,
) -> SimulateResult<alloy::primitives::Bytes> {
    match pool_state {
        PoolState::V2(v2) => {
            let amount_out = v2.amount_out(amount_in, zero_for_one);
            Ok(encode_v2_swap(amount_out, zero_for_one, recipient))
        }
        PoolState::V3(_) => Ok(encode_v3_swap(amount_in, zero_for_one, recipient)),
        PoolState::V4(_) => Ok(encode_v3_swap(amount_in, zero_for_one, recipient)),
    }
}

fn encode_back_calldata(
    pool_state: &PoolState,
    _dex_version: DexVersion,
    zero_for_one: bool,
    amount_in: U256,
    recipient: Address,
) -> SimulateResult<alloy::primitives::Bytes> {
    let reverse = !zero_for_one;
    match pool_state {
        PoolState::V2(v2) => {
            let amount_out = v2.amount_out(amount_in, reverse);
            Ok(encode_v2_swap(amount_out, reverse, recipient))
        }
        PoolState::V3(_) => Ok(encode_v3_swap(amount_in, reverse, recipient)),
        PoolState::V4(_) => Ok(encode_v3_swap(amount_in, reverse, recipient)),
    }
}
