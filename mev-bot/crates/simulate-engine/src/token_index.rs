use std::sync::Arc;

use alloy::primitives::Address;
use dashmap::DashMap;

use crate::types::DexVersion;

#[derive(Debug, Clone, Copy)]
pub struct PairInfo {
    pub token0:  Address,
    pub token1:  Address,
    pub version: DexVersion,
}

#[derive(Clone, Default)]
pub struct TokenIndex {
    index: Arc<DashMap<Address, Vec<Address>>>,
    pairs: Arc<DashMap<Address, PairInfo>>,
}

impl TokenIndex {
    pub fn new() -> Self { Self::default() }

    pub fn insert_pair(&self, pair: Address, token0: Address, token1: Address, version: DexVersion) {
        self.pairs.insert(pair, PairInfo { token0, token1, version });
        self.index.entry(token0).or_default().push(pair);
        self.index.entry(token1).or_default().push(pair);
    }

    pub fn pairs_for_token(&self, token: &Address) -> Vec<Address> {
        self.index.get(token).map(|r| r.clone()).unwrap_or_default()
    }

    pub fn zero_for_one(&self, pair: &Address, token_in: &Address) -> Option<bool> {
        self.pairs.get(pair).map(|r| r.token0 == *token_in)
    }

    pub fn pair_info(&self, pair: &Address) -> Option<PairInfo> {
        self.pairs.get(pair).map(|r| *r)
    }

    pub fn pair_count(&self) -> usize { self.pairs.len() }
}
