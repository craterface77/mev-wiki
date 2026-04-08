use alloy::primitives::{Address, Bytes, B256, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DexVersion {
    UniswapV2,
    UniswapV3,
    UniswapV4,
    SushiswapV2,
    SushiswapV3,
    PancakeV2,
    PancakeV3,
    CurveStable,
}

impl std::fmt::Display for DexVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UniswapV2   => write!(f, "Uniswap V2"),
            Self::UniswapV3   => write!(f, "Uniswap V3"),
            Self::UniswapV4   => write!(f, "Uniswap V4"),
            Self::SushiswapV2 => write!(f, "Sushiswap V2"),
            Self::SushiswapV3 => write!(f, "Sushiswap V3"),
            Self::PancakeV2   => write!(f, "Pancake V2"),
            Self::PancakeV3   => write!(f, "Pancake V3"),
            Self::CurveStable => write!(f, "Curve Stable"),
        }
    }
}

impl DexVersion {
    #[inline]
    pub fn requires_evm_simulation(&self) -> bool {
        matches!(
            self,
            DexVersion::UniswapV3
                | DexVersion::UniswapV4
                | DexVersion::SushiswapV3
                | DexVersion::PancakeV3
                | DexVersion::CurveStable
        )
    }

    #[inline]
    pub fn fee_bps(&self) -> u64 {
        match self {
            DexVersion::UniswapV2
            | DexVersion::SushiswapV2
            | DexVersion::PancakeV2 => 30,
            DexVersion::UniswapV3
            | DexVersion::SushiswapV3
            | DexVersion::PancakeV3 => 30,
            DexVersion::UniswapV4   => 30,
            DexVersion::CurveStable => 4,
        }
    }
}

// 64 bytes = one cache line
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct V2PoolState {
    pub reserve0: U256,
    pub reserve1: U256,
}

impl V2PoolState {
    // amountOut = (amountIn * 997 * reserveOut) / (reserveIn * 1000 + amountIn * 997)
    #[inline(always)]
    pub fn amount_out(&self, amount_in: U256, zero_for_one: bool) -> U256 {
        let (reserve_in, reserve_out) = if zero_for_one {
            (self.reserve0, self.reserve1)
        } else {
            (self.reserve1, self.reserve0)
        };
        let fee_in      = amount_in * U256::from(997u64);
        let numerator   = fee_in * reserve_out;
        let denominator = reserve_in * U256::from(1000u64) + fee_in;
        if denominator.is_zero() { U256::ZERO } else { numerator / denominator }
    }

    #[inline(always)]
    pub fn price_impact_bps(&self, amount_in: U256, zero_for_one: bool) -> u64 {
        let reserve_in = if zero_for_one { self.reserve0 } else { self.reserve1 };
        if reserve_in.is_zero() { return 10_000; }
        (amount_in.saturating_mul(U256::from(10_000u64)) / reserve_in)
            .saturating_to::<u64>()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct V3PoolState {
    pub sqrt_price_x96: U256,
    pub tick:           i32,
    pub liquidity:      u128,
    pub fee:            u32,
    pub token0:         Address,
    pub token1:         Address,
}

impl V3PoolState {
    // Rough approximation for pre-filter only. Accurate simulation requires revm.
    #[inline]
    pub fn price_impact_bps_approx(&self, amount_in: U256, zero_for_one: bool) -> u64 {
        if self.liquidity == 0 { return 10_000; }
        let liq = U256::from(self.liquidity);
        if liq.is_zero() { return 10_000; }
        let price_adj = if zero_for_one {
            self.sqrt_price_x96
        } else {
            let q192 = U256::from(1u64) << 192;
            if self.sqrt_price_x96.is_zero() { return 10_000; }
            q192 / self.sqrt_price_x96
        };
        let virtual_reserve: U256 = (liq * price_adj) >> 96;
        if virtual_reserve.is_zero() { return 10_000; }
        (amount_in.saturating_mul(U256::from(10_000u64)) / virtual_reserve)
            .saturating_to::<u64>()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct V4PoolState {
    pub pool_id:        B256,
    pub sqrt_price_x96: U256,
    pub tick:           i32,
    pub liquidity:      u128,
    pub fee:            u32,
    pub token0:         Address,
    pub token1:         Address,
    pub hooks:          Address,
}

impl V4PoolState {
    #[inline]
    pub fn has_hooks(&self) -> bool {
        self.hooks != Address::ZERO
    }

    #[inline]
    pub fn price_impact_bps_approx(&self, amount_in: U256, zero_for_one: bool) -> u64 {
        if self.liquidity == 0 { return 10_000; }
        let liq = U256::from(self.liquidity);
        let price_adj = if zero_for_one {
            self.sqrt_price_x96
        } else {
            let q192 = U256::from(1u64) << 192;
            if self.sqrt_price_x96.is_zero() { return 10_000; }
            q192 / self.sqrt_price_x96
        };
        let virtual_reserve: U256 = (liq * price_adj) >> 96;
        if virtual_reserve.is_zero() { return 10_000; }
        (amount_in.saturating_mul(U256::from(10_000u64)) / virtual_reserve)
            .saturating_to::<u64>()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PoolState {
    V2(V2PoolState),
    V3(V3PoolState),
    V4(V4PoolState),
}

impl PoolState {
    pub fn version(&self) -> &'static str {
        match self {
            PoolState::V2(_) => "V2",
            PoolState::V3(_) => "V3",
            PoolState::V4(_) => "V4",
        }
    }

    pub fn requires_evm_simulation(&self) -> bool {
        matches!(self, PoolState::V3(_) | PoolState::V4(_))
    }

    pub fn price_impact_bps(&self, amount_in: U256, zero_for_one: bool) -> u64 {
        match self {
            PoolState::V2(s) => s.price_impact_bps(amount_in, zero_for_one),
            PoolState::V3(s) => s.price_impact_bps_approx(amount_in, zero_for_one),
            PoolState::V4(s) => s.price_impact_bps_approx(amount_in, zero_for_one),
        }
    }

    pub fn amount_out_fast(&self, amount_in: U256, zero_for_one: bool) -> Option<U256> {
        match self {
            PoolState::V2(s) => Some(s.amount_out(amount_in, zero_for_one)),
            _ => None,
        }
    }

    pub fn token0(&self) -> Option<Address> {
        match self {
            PoolState::V2(_) => None,
            PoolState::V3(s) => Some(s.token0),
            PoolState::V4(s) => Some(s.token0),
        }
    }

    pub fn token1(&self) -> Option<Address> {
        match self {
            PoolState::V2(_) => None,
            PoolState::V3(s) => Some(s.token1),
            PoolState::V4(s) => Some(s.token1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimulateTx {
    pub caller:       Address,
    pub to:           Address,
    pub calldata:     Bytes,
    pub value:        U256,
    pub gas_limit:    u64,
    pub gas_price:    U256,
    pub priority_fee: Option<U256>,
}

impl SimulateTx {
    #[inline]
    pub fn target_pool(&self) -> Address { self.to }
}

#[derive(Debug, Clone)]
pub struct BundleSimulateResult {
    pub net_profit:        i128,
    pub gross_profit:      i128,
    pub gas_cost:          i128,
    pub front_gas_used:    u64,
    pub back_gas_used:     u64,
    pub front_calldata:    Bytes,
    pub back_calldata:     Bytes,
    pub front_access_list: Vec<(Address, Vec<B256>)>,
    pub back_access_list:  Vec<(Address, Vec<B256>)>,
    pub front_optimal_in:  U256,
    pub front_out:         U256,
}

#[derive(Debug, Clone, Copy)]
pub struct BlockCtx {
    pub number:    u64,
    pub timestamp: u64,
    pub base_fee:  U256,
    pub max_fee:   U256,
    pub coinbase:  Address,
}

#[derive(Debug, Clone)]
pub struct SandwichCandidate {
    pub block_ctx:         BlockCtx,
    pub victim:            SimulateTx,
    pub victim_raw:        Option<Bytes>,
    pub target_pair:       Address,
    pub pool_state:        PoolState,
    pub dex_version:       DexVersion,
    pub zero_for_one:      bool,
    pub token_in:          Address,
    pub token_out:         Address,
    pub amount_in_ceiling: U256,
    /// keccak256(token0 ++ token1 ++ fee ++ tickSpacing ++ hooks) for V3; B256::ZERO for V2.
    pub pool_key_hash:     B256,
}
