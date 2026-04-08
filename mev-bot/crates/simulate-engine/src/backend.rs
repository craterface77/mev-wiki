use std::sync::Arc;

use alloy_provider::Provider;
use parking_lot::RwLock;

use crate::db::ForkDb;
use crate::error::SimulateResult;

pub struct SimulateBackend<P: Provider + Clone + 'static> {
    provider: Arc<P>,
    pinned_block: Arc<RwLock<u64>>,
}

impl<P: Provider + Clone + 'static> SimulateBackend<P> {
    pub fn new(provider: Arc<P>, block_number: u64) -> Self {
        Self {
            provider,
            pinned_block: Arc::new(RwLock::new(block_number)),
        }
    }

    #[inline]
    pub fn fork_db(&self) -> ForkDb<P> {
        let block = *self.pinned_block.read();
        ForkDb::new(self.provider.clone(), block)
    }

    pub fn update_block(&self, block_number: u64) -> SimulateResult<()> {
        *self.pinned_block.write() = block_number;
        Ok(())
    }

    pub fn pinned_block(&self) -> u64 {
        *self.pinned_block.read()
    }
}
