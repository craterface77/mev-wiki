use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use alloy::primitives::Address;
use dashmap::DashMap;

use crate::types::{PoolState, V2PoolState};

#[derive(Clone)]
pub struct PoolStateStore {
    pools:        Arc<DashMap<Address, PoolState>>,
    block_number: Arc<AtomicU64>,
    base_fee:     Arc<AtomicU64>,
}

impl PoolStateStore {
    pub fn new() -> Self {
        Self {
            pools:        Arc::new(DashMap::new()),
            block_number: Arc::new(AtomicU64::new(0)),
            base_fee:     Arc::new(AtomicU64::new(0)),
        }
    }

    #[inline]
    pub fn get(&self, address: &Address) -> Option<PoolState> {
        self.pools.get(address).map(|r| *r)
    }

    #[inline]
    pub fn update(&self, address: Address, state: PoolState) {
        self.pools.insert(address, state);
    }

    #[inline]
    pub fn update_v2_reserves(&self, address: Address, reserve0: alloy::primitives::U256, reserve1: alloy::primitives::U256) {
        self.pools
            .entry(address)
            .and_modify(|s| {
                if let PoolState::V2(ref mut v2) = s {
                    v2.reserve0 = reserve0;
                    v2.reserve1 = reserve1;
                }
            })
            .or_insert(PoolState::V2(V2PoolState { reserve0, reserve1 }));
    }

    #[inline]
    pub fn block_number(&self) -> u64 {
        self.block_number.load(Ordering::Acquire)
    }

    #[inline]
    pub fn set_block_number(&self, n: u64) {
        self.block_number.store(n, Ordering::Release);
    }

    #[inline]
    pub fn base_fee(&self) -> u64 {
        self.base_fee.load(Ordering::Acquire)
    }

    #[inline]
    pub fn set_base_fee(&self, fee: u64) {
        self.base_fee.store(fee, Ordering::Release);
    }

    pub fn pool_count(&self) -> usize {
        self.pools.len()
    }

    pub fn pool_count_by_version(&self) -> (usize, usize, usize) {
        let mut v2 = 0usize;
        let mut v3 = 0usize;
        let mut v4 = 0usize;
        for r in self.pools.iter() {
            match r.value() {
                PoolState::V2(_) => v2 += 1,
                PoolState::V3(_) => v3 += 1,
                PoolState::V4(_) => v4 += 1,
            }
        }
        (v2, v3, v4)
    }
}

impl Default for PoolStateStore {
    fn default() -> Self { Self::new() }
}
