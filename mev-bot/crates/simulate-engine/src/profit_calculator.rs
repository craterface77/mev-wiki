use alloy::primitives::{keccak256, Address, B256, U256};
use alloy_provider::Provider;

use crate::db::ForkDb;
use crate::error::{SimulateResult};

// ERC-20 balanceOf storage slot: keccak256(abi.encode(address, mapping_slot))
pub fn known_balance_slot(token: Address) -> Option<u32> {
    // WETH
    if token == alloy::primitives::address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2") {
        return Some(3);
    }
    // USDC
    if token == alloy::primitives::address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48") {
        return Some(9);
    }
    // USDT
    if token == alloy::primitives::address!("dAC17F958D2ee523a2206206994597C13D831ec7") {
        return Some(2);
    }
    // DAI
    if token == alloy::primitives::address!("6B175474E89094C44Da98b954EedeAC495271d0F") {
        return Some(2);
    }
    // WBTC
    if token == alloy::primitives::address!("2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599") {
        return Some(0);
    }
    // UNI
    if token == alloy::primitives::address!("1f9840a85d5aF5bf1D1762F925BDADdC4201F984") {
        return Some(4);
    }
    None
}

pub fn balance_slot(account: Address, mapping_slot: u32) -> B256 {
    let mut buf = [0u8; 64];
    buf[12..32].copy_from_slice(account.as_slice());
    buf[60..64].copy_from_slice(&mapping_slot.to_be_bytes());
    keccak256(buf)
}

pub fn read_balance<P: Provider + Clone + Send + Sync + 'static>(
    db: &mut ForkDb<P>,
    token: Address,
    account: Address,
) -> SimulateResult<U256> {
    let slot_idx = find_balance_slot(db, token, account)?;
    let slot_key = balance_slot(account, slot_idx);
    db.read_storage(token, U256::from_be_bytes(*slot_key))
}

fn find_balance_slot<P: Provider + Clone + Send + Sync + 'static>(
    db: &mut ForkDb<P>,
    token: Address,
    account: Address,
) -> SimulateResult<u32> {
    if let Some(s) = known_balance_slot(token) {
        return Ok(s);
    }
    for slot in 0u32..20 {
        let key = balance_slot(account, slot);
        let val = db.read_storage(token, U256::from_be_bytes(*key)).unwrap_or(U256::ZERO);
        if !val.is_zero() {
            return Ok(slot);
        }
    }
    Ok(0)
}

pub struct BalanceSnapshot {
    pub token:   Address,
    pub account: Address,
    pub balance: U256,
}

impl BalanceSnapshot {
    pub fn take<P: Provider + Clone + Send + Sync + 'static>(
        db: &mut ForkDb<P>,
        token: Address,
        account: Address,
    ) -> SimulateResult<Self> {
        let balance = read_balance(db, token, account)?;
        Ok(Self { token, account, balance })
    }

    pub fn diff(&self, after: &BalanceSnapshot) -> i128 {
        if after.balance >= self.balance {
            (after.balance - self.balance).saturating_to::<i128>()
        } else {
            -((self.balance - after.balance).saturating_to::<i128>())
        }
    }
}

pub struct GasCostModel {
    pub base_fee:    U256,
    pub bribe_ratio: u64, // basis points
}

impl GasCostModel {
    pub fn new(base_fee: U256, bribe_ratio_bps: u64) -> Self {
        Self { base_fee, bribe_ratio: bribe_ratio_bps }
    }

    pub fn net_profit(&self, gross_profit: i128, total_gas: u64) -> i128 {
        if gross_profit <= 0 {
            return gross_profit;
        }
        let base_cost = (self.base_fee * U256::from(total_gas)).saturating_to::<i128>();
        let bribe     = gross_profit * (self.bribe_ratio as i128) / 10_000;
        gross_profit - base_cost - bribe
    }

    pub fn optimal_bribe(&self, gross_profit: i128, total_gas: u64) -> i128 {
        if gross_profit <= 0 { return 0; }
        let base_cost = (self.base_fee * U256::from(total_gas)).saturating_to::<i128>();
        let after_gas = gross_profit - base_cost;
        if after_gas <= 0 { return 0; }
        after_gas * (self.bribe_ratio as i128) / 10_000
    }
}

pub mod gas_estimates {
    pub const ERC20_TRANSFER_COLD:      u64 = 65_000;
    pub const ERC20_TRANSFER_WARM:      u64 = 30_000;
    pub const V2_SWAP:                  u64 = 80_000;
    pub const V2_SWAP_FOT:              u64 = 120_000;
    pub const V3_SWAP_1_TICK:           u64 = 130_000;
    pub const V3_SWAP_PER_EXTRA_TICK:   u64 = 15_000;
    pub const V4_SWAP:                  u64 = 85_000;
    pub const SANDWICH_V2: u64 = V2_SWAP * 2 + ERC20_TRANSFER_COLD * 2;
    pub const SANDWICH_V3: u64 = V3_SWAP_1_TICK * 2 + ERC20_TRANSFER_WARM * 2;
}
